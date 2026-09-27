use crate::{
    Result,
    lock::{LockFile, LockedComponent, LockedFile, safe_relative, sha256, write_lock},
    registry::Registry,
    support::{atomic_write, cleanup},
};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs, io,
    path::{Component, Path, PathBuf},
};
use toml_edit::{DocumentMut, Item, Table, value};
pub fn add_components(project: &Path, repo: &Path, names: &[String], dest: &str) -> Result<()> {
    safe_relative(dest)?;
    let registry = Registry::load(repo)?;
    let mut snapshots = Vec::new();
    let mut seen = BTreeSet::new();
    for name in names {
        for s in registry.resolve(repo, name)? {
            if seen.insert(s.component.clone()) {
                snapshots.push(s);
            }
        }
    }
    let manifest_path = project.join("Cargo.toml");
    let manifest_text = fs::read_to_string(&manifest_path)?;
    let mut manifest: DocumentMut = manifest_text.parse()?;
    if manifest.get("package").and_then(Item::as_table).is_none() {
        return Err("target Cargo.toml must contain a [package] table; virtual workspace roots cannot receive components".into());
    }
    let lock_path = project.join("mkit.toml");
    let mut lock_doc: DocumentMut =
        if lock_path.exists() { fs::read_to_string(&lock_path)?.parse()? } else { "".parse()? };
    let mut lock: LockFile = if lock_path.exists() {
        toml_edit::de::from_str(&lock_doc.to_string())?
    } else {
        LockFile { components: Vec::new() }
    };
    let mut writes: Vec<(PathBuf, Vec<u8>)> = Vec::new();
    let mut lock_updates = Vec::new();
    for snap in &snapshots {
        if lock.components.iter().any(|c| c.name == snap.component) {
            return Err(
                format!("component `{}` is already recorded in mkit.toml", snap.component).into()
            );
        }
        let mut locked_files = BTreeMap::new();
        for (source_rel, bytes) in &snap.files {
            let installed_rel =
                install_relative(&snap.component, source_rel, snap.files.len() == 1);
            let installed = PathBuf::from(dest).join(&installed_rel);
            safe_relative(installed.to_str().ok_or("invalid destination")?)?;
            let base = PathBuf::from(".mkit/base").join(&snap.component).join(source_rel);
            if escapes_existing_parent(project, &installed)
                || escapes_existing_parent(project, &base)
            {
                return Err(
                    format!("destination path escapes project: `{}`", installed.display()).into()
                );
            }
            if project.join(&installed).exists() || project.join(&base).exists() {
                return Err(format!(
                    "refusing to overwrite `{}`",
                    project.join(&installed).display()
                )
                .into());
            }
            let hash = sha256(bytes);
            writes.push((installed.clone(), bytes.clone()));
            writes.push((base.clone(), bytes.clone()));
            locked_files.insert(
                installed.to_string_lossy().replace('\\', "/"),
                LockedFile { base: base.to_string_lossy().replace('\\', "/"), sha256: hash },
            );
        }
        lock_updates.push(LockedComponent {
            name: snap.component.clone(),
            version: snap.version.clone(),
            origin: snap.origin.clone(),
            files: locked_files,
        });
    }
    let crates = snapshots
        .iter()
        .flat_map(|s| {
            registry
                .components
                .iter()
                .find(|c| c.name == s.component)
                .into_iter()
                .flat_map(|c| c.dependencies.clone())
        })
        .collect::<Vec<_>>();
    for dep in crates {
        let key = if dep.name == "gpui-pre" { "gpui_pre" } else { &dep.name };
        let inherited = workspace_dependency(project, key)?;
        insert_dependency_with_workspace(
            &mut manifest,
            &dep.name,
            &dep.version,
            inherited.as_ref(),
        )?;
    }
    // Write source and base files only after all paths and clobbers have been checked.
    let mut created = Vec::new();
    for (relative, bytes) in writes {
        let full = project.join(relative);
        if let Some(parent) = full.parent() {
            fs::create_dir_all(parent)?;
        }
        match fs::OpenOptions::new().write(true).create_new(true).open(&full) {
            Ok(mut file) => {
                created.push(full.clone());
                if let Err(e) = io::Write::write_all(&mut file, &bytes) {
                    cleanup(&created);
                    return Err(e.into());
                }
            }
            Err(e) => {
                cleanup(&created);
                return Err(format!("could not create {}: {e}", full.display()).into());
            }
        }
    }
    for update in lock_updates {
        lock.components.push(update);
    }
    let old_manifest = fs::read(&manifest_path)?;
    let old_lock = fs::read(&lock_path).ok();
    if let Err(e) = atomic_write(&manifest_path, manifest.to_string().as_bytes())
        .and_then(|_| write_lock(&mut lock_doc, &lock, &lock_path))
    {
        let _ = atomic_write(&manifest_path, &old_manifest);
        if let Some(old) = old_lock {
            let _ = atomic_write(&lock_path, &old);
        } else {
            let _ = fs::remove_file(&lock_path);
        }
        cleanup(&created);
        return Err(e);
    }
    Ok(())
}
#[cfg(test)]
pub(crate) fn insert_dependency(doc: &mut DocumentMut, name: &str, version: &str) -> Result<()> {
    insert_dependency_with_workspace(doc, name, version, None)
}

