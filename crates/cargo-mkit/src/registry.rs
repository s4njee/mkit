use crate::{Result, lock::safe_relative};
use serde::Deserialize;
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::Path,
};

#[derive(Debug, Deserialize)]
pub(crate) struct Registry {
    pub(crate) registry_format_version: u32,
    pub(crate) release_version: String,
    pub(crate) distribution: Distribution,
    pub(crate) components: Vec<ComponentEntry>,
}
#[derive(Debug, Deserialize)]
pub(crate) struct Distribution {
    pub(crate) kind: String,
    pub(crate) path: String,
}
#[derive(Clone, Debug, Deserialize)]
pub(crate) struct ComponentEntry {
    pub(crate) name: String,
    pub(crate) version: String,
    pub(crate) status: String,
    pub(crate) source_files: Vec<String>,
    #[serde(default)]
    pub(crate) dependencies: Vec<CrateDependency>,
    #[serde(default)]
    pub(crate) component_dependencies: Vec<ComponentDependency>,
}
#[derive(Clone, Debug, Deserialize)]
pub(crate) struct CrateDependency {
    pub(crate) name: String,
    pub(crate) version: String,
}
#[derive(Clone, Debug, Deserialize)]
pub(crate) struct ComponentDependency(pub String);
#[derive(Clone, Debug)]
pub(crate) struct SourceSnapshot {
    pub(crate) component: String,
    pub(crate) version: String,
    pub(crate) origin: String,
    pub(crate) files: BTreeMap<String, Vec<u8>>,
}

