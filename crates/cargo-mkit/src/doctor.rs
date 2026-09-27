//! Conservative project setup checks for `cargo mkit doctor`.
//!
//! Requirements come from the registry entries matching the components recorded in
//! `mkit.toml`. The lock file does not currently persist dependency requirements, so
//! callers must not derive them from a stale component catalog.

use std::fs;
use std::path::Path;
use toml_edit::{DocumentMut, Item, TableLike};

/// Registry requirements applicable to the installed components in a project.
///
/// A `None` value means the caller could not establish a trustworthy requirement
/// (for example, because an installed component's catalog version is stale).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct DoctorRequirements {
    /// Required `gpui-pre` version or Cargo version requirement.
    pub gpui_pre: Option<String>,
    /// Required `mkit-core` version or Cargo version requirement.
    pub mkit_core: Option<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IssueLevel {
    Info,
    Warning,
}

/// One human-readable doctor finding, suitable for formatting by the CLI.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DoctorIssue {
    pub level: IssueLevel,
    pub code: &'static str,
    pub message: String,
}

/// Structured result of checking a project.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct DoctorReport {
    pub issues: Vec<DoctorIssue>,
}

impl DoctorReport {
    pub fn warnings(&self) -> usize {
        self.issues.iter().filter(|i| i.level == IssueLevel::Warning).count()
    }
}

/// Inspect a project's Cargo manifest, optional `mkit.toml`, and workspace dependency
/// declarations. `requirements` should contain only requirements confirmed against the
/// installed component versions in the registry.
pub fn doctor(project: &Path, requirements: &DoctorRequirements) -> DoctorReport {
    let mut report = DoctorReport::default();
    let manifest_path = project.join("Cargo.toml");
    let manifest = match fs::read_to_string(&manifest_path)
        .map_err(|e| e.to_string())
        .and_then(|s| s.parse::<DocumentMut>().map_err(|e| e.to_string()))
    {
        Ok(doc) => doc,
        Err(error) => {
            report.push(
                IssueLevel::Warning,
                "manifest-unreadable",
                format!("could not read or parse {}: {error}", manifest_path.display()),
            );
            return report;
        }
    };

    let workspace = workspace_manifest(project, &manifest);
    let deps = collect_dependencies(&manifest, workspace.as_ref());
    check_required_dependency(&mut report, &deps, "gpui-pre", requirements.gpui_pre.as_deref());
    check_required_dependency(&mut report, &deps, "mkit-core", requirements.mkit_core.as_deref());
    check_gpui_kit(&mut report, &deps);

    let lock_path = project.join("mkit.toml");
    match fs::read_to_string(&lock_path) {
        Ok(text) => match text.parse::<DocumentMut>() {
            Ok(doc) => {
                let component_count = doc["components"].as_array_of_tables().map_or(0, |a| a.len());
                if component_count > 0
                    && (requirements.gpui_pre.is_none() || requirements.mkit_core.is_none())
                {
                    report.push(IssueLevel::Warning, "component-requirements-unknown", format!(
                        "{} installed component(s) are recorded in mkit.toml, but one or more required versions could not be confirmed from the current registry; compatibility is unknown",
                        component_count
                    ));
                }
            }
            Err(error) => report.push(
                IssueLevel::Warning,
                "mkit-lock-unreadable",
                format!("could not parse {}: {error}", lock_path.display()),
            ),
        },
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            report.push(
                IssueLevel::Info,
                "no-mkit-lock",
                "mkit.toml is absent; no installed component requirements were checked".into(),
            );
        }
        Err(error) => report.push(
            IssueLevel::Warning,
            "mkit-lock-unreadable",
            format!("could not read {}: {error}", lock_path.display()),
        ),
    }
    report
}

impl DoctorReport {
    fn push(&mut self, level: IssueLevel, code: &'static str, message: String) {
        self.issues.push(DoctorIssue { level, code, message });
    }
}

#[derive(Clone, Debug)]
struct Dependency {
    package: String,
    alias: String,
    version: Option<String>,
}