pub(crate) fn insert_dependency_with_workspace(
    doc: &mut DocumentMut,
    name: &str,
    version: &str,
    inherited: Option<&Item>,
) -> Result<()> {
    if doc.get("dependencies").is_none() {
        doc["dependencies"] = Item::Table(Table::new());
    }
    if !doc["dependencies"].is_table() {
        return Err("Cargo.toml dependencies must be a table".into());
    }
    let deps = doc["dependencies"].as_table_mut().ok_or("invalid dependencies table")?;
    let key = if name == "gpui-pre" { "gpui_pre" } else { name };
    if let Some(existing) = deps.get(key) {
        let is_gpui = name == "gpui-pre";
        let package = existing.get("package").and_then(Item::as_str);
        if existing.get("workspace").and_then(Item::as_bool) == Some(true) {
            let workspace = inherited.ok_or_else(|| format!("dependency `{key}` uses workspace = true, but no matching [workspace.dependencies] entry was found"))?;
            let resolved_package = workspace.get("package").and_then(Item::as_str).unwrap_or(key);
            let expected_package = if is_gpui { "gpui-pre" } else { name };
            if resolved_package != expected_package {
                return Err(format!("workspace dependency `{key}` refers to package `{resolved_package}`, but registry requires `{expected_package}`").into());
            }
            let resolved_version = workspace
                .as_str()
                .or_else(|| workspace.get("version").and_then(Item::as_str))
                .unwrap_or("")
                .trim_start_matches('=');
            if resolved_version != version {
                return Err(format!("workspace dependency `{key}` has version `{resolved_version}`, but registry requires `{version}`").into());
            }
            return Ok(());
        }
        if is_gpui && package != Some("gpui-pre") {
            return Err(format!("dependency key `{key}` must refer to package `gpui-pre` via `package = \"gpui-pre\"`").into());
        }
        if !is_gpui && package.is_some_and(|package| package != name) {
            return Err(format!(
                "dependency key `{key}` refers to package `{}`, but registry requires `{name}`",
                package.unwrap()
            )
            .into());
        }
        let got = existing
            .as_str()
            .or_else(|| existing.get("version").and_then(Item::as_str))
            .unwrap_or("");
        let got = got.trim_start_matches('=');
        if got != version {
            return Err(format!(
                "dependency `{key}` already has version `{got}`, but registry requires `{version}`"
            )
            .into());
        }
        return Ok(());
    }
    let mut d = Table::new();
    d["version"] = value(format!("={version}"));
    if name == "gpui-pre" {
        d["package"] = value("gpui-pre");
    }
    deps[key] = Item::Table(d);
    Ok(())
}

pub(crate) fn workspace_dependency(project: &Path, key: &str) -> Result<Option<Item>> {
    for ancestor in project.ancestors() {
        let path = ancestor.join("Cargo.toml");
        if !path.is_file() {
            continue;
        }
        let text = fs::read_to_string(path)?;
        let doc: DocumentMut = text.parse()?;
        if let Some(dep) =
            doc.get("workspace").and_then(|w| w.get("dependencies")).and_then(|d| d.get(key))
        {
            return Ok(Some(dep.clone()));
        }
        if doc.get("workspace").is_some() {
            return Ok(None);
        }
    }
    Ok(None)
}
fn install_relative(component: &str, source_rel: &str, single: bool) -> PathBuf {
    let p = Path::new(source_rel);
    let leaf = p.file_name().unwrap_or_default();
    let module_name = component.replace('-', "_");
    if single && leaf == "lib.rs" {
        PathBuf::from(format!("{module_name}.rs"))
    } else if single {
        PathBuf::from(leaf)
    } else {
        PathBuf::from(module_name).join(p)
    }
}
fn escapes_existing_parent(project: &Path, relative: &Path) -> bool {
    let root = match project.canonicalize() {
        Ok(p) => p,
        Err(_) => return true,
    };
    let mut candidate = project.to_path_buf();
    for c in relative.components() {
        if let Component::Normal(part) = c {
            candidate.push(part);
            if candidate.exists() {
                match candidate.canonicalize() {
                    Ok(p) if p.starts_with(&root) => {}
                    _ => return true,
                }
            } else {
                break;
            }
        }
    }
    false
}

#[cfg(test)]
mod installer_tests {
    use super::insert_dependency_with_workspace;
    use toml_edit::{DocumentMut, Item};

    #[test]
    fn accepts_matching_inherited_workspace_dependencies() {
        let mut manifest: DocumentMut =
            "[dependencies]\nmkit-core = { workspace = true }\ngpui_pre = { workspace = true }\n"
                .parse()
                .unwrap();
        let workspace: Item = "[workspace.dependencies]\nmkit-core = { version = \"=0.1.0\" }\ngpui_pre = { package = \"gpui-pre\", version = \"=0.3.5\" }\n".parse::<DocumentMut>().unwrap().as_item().clone();
        let deps = workspace.get("workspace").and_then(|w| w.get("dependencies")).unwrap();
        insert_dependency_with_workspace(
            &mut manifest,
            "mkit-core",
            "0.1.0",
            deps.get("mkit-core"),
        )
        .unwrap();
        insert_dependency_with_workspace(&mut manifest, "gpui-pre", "0.3.5", deps.get("gpui_pre"))
            .unwrap();
    }

    #[test]
    fn rejects_mismatched_inherited_workspace_dependencies() {
        let mut manifest: DocumentMut =
            "[dependencies]\nmkit-core = { workspace = true }\n".parse().unwrap();
        let inherited: Item = "{ version = \"=0.2.0\" }".parse().unwrap();
        let error =
            insert_dependency_with_workspace(&mut manifest, "mkit-core", "0.1.0", Some(&inherited))
                .unwrap_err();
        assert!(error.to_string().contains("requires `0.1.0`"));
    }
}
