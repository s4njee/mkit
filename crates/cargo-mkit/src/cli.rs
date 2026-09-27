use crate::{
    Result, add_components, diff_components,
    doctor::{DoctorRequirements, IssueLevel, doctor},
    lock::LockFile,
    registry::Registry,
    update_components,
};
use std::{
    env, fs,
    path::{Path, PathBuf},
    process,
};
pub fn cli_main() {
    if let Err(e) = run() {
        eprintln!("cargo-mkit: {e}");
        process::exit(2);
    }
}
pub(crate) fn find_repository_root(start: &Path) -> Option<PathBuf> {
    start
        .ancestors()
        .find(|path| path.join("registry/registry.json").is_file())
        .map(Path::to_path_buf)
}

fn run() -> Result<()> {
    let mut args = env::args().skip(1).collect::<Vec<_>>();
    if args.first().is_some_and(|s| s == "mkit") {
        args.remove(0);
    }
    let command = args.first().cloned().ok_or("usage: cargo mkit <add|diff|update|doctor> ...")?;
    args.remove(0);
    let project = env::current_dir()?;
    let mut repo = None;
    match command.as_str() {
        "add" => {
            let mut dest = "src/ui".to_owned();
            let mut names = Vec::new();
            let mut i = 0;
            while i < args.len() {
                match args[i].as_str() {
                    "--path" | "--registry" => {
                        i += 1;
                        repo = Some(PathBuf::from(args.get(i).ok_or("missing path argument")?));
                    }
                    "--dest" => {
                        i += 1;
                        dest = args.get(i).ok_or("missing destination")?.clone();
                    }
                    s if s.starts_with('-') => return Err(format!("unknown option `{s}`").into()),
                    s => names.push(s.to_owned()),
                }
                i += 1;
            }
            if names.is_empty() {
                return Err("provide at least one component name".into());
            }
            let repo = resolve_repo(repo, &project)?;
            add_components(&project, &repo, &names, &dest)
        }
        "diff" => {
            let mut filter = None;
            let mut i = 0;
            while i < args.len() {
                match args[i].as_str() {
                    "--path" | "--registry" => {
                        i += 1;
                        repo = Some(PathBuf::from(args.get(i).ok_or("missing path argument")?));
                    }
                    s if s.starts_with('-') => return Err(format!("unknown option `{s}`").into()),
                    s if filter.is_none() => filter = Some(s.to_owned()),
                    s => return Err(format!("unexpected argument `{s}`").into()),
                }
                i += 1;
            }
            let repo = resolve_repo(repo, &project)?;
            print!("{}", diff_components(&project, &repo, filter.as_deref())?);
            Ok(())
        }
        "update" => {
            let mut filter = None;
            let mut check = false;
            let mut i = 0;
            while i < args.len() {
                match args[i].as_str() {
                    "--path" | "--registry" => {
                        i += 1;
                        repo = Some(PathBuf::from(args.get(i).ok_or("missing path argument")?));
                    }
                    "--check" => check = true,
                    s if s.starts_with('-') => return Err(format!("unknown option `{s}`").into()),
                    s if filter.is_none() => filter = Some(s.to_owned()),
                    s => return Err(format!("unexpected argument `{s}`").into()),
                }
                i += 1;
            }
            let repo = resolve_repo(repo, &project)?;
            let report = update_components(&project, &repo, filter.as_deref(), check)?;
            if !report.has_updates() {
                println!("all selected components are current");
                return Ok(());
            }
            for component in &report.components {
                println!(
                    "{}: {} → {} ({} changed file(s), {} conflict(s))",
                    component.name,
                    component.from,
                    component.to,
                    component.changed_files,
                    component.conflicts
                );
            }
            if check {
                return Err("component updates are available".into());
            }
            let conflicts: usize =
                report.components.iter().map(|component| component.conflicts).sum();
            if conflicts > 0 {
                return Err(format!(
                    "update wrote {conflicts} merge conflict(s); resolve the markers in your source"
                )
                .into());
            }
            Ok(())
        }
        "doctor" => {
            let mut i = 0;
            while i < args.len() {
                match args[i].as_str() {
                    "--path" | "--registry" => {
                        i += 1;
                        repo = Some(PathBuf::from(args.get(i).ok_or("missing path argument")?));
                    }
                    s => return Err(format!("unexpected doctor argument `{s}`").into()),
                }
                i += 1;
            }
            let repository = repo.or_else(|| find_repository_root(&project));
            let requirements = confirmed_requirements(&project, repository.as_deref());
            let report = doctor(&project, &requirements);
            for issue in &report.issues {
                let level = match issue.level {
                    IssueLevel::Info => "info",
                    IssueLevel::Warning => "warning",
                };
                println!("{level} [{}]: {}", issue.code, issue.message);
            }
            if report.warnings() > 0 {
                return Err(format!("doctor found {} warning(s)", report.warnings()).into());
            }
            Ok(())
        }
        _ => Err(format!(
            "unknown command `{command}`; expected `add`, `diff`, `update`, or `doctor`"
        )
        .into()),
    }
}
fn resolve_repo(repo: Option<PathBuf>, project: &Path) -> Result<PathBuf> {
    match repo { Some(path) => Ok(path), None => find_repository_root(project).ok_or_else(|| "could not find registry/registry.json in the current directory or its parents; pass --path <repository-root>".into()) }
}