fn collect_dependencies(
    manifest: &DocumentMut,
    workspace: Option<&DocumentMut>,
) -> Vec<Dependency> {
    let mut result = Vec::new();
    for section in ["dependencies", "dev-dependencies", "build-dependencies"] {
        if let Some(table) = manifest.get(section).and_then(Item::as_table_like) {
            collect_from_table(table, workspace, &mut result);
        }
    }
    if let Some(targets) = manifest.get("target").and_then(Item::as_table_like) {
        for (_, target) in targets.iter() {
            for section in ["dependencies", "dev-dependencies", "build-dependencies"] {
                if let Some(table) = target.get(section).and_then(Item::as_table_like) {
                    collect_from_table(table, workspace, &mut result);
                }
            }
        }
    }
    result
}

fn collect_from_table(
    table: &dyn TableLike,
    workspace: Option<&DocumentMut>,
    out: &mut Vec<Dependency>,
) {
    for (alias, item) in table.iter() {
        let (package, version, inherits_workspace) = match item.as_table_like() {
            Some(spec) => (
                spec.get("package").and_then(Item::as_str).unwrap_or(alias).to_owned(),
                spec.get("version").and_then(Item::as_str).map(str::to_owned),
                spec.get("workspace").and_then(Item::as_bool) == Some(true),
            ),
            None => (alias.to_owned(), item.as_str().map(str::to_owned), false),
        };
        let inherited = if inherits_workspace {
            workspace
                .and_then(|doc| doc.get("workspace"))
                .and_then(|w| w.get("dependencies"))
                .and_then(Item::as_table_like)
                .and_then(|t| t.get(alias))
                .map(dependency_spec)
        } else {
            None
        };
        let (inherited_package, inherited_version) = inherited
            .map(|(package, version)| ((!package.is_empty()).then_some(package), version))
            .unwrap_or((None, None));
        out.push(Dependency {
            package: if package == alias { inherited_package.unwrap_or(package) } else { package },
            alias: alias.to_owned(),
            version: version.or(inherited_version),
        });
    }
}

fn dependency_spec(item: &Item) -> (String, Option<String>) {
    match item.as_table_like() {
        Some(table) => (
            table.get("package").and_then(Item::as_str).unwrap_or("").to_owned(),
            table.get("version").and_then(Item::as_str).map(str::to_owned),
        ),
        None => (String::new(), item.as_str().map(str::to_owned)),
    }
}

fn workspace_manifest(project: &Path, manifest: &DocumentMut) -> Option<DocumentMut> {
    if manifest.get("workspace").is_some() {
        return Some(manifest.clone());
    }
    for ancestor in project.ancestors().skip(1) {
        let path = ancestor.join("Cargo.toml");
        let Ok(text) = fs::read_to_string(path) else {
            continue;
        };
        let Ok(doc) = text.parse::<DocumentMut>() else {
            continue;
        };
        if doc.get("workspace").is_some() {
            return Some(doc);
        }
    }
    None
}

fn check_required_dependency(
    report: &mut DoctorReport,
    deps: &[Dependency],
    package: &str,
    required: Option<&str>,
) {
    let found: Vec<_> = deps.iter().filter(|d| d.package == package).collect();
    if found.is_empty() {
        if let Some(version) = required {
            report.push(
                IssueLevel::Warning,
                "required-dependency-missing",
                format!("required dependency `{package}` ({version}) is missing from Cargo.toml"),
            );
        }
        // A project without known installed component requirements may not need
        // this dependency at all. The lock-file check reports unknown requirements
        // when it finds recorded components.
        return;
    }
    if found.iter().any(|d| {
        d.version
            .as_deref()
            .zip(required)
            .is_some_and(|(have, need)| requirement_accepts(have, need))
    }) {
        return;
    }
    match required {
        None => report.push(IssueLevel::Warning, "required-version-unknown", format!(
            "`{package}` is declared, but its required version could not be confirmed from the registry"
        )),
        Some(need) => {
            let declared = found.iter().map(|d| format!("{}={}", d.alias, d.version.as_deref().unwrap_or("<inherited version unavailable>"))).collect::<Vec<_>>().join(", ");
            report.push(IssueLevel::Warning, "dependency-version-mismatch", format!(
                "`{package}` requires a declaration compatible with `{need}`; found {declared}"
            ));
        }
    }
}

