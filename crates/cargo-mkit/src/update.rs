use crate::{
    Result,
    installer::{insert_dependency_with_workspace, workspace_dependency},
    lock::{LockFile, LockedFile, safe_relative, sha256, write_lock},
    registry::Registry,
};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
};
use toml_edit::DocumentMut;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct UpdateReport {
    pub components: Vec<ComponentUpdate>,
    pub check: bool,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ComponentUpdate {
    pub name: String,
    pub from: String,
    pub to: String,
    pub changed_files: usize,
    pub conflicts: usize,
}
struct LockedInput {
    name: String,
    version: String,
    origin: String,
    files: BTreeMap<String, LockedFile>,
}
impl UpdateReport {
    pub fn has_updates(&self) -> bool {
        !self.components.is_empty()
    }
}

/// Update locked source snapshots from the repository catalog. In check mode this only reads files.
pub fn update_components(
    project: &Path,
    repo: &Path,
    filter: Option<&str>,
    check: bool,
) -> Result<UpdateReport> {
    let lock_path = project.join("mkit.toml");
    if !lock_path.is_file() {
        return Err("mkit.toml not found; no installed components are recorded".into());
    }
    let mut lock_doc: DocumentMut = fs::read_to_string(&lock_path)?.parse()?;
    let mut lock: LockFile = toml_edit::de::from_str(&lock_doc.to_string())?;
    let selected = lock
        .components
        .iter()
        .filter(|c| filter.is_none_or(|n| c.name == n))
        .map(|c| LockedInput {
            name: c.name.clone(),
            version: c.version.clone(),
            origin: c.origin.clone(),
            files: c
                .files
                .iter()
                .map(|(path, file)| {
                    (
                        path.clone(),
                        LockedFile { base: file.base.clone(), sha256: file.sha256.clone() },
                    )
                })
                .collect(),
        })
        .collect::<Vec<_>>();
    if selected.is_empty() {
        return Err(match filter {
            Some(n) => format!("component `{n}` is not recorded in mkit.toml").into(),
            None => "mkit.toml contains no recorded components".into(),
        });
    }
    let registry = Registry::load(repo)?;
    let project_root = project.canonicalize()?;
    let repo_root = repo.canonicalize()?;
    let mut report = UpdateReport { check, ..Default::default() };
    let mut writes: BTreeMap<PathBuf, Vec<u8>> = BTreeMap::new();
    let mut deletes = BTreeSet::new();

    for locked in selected {
        let current =
            registry.components.iter().find(|c| c.name == locked.name).ok_or_else(|| {
                format!("component `{}` is not in the current registry", locked.name)
            })?;
        if current.status != "source_ready" {
            return Err(format!(
                "component `{}` is not installable (status: {})",
                locked.name, current.status
            )
            .into());
        }
        if locked.origin != format!("registry/{}", locked.name) {
            return Err(format!(
                "unexpected recorded origin `{}` for component `{}`",
                locked.origin, locked.name
            )
            .into());
        }
        let mut next_files = BTreeMap::new();
        let expected_base = Path::new(".mkit/base").join(&locked.name);
        let mut seen_upstream = BTreeSet::new();
        let mut changed = 0;
        let mut conflicts = 0;
        for (installed_s, record) in &locked.files {
            safe_relative(installed_s)?;
            safe_relative(&record.base)?;
            if !Path::new(&record.base).starts_with(&expected_base) {
                return Err(format!(
                    "recorded base `{}` is outside this component's .mkit/base snapshot",
                    record.base
                )
                .into());
            }
            let suffix = Path::new(&record.base).strip_prefix(&expected_base)?;
            let upstream_rel = Path::new(&locked.origin).join(suffix);
            let upstream_s = upstream_rel.to_string_lossy().replace('\\', "/");
            seen_upstream.insert(upstream_s.clone());
            let installed =
                safe_project_file(&project_root, &project.join(installed_s), installed_s)?;
            let base_path =
                safe_project_file(&project_root, &project.join(&record.base), &record.base)?;
            let base = fs::read(&base_path)?;
            if sha256(&base) != record.sha256 {
                return Err(format!("recorded base hash mismatch for `{}`", record.base).into());
            }
            let local = read_optional(&installed)?;
            let upstream_path = safe_repo_file(&repo_root, &repo.join(&upstream_rel), &upstream_s)?;
            let upstream = read_optional(&upstream_path)?;
            let plan = plan_file(local.as_deref(), &base, upstream.as_deref());
            if plan.output.as_deref() != local.as_deref()
                || plan.next_base.as_deref() != Some(base.as_slice())
            {
                changed += 1;
            }
            if plan.conflict {
                conflicts += 1;
            }
            let remove_base = plan.next_base.is_none();
            let delete_installed = plan.delete_installed;
            match plan.output {
                Some(bytes) => {
                    writes.insert(PathBuf::from(installed_s), bytes);
                }
                None => {
                    if delete_installed {
                        deletes.insert(PathBuf::from(installed_s));
                    }
                }
            }
            if let Some(bytes) = plan.next_base {
                writes.insert(PathBuf::from(&record.base), bytes.clone());
                next_files.insert(
                    installed_s.clone(),
                    LockedFile { base: record.base.clone(), sha256: sha256(&bytes) },
                );
            }
            if remove_base {
                deletes.insert(PathBuf::from(&record.base));
            }
        }
        // Current catalog files not represented by the old lock are upstream additions.
        for source in &current.source_files {
            safe_relative(source)?;
            if seen_upstream.contains(source) {
                continue;
            }
            if !Path::new(source).starts_with(Path::new(&locked.origin)) {
                return Err(format!("current source `{source}` is outside component origin").into());
            }
            let suffix = Path::new(source).strip_prefix(Path::new(&locked.origin))?;
            let dest = infer_destination(&locked.name, &locked.files)?;
            let installed_rel =
                dest.join(install_relative(&locked.name, suffix, current.source_files.len() == 1));
            let installed_s = installed_rel.to_string_lossy().replace('\\', "/");
            let base_s = expected_base.join(suffix).to_string_lossy().replace('\\', "/");
            safe_relative(&installed_s)?;
            safe_relative(&base_s)?;
            let source_path = safe_repo_file(&repo_root, &repo.join(source), source)?;
            let bytes = fs::read(source_path)?;
            let ip = safe_project_file(&project_root, &project.join(&installed_s), &installed_s)?;
            let bp = safe_project_file(&project_root, &project.join(&base_s), &base_s)?;
            if ip.exists() || bp.exists() {
                return Err(
                    format!("refusing to overwrite untracked update path `{installed_s}`").into()
                );
            }
            writes.insert(PathBuf::from(&installed_s), bytes.clone());
            writes.insert(PathBuf::from(&base_s), bytes.clone());
            next_files.insert(installed_s, LockedFile { base: base_s, sha256: sha256(&bytes) });
            changed += 1;
        }
        report.components.push(ComponentUpdate {
            name: locked.name.clone(),
            from: locked.version.clone(),
            to: current.version.clone(),
            changed_files: changed,
            conflicts,
        });
        if let Some(target) = lock.components.iter_mut().find(|c| c.name == locked.name) {
            target.version = current.version.clone();
            target.files = next_files;
        }
    }
    report.components.retain(|c| c.from != c.to || c.changed_files > 0);
    if check || report.components.is_empty() {
        return Ok(report);
    }
    // Validate every dependency requirement before writing any merged source. New
    // dependencies can be added, but an existing declaration is never silently
    // replaced: it may be an intentional application or workspace constraint.
    let manifest_path = project.join("Cargo.toml");
    let mut manifest: DocumentMut = fs::read_to_string(&manifest_path)?.parse()?;
    for component in &report.components {
        let entry =
            registry.components.iter().find(|entry| entry.name == component.name).ok_or_else(
                || format!("component `{}` is not in the current registry", component.name),
            )?;
        for dependency in &entry.dependencies {
            let key = if dependency.name == "gpui-pre" { "gpui_pre" } else { &dependency.name };
            let inherited = workspace_dependency(project, key)?;
            insert_dependency_with_workspace(
                &mut manifest,
                &dependency.name,
                &dependency.version,
                inherited.as_ref(),
            )?;
        }
    }
    let manifest_bytes = manifest.to_string().into_bytes();
    if fs::read(&manifest_path)? != manifest_bytes {
        writes.insert(PathBuf::from("Cargo.toml"), manifest_bytes);
    }
    let old_lock = fs::read(&lock_path)?;
    commit(project, writes, deletes, &mut lock_doc, &lock, &lock_path, &old_lock)?;
    Ok(report)
}

struct FilePlan {
    output: Option<Vec<u8>>,
    next_base: Option<Vec<u8>>,
    delete_installed: bool,
    conflict: bool,
}

fn plan_file(local: Option<&[u8]>, base: &[u8], upstream: Option<&[u8]>) -> FilePlan {
    match (local, upstream) {
        (None, None) => {
            FilePlan { output: None, next_base: None, delete_installed: true, conflict: false }
        }
        (None, Some(up)) if up == base => FilePlan {
            output: None,
            next_base: Some(up.to_vec()),
            delete_installed: false,
            conflict: false,
        },
        (None, Some(up)) => FilePlan {
            output: Some(conflict(&[], up)),
            next_base: Some(up.to_vec()),
            delete_installed: false,
            conflict: true,
        },
        (Some(loc), None) if loc == base => {
            FilePlan { output: None, next_base: None, delete_installed: true, conflict: false }
        }
        (Some(loc), None) if has_conflict_markers(loc) => FilePlan {
            output: Some(loc.to_vec()),
            next_base: Some(base.to_vec()),
            delete_installed: false,
            conflict: false,
        },
        (Some(loc), None) => FilePlan {
            output: Some(conflict(loc, &[])),
            next_base: Some(base.to_vec()),
            delete_installed: false,
            conflict: true,
        },
        (Some(loc), Some(up)) if loc == base => FilePlan {
            output: Some(up.to_vec()),
            next_base: Some(up.to_vec()),
            delete_installed: false,
            conflict: false,
        },
        (Some(loc), Some(up)) if up == base || loc == up => FilePlan {
            output: Some(loc.to_vec()),
            next_base: Some(up.to_vec()),
            delete_installed: false,
            conflict: false,
        },
        (Some(loc), Some(up)) => {
            let (merged, conflicts) = merge3(base, loc, up);
            FilePlan {
                output: Some(merged),
                next_base: Some(up.to_vec()),
                delete_installed: false,
                conflict: conflicts,
            }
        }
    }
}

fn has_conflict_markers(bytes: &[u8]) -> bool {
    let text = String::from_utf8_lossy(bytes);
    text.lines().any(|line| line.starts_with("<<<<<<< local"))
        && text.lines().any(|line| line == "=======")
        && text.lines().any(|line| line.starts_with(">>>>>>> upstream"))
}

fn conflict(local: &[u8], upstream: &[u8]) -> Vec<u8> {
    let mut out = b"<<<<<<< local\n".to_vec();
    out.extend_from_slice(local);
    ensure_newline(&mut out);
    out.extend_from_slice(b"=======\n");
    out.extend_from_slice(upstream);
    ensure_newline(&mut out);
    out.extend_from_slice(b">>>>>>> upstream\n");
    out
}
fn ensure_newline(v: &mut Vec<u8>) {
    if !v.is_empty() && !v.ends_with(b"\n") {
        v.push(b'\n');
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct Edit {
    start: usize,
    end: usize,
    replacement: Vec<Vec<u8>>,
}
fn lines(bytes: &[u8]) -> Vec<Vec<u8>> {
    if bytes.is_empty() {
        return vec![];
    }
    bytes.split_inclusive(|b| *b == b'\n').map(|s| s.to_vec()).collect()
}
fn edits(base: &[Vec<u8>], side: &[Vec<u8>]) -> Vec<Edit> {
    let (n, m) = (base.len(), side.len());
    let mut lcs = vec![vec![0usize; m + 1]; n + 1];
    for i in (0..n).rev() {
        for j in (0..m).rev() {
            lcs[i][j] = if base[i] == side[j] {
                1 + lcs[i + 1][j + 1]
            } else {
                lcs[i + 1][j].max(lcs[i][j + 1])
            };
        }
    }
    let (mut i, mut j) = (0, 0);
    let mut out = Vec::new();
    let mut active: Option<Edit> = None;
    while i < n || j < m {
        if i < n && j < m && base[i] == side[j] {
            if let Some(e) = active.take() {
                out.push(e);
            }
            i += 1;
            j += 1;
        } else if j < m && (i == n || lcs[i][j + 1] > lcs[i + 1][j]) {
            let e = active.get_or_insert(Edit { start: i, end: i, replacement: vec![] });
            e.replacement.push(side[j].clone());
            j += 1;
        } else {
            let e = active.get_or_insert(Edit { start: i, end: i, replacement: vec![] });
            i += 1;
            e.end = i;
        }
    }
    if let Some(e) = active {
        out.push(e);
    }
    out
}
fn merge3(base: &[u8], local: &[u8], upstream: &[u8]) -> (Vec<u8>, bool) {
    let b = lines(base);
    let le = edits(&b, &lines(local));
    let ue = edits(&b, &lines(upstream));
    let mut all: Vec<(Edit, bool)> =
        le.into_iter().map(|e| (e, true)).chain(ue.into_iter().map(|e| (e, false))).collect();
    all.sort_by_key(|(e, side)| (e.start, e.end, *side));
    let mut out = Vec::new();
    let mut pos = 0;
    let mut i = 0;
    let mut had_conflict = false;
    while i < all.len() {
        let first = all[i].0.clone();
        if first.start < pos {
            i += 1;
            continue;
        }
        out.extend(b[pos..first.start].iter().flatten());
        let mut group = vec![all[i].clone()];
        let mut end = first.end;
        i += 1;
        while i < all.len()
            && (all[i].0.start < end
                || (all[i].0.start == end && all[i].0.start == all[i].0.end && end == first.start))
        {
            end = end.max(all[i].0.end);
            group.push(all[i].clone());
            i += 1;
        }
        let locals = group.iter().filter(|x| x.1).collect::<Vec<_>>();
        let ups = group.iter().filter(|x| !x.1).collect::<Vec<_>>();
        let local_repl = apply_group(&b, first.start, end, &locals);
        let up_repl = apply_group(&b, first.start, end, &ups);
        if locals.is_empty() {
            out.extend(up_repl.iter().flatten());
        } else if ups.is_empty() || local_repl == up_repl {
            out.extend(local_repl.iter().flatten());
        } else {
            had_conflict = true;
            out.extend(conflict(&local_repl.concat(), &up_repl.concat()));
        }
        pos = end;
    }
    out.extend(b[pos..].iter().flatten());
    (out, had_conflict)
}
fn apply_group(base: &[Vec<u8>], start: usize, end: usize, es: &[&(Edit, bool)]) -> Vec<Vec<u8>> {
    if es.is_empty() {
        return base[start..end].to_vec();
    }
    let mut out = Vec::new();
    let mut cursor = start;
    for (e, _) in es {
        out.extend(base[cursor..e.start].iter().cloned());
        out.extend(e.replacement.clone());
        cursor = e.end;
    }
    out.extend(base[cursor..end].iter().cloned());
    out
}

fn infer_destination(component: &str, files: &BTreeMap<String, LockedFile>) -> Result<PathBuf> {
    if files.is_empty() {
        return Err(
            "cannot infer destination for an upstream-added file: component has no tracked files"
                .into(),
        );
    }
    let mut inferred: Option<PathBuf> = None;
    for (installed, locked) in files {
        let source_suffix =
            Path::new(&locked.base).strip_prefix(Path::new(".mkit/base").join(component))?;
        let installed_path = Path::new(installed);
        let candidates = [
            install_relative(component, source_suffix, true),
            install_relative(component, source_suffix, false),
        ];
        let tail = candidates
            .iter()
            .filter_map(|candidate| {
                strip_path_suffix(installed_path, candidate)
                    .ok()
                    .map(|prefix| (prefix, candidate.components().count()))
            })
            .max_by_key(|(_, length)| *length)
            .ok_or_else(|| format!("cannot infer destination from locked path `{installed}`"))?;
        if inferred.as_ref().is_some_and(|prefix| prefix != &tail.0) {
            return Err(
                "cannot infer one destination prefix from the component's locked files".into()
            );
        }
        inferred = Some(tail.0);
    }
    Ok(inferred.unwrap_or_default())
}

fn strip_path_suffix(path: &Path, suffix: &Path) -> Result<PathBuf> {
    let path_components = path.components().collect::<Vec<_>>();
    let suffix_components = suffix.components().collect::<Vec<_>>();
    if suffix_components.len() > path_components.len()
        || path_components[path_components.len() - suffix_components.len()..] != suffix_components
    {
        return Err("path does not end with expected suffix".into());
    }
    let mut prefix = PathBuf::new();
    for component in &path_components[..path_components.len() - suffix_components.len()] {
        prefix.push(component.as_os_str());
    }
    Ok(prefix)
}

fn safe_project_file(root: &Path, path: &Path, rel: &str) -> Result<PathBuf> {
    safe_relative(rel)?;
    safe_contained(root, path, "project")
}
fn safe_repo_file(root: &Path, path: &Path, rel: &str) -> Result<PathBuf> {
    safe_relative(rel)?;
    safe_contained(root, path, "repository")
}
fn read_optional(path: &Path) -> Result<Option<Vec<u8>>> {
    match fs::read(path) {
        Ok(bytes) => Ok(Some(bytes)),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(e.into()),
    }
}
fn safe_contained(root: &Path, path: &Path, label: &str) -> Result<PathBuf> {
    let mut p = path.to_path_buf();
    while !p.exists() {
        if !p.pop() {
            break;
        }
    }
    if p.exists() && !p.canonicalize()?.starts_with(root) {
        return Err(format!("{label} path escapes its root: `{}`", path.display()).into());
    }
    Ok(path.to_path_buf())
}
fn install_relative(component: &str, source_rel: &Path, single: bool) -> PathBuf {
    let leaf = source_rel.file_name().unwrap_or_default();
    let module = component.replace('-', "_");
    if single && leaf == "lib.rs" {
        PathBuf::from(format!("{module}.rs"))
    } else if single {
        PathBuf::from(leaf)
    } else {
        PathBuf::from(module).join(source_rel)
    }
}
fn commit(
    project: &Path,
    writes: BTreeMap<PathBuf, Vec<u8>>,
    deletes: BTreeSet<PathBuf>,
    doc: &mut DocumentMut,
    lock: &LockFile,
    lock_path: &Path,
    old_lock: &[u8],
) -> Result<()> {
    let mut backups: BTreeMap<PathBuf, Option<Vec<u8>>> = BTreeMap::new();
    for rel in writes.keys().chain(deletes.iter()) {
        safe_relative(rel.to_str().ok_or("invalid path")?)?;
        let path = project.join(rel);
        let root = project.canonicalize()?;
        let checked = safe_contained(&root, &path, "project")?;
        backups.insert(rel.clone(), if checked.exists() { Some(fs::read(checked)?) } else { None });
    }
    let apply = (|| -> Result<()> {
        for (rel, bytes) in &writes {
            let path = project.join(rel);
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent)?;
            }
            crate::support::atomic_write(&path, bytes)?;
        }
        for rel in &deletes {
            let path = project.join(rel);
            if path.exists() {
                fs::remove_file(path)?;
            }
        }
        write_lock(doc, lock, lock_path)?;
        Ok(())
    })();
    if let Err(err) = apply {
        for (rel, old) in backups {
            let path = project.join(rel);
            match old {
                Some(bytes) => {
                    let _ = crate::support::atomic_write(&path, &bytes);
                }
                None => {
                    let _ = fs::remove_file(path);
                }
            }
        }
        let _ = crate::support::atomic_write(lock_path, old_lock);
        return Err(err);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::add_components;
    use std::{
        env, process,
        time::{SystemTime, UNIX_EPOCH},
    };

    fn fixture() -> (PathBuf, PathBuf) {
        let root = env::temp_dir().join(format!(
            "mkit-update-{}-{}",
            process::id(),
            SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos()
        ));
        let project = root.join("project");
        let repo = root.join("repo");
        fs::create_dir_all(project.join("src/ui")).unwrap();
        fs::create_dir_all(repo.join("registry/widget/src")).unwrap();
        fs::write(
            project.join("Cargo.toml"),
            "[package]\nname='fixture'\nversion='0.1.0'\nedition='2024'\n",
        )
        .unwrap();
        fs::write(repo.join("registry/widget/src/lib.rs"), "one\ntwo\nthree\n").unwrap();
        fs::write(repo.join("registry/registry.json"), r#"{"registry_format_version":1,"release_version":"1","distribution":{"kind":"repository","path":"registry/"},"components":[{"name":"widget","version":"1.0.0","status":"source_ready","source_files":["registry/widget/src/lib.rs"],"dependencies":[],"component_dependencies":[]}] }"#).unwrap();
        add_components(&project, &repo, &["widget".into()], "src/ui").unwrap();
        (project, repo)
    }
    fn catalog(repo: &Path, version: &str, files: &[&str]) {
        let list =
            files.iter().map(|f| format!("\"registry/widget/{f}\"")).collect::<Vec<_>>().join(",");
        fs::write(repo.join("registry/registry.json"), format!(r#"{{"registry_format_version":1,"release_version":"{version}","distribution":{{"kind":"repository","path":"registry/"}},"components":[{{"name":"widget","version":"{version}","status":"source_ready","source_files":[{list}],"dependencies":[],"component_dependencies":[]}}]}}"#)).unwrap();
    }
    fn catalog_with_dependency(repo: &Path, version: &str, dependency_version: &str) {
        fs::write(repo.join("registry/registry.json"), format!(
            r#"{{"registry_format_version":1,"release_version":"{version}","distribution":{{"kind":"repository","path":"registry/"}},"components":[{{"name":"widget","version":"{version}","status":"source_ready","source_files":["registry/widget/src/lib.rs"],"dependencies":[{{"name":"mkit-core","version":"{dependency_version}"}}],"component_dependencies":[]}}]}}"#
        )).unwrap();
    }
    #[test]
    fn independent_line_edits_merge() {
        assert_eq!(merge3(b"a\nb\nc\n", b"A\nb\nc\n", b"a\nb\nC\n").0, b"A\nb\nC\n");
    }
    #[test]
    fn update_adds_new_dependencies_and_check_keeps_manifest_unchanged() {
        let (project, repo) = fixture();
        catalog_with_dependency(&repo, "1.1.0", "0.1.0");
        let before = fs::read(project.join("Cargo.toml")).unwrap();
        assert!(update_components(&project, &repo, None, true).unwrap().has_updates());
        assert_eq!(fs::read(project.join("Cargo.toml")).unwrap(), before);
        update_components(&project, &repo, None, false).unwrap();
        let manifest = fs::read_to_string(project.join("Cargo.toml")).unwrap();
        assert!(manifest.contains("mkit-core"));
        assert!(manifest.contains("=0.1.0"));
        fs::remove_dir_all(project.parent().unwrap()).unwrap();
    }

    #[test]
    fn update_refuses_dependency_version_change_without_touching_source_or_lock() {
        let (project, repo) = fixture();
        catalog_with_dependency(&repo, "1.1.0", "0.1.0");
        update_components(&project, &repo, None, false).unwrap();
        catalog_with_dependency(&repo, "1.2.0", "0.2.0");
        let before_source = fs::read(project.join("src/ui/widget.rs")).unwrap();
        let before_lock = fs::read(project.join("mkit.toml")).unwrap();
        let before_manifest = fs::read(project.join("Cargo.toml")).unwrap();
        let error = update_components(&project, &repo, None, false).unwrap_err();
        assert!(error.to_string().contains("registry requires `0.2.0`"));
        assert_eq!(fs::read(project.join("src/ui/widget.rs")).unwrap(), before_source);
        assert_eq!(fs::read(project.join("mkit.toml")).unwrap(), before_lock);
        assert_eq!(fs::read(project.join("Cargo.toml")).unwrap(), before_manifest);
        fs::remove_dir_all(project.parent().unwrap()).unwrap();
    }
    #[test]
    fn overlapping_line_edits_get_markers() {
        let (out, conflicted) = merge3(b"a\nb\n", b"a\nlocal\n", b"a\nupstream\n");
        assert!(conflicted);
        assert!(
            String::from_utf8(out)
                .unwrap()
                .contains("<<<<<<< local\nlocal\n=======\nupstream\n>>>>>>> upstream")
        );
    }

    #[test]
    fn existing_marker_text_is_not_counted_as_a_new_conflict() {
        let local = b"<<<<<<< local\nuser text\n=======\nother text\n>>>>>>> upstream\n";
        let plan = plan_file(Some(local), b"base\n", Some(b"base\n"));
        assert!(!plan.conflict);
    }

    #[test]
    fn conflict_keeps_unchanged_local_lines_between_edits() {
        let (out, conflicted) = merge3(b"a\nb\nc\nd\n", b"A\nb\nC\nd\n", b"U\nV\nW\nd\n");
        assert!(conflicted);
        assert!(
            String::from_utf8(out)
                .unwrap()
                .contains("<<<<<<< local\nA\nb\nC\n=======\nU\nV\nW\n>>>>>>> upstream")
        );
    }

    #[test]
    fn upstream_added_file_keeps_the_locked_destination_prefix() {
        let (project, repo) = fixture();
        fs::write(repo.join("registry/widget/src/extra.rs"), "pub fn extra() {}\n").unwrap();
        catalog(&repo, "1.1.0", &["src/lib.rs", "src/extra.rs"]);
        let report = update_components(&project, &repo, None, false).unwrap();
        assert_eq!(report.components[0].changed_files, 1);
        assert_eq!(
            fs::read(project.join("src/ui/widget/src/extra.rs")).unwrap(),
            b"pub fn extra() {}\n"
        );
        assert_eq!(
            fs::read(project.join(".mkit/base/widget/src/extra.rs")).unwrap(),
            b"pub fn extra() {}\n"
        );
        let lock: LockFile =
            toml_edit::de::from_str(&fs::read_to_string(project.join("mkit.toml")).unwrap())
                .unwrap();
        assert!(lock.components[0].files.contains_key("src/ui/widget/src/extra.rs"));
        fs::remove_dir_all(project.parent().unwrap()).unwrap();
    }

    #[test]
    fn upstream_deletion_cleans_base_and_keeps_edited_file_tracked_on_conflict() {
        let (project, repo) = fixture();
        fs::remove_file(repo.join("registry/widget/src/lib.rs")).unwrap();
        fs::write(project.join("src/ui/widget.rs"), "local edit\n").unwrap();
        catalog(&repo, "1.1.0", &["src/lib.rs"]);
        let report = update_components(&project, &repo, None, false).unwrap();
        assert_eq!(report.components[0].conflicts, 1);
        assert!(project.join(".mkit/base/widget/src/lib.rs").exists());
        let lock: LockFile =
            toml_edit::de::from_str(&fs::read_to_string(project.join("mkit.toml")).unwrap())
                .unwrap();
        assert!(lock.components[0].files.contains_key("src/ui/widget.rs"));
        assert!(!update_components(&project, &repo, None, false).unwrap().has_updates());
        fs::remove_dir_all(project.parent().unwrap()).unwrap();

        let (project, repo) = fixture();
        fs::remove_file(repo.join("registry/widget/src/lib.rs")).unwrap();
        catalog(&repo, "1.1.0", &["src/lib.rs"]);
        update_components(&project, &repo, None, false).unwrap();
        assert!(!project.join(".mkit/base/widget/src/lib.rs").exists());
        let lock: LockFile =
            toml_edit::de::from_str(&fs::read_to_string(project.join("mkit.toml")).unwrap())
                .unwrap();
        assert!(lock.components[0].files.is_empty());
        fs::remove_dir_all(project.parent().unwrap()).unwrap();
    }

    #[test]
    fn check_mode_reports_without_writing_and_normal_update_is_repeatable() {
        let (project, repo) = fixture();
        fs::write(project.join("src/ui/widget.rs"), "ONE\ntwo\nthree\n").unwrap();
        fs::write(repo.join("registry/widget/src/lib.rs"), "one\ntwo\nTHREE\n").unwrap();
        catalog(&repo, "1.1.0", &["src/lib.rs"]);
        let before_file = fs::read(project.join("src/ui/widget.rs")).unwrap();
        let before_base = fs::read(project.join(".mkit/base/widget/src/lib.rs")).unwrap();
        let before_lock = fs::read(project.join("mkit.toml")).unwrap();
        let check = update_components(&project, &repo, None, true).unwrap();
        assert!(check.has_updates());
        assert_eq!(fs::read(project.join("src/ui/widget.rs")).unwrap(), before_file);
        assert_eq!(fs::read(project.join(".mkit/base/widget/src/lib.rs")).unwrap(), before_base);
        assert_eq!(fs::read(project.join("mkit.toml")).unwrap(), before_lock);
        let updated = update_components(&project, &repo, None, false).unwrap();
        assert_eq!(updated.components[0].conflicts, 0);
        assert_eq!(fs::read(project.join("src/ui/widget.rs")).unwrap(), b"ONE\ntwo\nTHREE\n");
        assert_eq!(
            fs::read(project.join(".mkit/base/widget/src/lib.rs")).unwrap(),
            b"one\ntwo\nTHREE\n"
        );
        let lock: LockFile =
            toml_edit::de::from_str(&fs::read_to_string(project.join("mkit.toml")).unwrap())
                .unwrap();
        assert_eq!(lock.components[0].version, "1.1.0");
        assert_eq!(
            lock.components[0].files["src/ui/widget.rs"].sha256,
            sha256(b"one\ntwo\nTHREE\n")
        );
        assert!(!update_components(&project, &repo, None, false).unwrap().has_updates());
        assert_eq!(
            fs::read(project.join(".mkit/base/widget/src/lib.rs")).unwrap(),
            b"one\ntwo\nTHREE\n"
        );
        fs::remove_dir_all(project.parent().unwrap()).unwrap();
    }

    #[test]
    fn same_line_edits_are_conflicted_and_base_advances_to_upstream() {
        let (project, repo) = fixture();
        fs::write(project.join("src/ui/widget.rs"), "one\nlocal\nthree\n").unwrap();
        fs::write(repo.join("registry/widget/src/lib.rs"), "one\nupstream\nthree\n").unwrap();
        catalog(&repo, "1.1.0", &["src/lib.rs"]);
        let report = update_components(&project, &repo, Some("widget"), false).unwrap();
        assert_eq!(report.components[0].conflicts, 1);
        let output = fs::read_to_string(project.join("src/ui/widget.rs")).unwrap();
        assert!(output.contains("<<<<<<< local\nlocal\n=======\nupstream\n>>>>>>> upstream"));
        assert_eq!(
            fs::read(project.join(".mkit/base/widget/src/lib.rs")).unwrap(),
            b"one\nupstream\nthree\n"
        );
        fs::remove_dir_all(project.parent().unwrap()).unwrap();
    }

    #[test]
    fn baseline_hash_mismatch_refuses_update() {
        let (project, repo) = fixture();
        fs::write(project.join(".mkit/base/widget/src/lib.rs"), "tampered\n").unwrap();
        assert!(
            update_components(&project, &repo, None, false)
                .unwrap_err()
                .to_string()
                .contains("hash mismatch")
        );
        fs::remove_dir_all(project.parent().unwrap()).unwrap();
    }
}