fn confirmed_requirements(project: &Path, repo: Option<&Path>) -> DoctorRequirements {
    let Some(repo) = repo else { return DoctorRequirements::default() };
    let Ok(lock_text) = fs::read_to_string(project.join("mkit.toml")) else {
        return DoctorRequirements::default();
    };
    let Ok(lock) = toml_edit::de::from_str::<LockFile>(&lock_text) else {
        return DoctorRequirements::default();
    };
    let Ok(registry) = Registry::load(repo) else { return DoctorRequirements::default() };
    if lock.components.is_empty() {
        return DoctorRequirements::default();
    }
    let mut gpui_pre: Option<String> = None;
    let mut mkit_core: Option<String> = None;
    for installed in &lock.components {
        let Some(current) = registry
            .components
            .iter()
            .find(|entry| entry.name == installed.name && entry.version == installed.version)
        else {
            return DoctorRequirements::default();
        };
        for (name, target) in [("gpui-pre", &mut gpui_pre), ("mkit-core", &mut mkit_core)] {
            let Some(required) = current.dependencies.iter().find(|dep| dep.name == name) else {
                return DoctorRequirements::default();
            };
            if target.as_ref().is_some_and(|previous| previous != &required.version) {
                return DoctorRequirements::default();
            }
            *target = Some(required.version.clone());
        }
    }
    DoctorRequirements { gpui_pre, mkit_core }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn doctor_uses_only_requirements_matching_the_recorded_version() {
        let path = env::temp_dir().join(format!(
            "cargo-mkit-doctor-cli-{}-{}",
            process::id(),
            SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos()
        ));
        let project = path.join("project");
        let repo = path.join("repo");
        fs::create_dir_all(repo.join("registry")).unwrap();
        fs::create_dir_all(&project).unwrap();
        fs::write(
            repo.join("registry/registry.json"),
            r#"{"registry_format_version":1,"release_version":"1.0.0","distribution":{"kind":"repository","path":"registry/","published":false},"components":[{"name":"field","version":"1.0.0","status":"source_ready","source_files":["registry/field/src/lib.rs"],"dependencies":[{"name":"gpui-pre","version":"0.3.5"},{"name":"mkit-core","version":"0.1.0"}],"component_dependencies":[]}]}"#,
        )
        .unwrap();
        fs::write(
            project.join("mkit.toml"),
            "[[components]]\nname = 'field'\nversion = '1.0.0'\norigin = 'registry/field'\n",
        )
        .unwrap();
        let matching = confirmed_requirements(&project, Some(&repo));
        assert_eq!(matching.gpui_pre.as_deref(), Some("0.3.5"));
        assert_eq!(matching.mkit_core.as_deref(), Some("0.1.0"));
        fs::write(
            project.join("mkit.toml"),
            "[[components]]\nname = 'field'\nversion = '0.9.0'\norigin = 'registry/field'\n",
        )
        .unwrap();
        assert_eq!(confirmed_requirements(&project, Some(&repo)), DoctorRequirements::default());
        fs::remove_dir_all(path).unwrap();
    }
}