impl Registry {
    pub(crate) fn load(root: &Path) -> Result<Self> {
        let path = root.join("registry/registry.json");
        let reg: Registry = serde_json::from_slice(&fs::read(path)?)?;
        if reg.registry_format_version != 1
            || reg.distribution.kind != "repository"
            || reg.distribution.path != "registry/"
            || reg.release_version.is_empty()
        {
            return Err("unsupported registry format or distribution".into());
        }
        let mut names = BTreeSet::new();
        for component in &reg.components {
            if !names.insert(component.name.as_str()) {
                return Err(format!("duplicate component `{}` in registry", component.name).into());
            }
            let mut dependencies = BTreeSet::new();
            for dependency in &component.component_dependencies {
                if !dependencies.insert(dependency.0.as_str()) {
                    return Err(format!(
                        "component `{}` has duplicate component dependency `{}`",
                        component.name, dependency.0
                    )
                    .into());
                }
            }
        }
        Ok(reg)
    }
    pub(crate) fn resolve(&self, root: &Path, requested: &str) -> Result<Vec<SourceSnapshot>> {
        let mut out = Vec::new();
        let mut seen = BTreeSet::new();
        let mut visiting = BTreeSet::new();
        self.resolve_one(root, requested, &mut seen, &mut visiting, &mut out)?;
        Ok(out)
    }
    fn resolve_one(
        &self,
        root: &Path,
        name: &str,
        seen: &mut BTreeSet<String>,
        visiting: &mut BTreeSet<String>,
        out: &mut Vec<SourceSnapshot>,
    ) -> Result<()> {
        if visiting.contains(name) {
            return Err(format!("component dependency cycle involving `{name}`").into());
        }
        if seen.contains(name) {
            return Ok(());
        }
        let entry = self
            .components
            .iter()
            .find(|c| c.name == name)
            .ok_or_else(|| format!("component `{name}` is not in the registry"))?;
        if entry.status != "source_ready" {
            return Err(format!(
                "component `{name}` is not installable (status: {})",
                entry.status
            )
            .into());
        }
        visiting.insert(name.to_owned());
        for dep in &entry.component_dependencies {
            let dep_name = &dep.0;
            if !self.components.iter().any(|c| &c.name == dep_name) {
                return Err(format!("unknown component dependency `{dep_name}`").into());
            }
            self.resolve_one(root, dep_name, seen, visiting, out)?;
        }
        if entry.source_files.is_empty() {
            return Err(format!("component `{name}` has no source files").into());
        }
        let mut files = BTreeMap::new();
        let mut source_files = BTreeSet::new();
        for rel in &entry.source_files {
            safe_relative(rel)?;
            if !source_files.insert(rel.as_str()) {
                return Err(
                    format!("component `{name}` lists source `{rel}` more than once").into()
                );
            }
            let full = root.join(rel);
            let canonical_root = root.canonicalize()?;
            if !full.canonicalize()?.starts_with(&canonical_root) {
                return Err(format!("source path escapes repository: `{rel}`").into());
            }
            let bytes = fs::read(full)?;
            let prefix = format!("registry/{name}/");
            let suffix = rel
                .strip_prefix(&prefix)
                .ok_or_else(|| format!("source `{rel}` is outside its component directory"))?;
            safe_relative(suffix)?;
            files.insert(suffix.to_string(), bytes);
        }
        out.push(SourceSnapshot {
            component: name.to_owned(),
            version: entry.version.clone(),
            origin: format!("registry/{name}"),
            files,
        });
        visiting.remove(name);
        seen.insert(name.to_owned());
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::Registry;
    use std::{env, fs, path::PathBuf, process, time::SystemTime};

    fn fixture(registry: &str) -> PathBuf {
        let root = env::temp_dir().join(format!(
            "cargo-mkit-registry-{}-{}",
            process::id(),
            SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_nanos()
        ));
        fs::create_dir_all(root.join("registry/a/src")).unwrap();
        fs::create_dir_all(root.join("registry/b/src")).unwrap();
        fs::write(root.join("registry/a/src/lib.rs"), "").unwrap();
        fs::write(root.join("registry/b/src/lib.rs"), "").unwrap();
        fs::write(root.join("registry/registry.json"), registry).unwrap();
        root
    }

    const HEADER: &str = r#"{"registry_format_version":1,"release_version":"1.0.0","distribution":{"kind":"repository","path":"registry/"},"components":["#;
    const A: &str = r#"{"name":"a","version":"1.0.0","status":"source_ready","source_files":["registry/a/src/lib.rs"],"component_dependencies":[]}"#;
    const B: &str = r#"{"name":"b","version":"1.0.0","status":"source_ready","source_files":["registry/b/src/lib.rs"],"component_dependencies":[]}"#;

    #[test]
    fn load_rejects_duplicate_component_names() {
        let root = fixture(&format!("{HEADER}{A},{A}]}}"));
        let error = Registry::load(&root).unwrap_err().to_string();
        assert!(error.contains("duplicate component `a`"));
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn resolve_rejects_component_dependency_cycles() {
        let a = r#"{"name":"a","version":"1.0.0","status":"source_ready","source_files":["registry/a/src/lib.rs"],"component_dependencies":["b"]}"#;
        let b = r#"{"name":"b","version":"1.0.0","status":"source_ready","source_files":["registry/b/src/lib.rs"],"component_dependencies":["a"]}"#;
        let root = fixture(&format!("{HEADER}{a},{b}]}}"));
        let registry = Registry::load(&root).unwrap();
        let error = registry.resolve(&root, "a").unwrap_err().to_string();
        assert!(error.contains("component dependency cycle"));
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn load_rejects_duplicate_component_dependencies() {
        let a = r#"{"name":"a","version":"1.0.0","status":"source_ready","source_files":["registry/a/src/lib.rs"],"component_dependencies":["b","b"]}"#;
        let root = fixture(&format!("{HEADER}{a},{B}]}}"));
        let error = Registry::load(&root).unwrap_err().to_string();
        assert!(error.contains("duplicate component dependency `b`"));
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn resolve_rejects_duplicate_source_paths() {
        let a = r#"{"name":"a","version":"1.0.0","status":"source_ready","source_files":["registry/a/src/lib.rs","registry/a/src/lib.rs"],"component_dependencies":[]}"#;
        let root = fixture(&format!("{HEADER}{a}]}}"));
        let registry = Registry::load(&root).unwrap();
        let error = registry.resolve(&root, "a").unwrap_err().to_string();
        assert!(error.contains("lists source `registry/a/src/lib.rs` more than once"));
        let _ = fs::remove_dir_all(root);
    }
}