fn check_gpui_kit(report: &mut DoctorReport, deps: &[Dependency]) {
    let kits: Vec<_> = deps.iter().filter(|d| d.package == "gpui-kit").collect();
    if kits.is_empty() {
        report.push(
            IssueLevel::Info,
            "gpui-kit-absent",
            "gpui-kit is not declared; coexistence is not applicable".into(),
        );
        return;
    }
    let kit_versions = kits.iter().map(|d| d.version.as_deref()).collect::<Vec<_>>();
    let gpui_versions = deps
        .iter()
        .filter(|d| d.package == "gpui-pre")
        .map(|d| d.version.as_deref())
        .collect::<Vec<_>>();
    let known_pair = kit_versions.iter().any(|v| v.is_some_and(|s| version_equal(s, "0.6.4")))
        && gpui_versions.iter().any(|v| v.is_some_and(|s| requirement_accepts(s, "0.3.5")));
    if known_pair {
        report.push(IssueLevel::Info, "gpui-kit-known-pair", "gpui-kit 0.6.4 with gpui-pre 0.3.5 matches the repository's recorded coexistence baseline".into());
    } else {
        let kit = kits
            .iter()
            .map(|d| {
                format!("{}={}", d.alias, d.version.as_deref().unwrap_or("<version unavailable>"))
            })
            .collect::<Vec<_>>()
            .join(", ");
        let gpui = if gpui_versions.is_empty() {
            "gpui-pre not declared".to_owned()
        } else {
            gpui_versions
                .iter()
                .map(|v| v.unwrap_or("<version unavailable>"))
                .collect::<Vec<_>>()
                .join(", ")
        };
        report.push(IssueLevel::Warning, "gpui-kit-coexistence-unknown", format!(
            "GPUI Kit coexistence is unverified for {kit} with {gpui}; the recorded baseline is gpui-kit 0.6.4 with gpui-pre 0.3.5"
        ));
    }
}

fn requirement_accepts(declaration: &str, required: &str) -> bool {
    // Cargo version requirements are comma-separated intersections. Support the common
    // exact/caret/tilde/bound forms; unknown syntax deliberately fails closed.
    declaration.split(',').all(|part| constraint_accepts(part.trim(), required))
}

fn constraint_accepts(constraint: &str, required: &str) -> bool {
    if constraint == "*" {
        return true;
    }
    let (op, value) = [">=", "<=", ">", "<", "=", "^", "~"]
        .into_iter()
        .find_map(|op| constraint.strip_prefix(op).map(|v| (op, v.trim())))
        .unwrap_or(("^", constraint));
    let Some(want) = Version::parse(required) else {
        return false;
    };
    let Some(have) = Version::parse(value) else {
        return false;
    };
    match op {
        "=" => have == want,
        ">" => want > have,
        ">=" => want >= have,
        "<" => want < have,
        "<=" => want <= have,
        "~" => want >= have && want.major == have.major && want.minor == have.minor,
        "^" => {
            if have.major > 0 {
                want >= have && want.major == have.major
            } else if have.minor > 0 {
                want >= have && want.major == 0 && want.minor == have.minor
            } else {
                want >= have && want.major == 0 && want.minor == 0 && want.patch == have.patch
            }
        }
        _ => false,
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Version {
    major: u64,
    minor: u64,
    patch: u64,
}

impl Version {
    fn parse(s: &str) -> Option<Self> {
        let s = s.trim().trim_start_matches('v');
        if s.chars().any(|c| c == '-' || c == '+') {
            return None;
        }
        let mut pieces = s.split('.');
        let major = pieces.next()?.parse().ok()?;
        let minor = pieces.next().unwrap_or("0").parse().ok()?;
        let patch = pieces.next().unwrap_or("0").parse().ok()?;
        if pieces.next().is_some() {
            return None;
        }
        Some(Self { major, minor, patch })
    }
}

fn version_equal(declaration: &str, exact: &str) -> bool {
    let value = declaration.trim().strip_prefix('=').unwrap_or(declaration.trim()).trim();
    Version::parse(value) == Version::parse(exact)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicU64, Ordering};

    static NEXT: AtomicU64 = AtomicU64::new(0);
    struct Project(PathBuf);
    impl Project {
        fn new(manifest: &str, lock: Option<&str>) -> Self {
            let id = NEXT.fetch_add(1, Ordering::Relaxed);
            let path =
                std::env::temp_dir().join(format!("cargo-mkit-doctor-{}-{id}", std::process::id()));
            fs::create_dir_all(&path).unwrap();
            fs::write(path.join("Cargo.toml"), manifest).unwrap();
            if let Some(lock) = lock {
                fs::write(path.join("mkit.toml"), lock).unwrap();
            }
            Self(path)
        }
    }
    impl Drop for Project {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn validates_package_identity_and_required_versions() {
        let project = Project::new(
            "[dependencies]\ngpui_pre = { package = \"gpui-pre\", version = \"=0.3.5\" }\nmkit-core = \"0.1.0\"\n",
            Some("[[components]]\nname = \"button\"\nversion = \"1.0.0\"\n"),
        );
        let report = doctor(
            &project.0,
            &DoctorRequirements { gpui_pre: Some("0.3.5".into()), mkit_core: Some("0.1.0".into()) },
        );
        assert_eq!(report.warnings(), 0, "{:#?}", report.issues);
    }

    #[test]
    fn flags_wrong_alias_package_and_mismatched_version() {
        let project =
            Project::new("[dependencies]\ngpui_pre = \"0.3.4\"\nmkit-core = \"0.2.0\"\n", None);
        let report = doctor(
            &project.0,
            &DoctorRequirements { gpui_pre: Some("0.3.5".into()), mkit_core: Some("0.1.0".into()) },
        );
        assert!(
            report
                .issues
                .iter()
                .any(|i| i.code == "required-dependency-missing" && i.message.contains("gpui-pre"))
        );
        assert!(report.issues.iter().any(|i| i.code == "dependency-version-mismatch" && i.message.contains("mkit-core")));
    }

    #[test]
    fn recognizes_known_kit_baseline_and_unknown_pair() {
        let known = Project::new(
            "[dependencies]\ngpui_pre = { package = \"gpui-pre\", version = \"=0.3.5\" }\ngpui-kit = \"=0.6.4\"\n",
            None,
        );
        let report = doctor(&known.0, &DoctorRequirements::default());
        assert!(report.issues.iter().any(|i| i.code == "gpui-kit-known-pair"));
        let unknown = Project::new("[dependencies]\ngpui-kit = \"0.7\"\n", None);
        let report = doctor(&unknown.0, &DoctorRequirements::default());
        assert!(report.issues.iter().any(|i| i.code == "gpui-kit-coexistence-unknown"));
    }

    #[test]
    fn reports_unknown_component_requirements_when_catalog_input_missing() {
        let project = Project::new("[dependencies]\n", Some("[[components]]\nname=\"x\"\n"));
        let report = doctor(&project.0, &DoctorRequirements::default());
        assert!(report.issues.iter().any(|i| i.code == "component-requirements-unknown"));
    }

    #[test]
    fn does_not_warn_for_an_unrelated_project_without_mkit_lock() {
        let project = Project::new("[package]\nname=\"unrelated\"\nversion=\"0.1.0\"\n", None);
        let report = doctor(&project.0, &DoctorRequirements::default());
        assert_eq!(report.warnings(), 0, "{:#?}", report.issues);
        assert!(report.issues.iter().any(|issue| issue.code == "no-mkit-lock"));
    }

    #[test]
    fn resolves_inherited_workspace_dependency_version() {
        let parent = Project::new(
            "[workspace]\nmembers = [\"app\"]\n[workspace.dependencies]\ngpui_pre = { package = \"gpui-pre\", version = \"=0.3.5\" }\nmkit-core = \"=0.1.0\"\n",
            None,
        );
        let app = parent.0.join("app");
        fs::create_dir_all(&app).unwrap();
        fs::write(app.join("Cargo.toml"), "[package]\nname=\"app\"\nversion=\"0.1.0\"\n[dependencies]\ngpui_pre.workspace=true\nmkit-core.workspace=true\n").unwrap();
        let report = doctor(
            &app,
            &DoctorRequirements { gpui_pre: Some("0.3.5".into()), mkit_core: Some("0.1.0".into()) },
        );
        assert!(
            !report.issues.iter().any(|i| i.code == "dependency-version-mismatch"
                || i.code == "required-dependency-missing")
        );
    }
}
