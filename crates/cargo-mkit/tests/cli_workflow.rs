use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Output},
    sync::atomic::{AtomicU64, Ordering},
};

static NEXT: AtomicU64 = AtomicU64::new(0);

struct Fixture(PathBuf);

impl Fixture {
    fn new() -> Self {
        let id = NEXT.fetch_add(1, Ordering::Relaxed);
        let root = std::env::temp_dir()
            .join(format!("cargo-mkit-cli-workflow-{}-{id}", std::process::id()));
        fs::create_dir_all(&root).unwrap();
        Self(root)
    }

    fn repo(&self) -> PathBuf {
        self.0.join("repo")
    }

    fn project(&self) -> PathBuf {
        self.0.join("project")
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn write_registry(repo: &Path, version: &str, source_files: &[&str], status: &str) {
    let component_files = source_files
        .iter()
        .map(|source| {
            let relative = source.strip_prefix("registry/panel/").unwrap();
            let path = repo.join(source);
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent).unwrap();
            }
            if !path.exists() {
                fs::write(path, format!("pub fn {}() {{}}\n", relative.replace(['/', '.'], "_")))
                    .unwrap();
            }
            format!("\"{source}\"")
        })
        .collect::<Vec<_>>()
        .join(",");
    let registry = format!(
        r#"{{"registry_format_version":1,"release_version":"{version}","distribution":{{"kind":"repository","path":"registry/","published":false}},"components":[{{"name":"panel","version":"{version}","status":"{status}","source_files":[{component_files}],"dependencies":[{{"name":"gpui-pre","version":"0.3.5"}},{{"name":"mkit-core","version":"0.1.0"}}],"component_dependencies":[]}},{{"name":"draft","version":"0.1.0","status":"implementation_in_progress","source_files":["registry/draft/src/lib.rs"],"dependencies":[],"component_dependencies":[]}}]}}"#
    );
    fs::create_dir_all(repo.join("registry")).unwrap();
    fs::write(repo.join("registry/registry.json"), registry).unwrap();
    let draft = repo.join("registry/draft/src/lib.rs");
    fs::create_dir_all(draft.parent().unwrap()).unwrap();
    fs::write(draft, "pub fn draft() {}\n").unwrap();
}

fn run(project: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_cargo-mkit")).args(args).current_dir(project).output().unwrap()
}

fn text(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).into_owned()
}

#[test]
fn cli_add_diff_update_and_doctor_work_without_network_access() {
    let fixture = Fixture::new();
    let repo = fixture.repo();
    let project = fixture.project();
    fs::create_dir_all(&project).unwrap();
    fs::write(
        project.join("Cargo.toml"),
        "[package]\nname = \"fixture-app\"\nversion = \"0.1.0\"\nedition = \"2024\"\n",
    )
    .unwrap();
    write_registry(&repo, "1.0.0", &["registry/panel/src/lib.rs"], "source_ready");

    let added = run(&project, &["add", "panel", "--path", repo.to_str().unwrap()]);
    assert!(added.status.success(), "add failed: {}", text(&added.stderr));
    assert!(project.join("src/ui/panel.rs").is_file());
    assert!(project.join(".mkit/base/panel/src/lib.rs").is_file());
    let lock_before = fs::read(project.join("mkit.toml")).unwrap();
    let manifest_before = fs::read(project.join("Cargo.toml")).unwrap();
    let source_before = fs::read(project.join("src/ui/panel.rs")).unwrap();
    let base_before = fs::read(project.join(".mkit/base/panel/src/lib.rs")).unwrap();

    fs::write(repo.join("registry/panel/src/lib.rs"), "pub fn panel() { /* upstream */ }\n")
        .unwrap();
    fs::write(repo.join("registry/panel/src/extra.rs"), "pub fn extra() {}\n").unwrap();
    write_registry(
        &repo,
        "1.1.0",
        &["registry/panel/src/lib.rs", "registry/panel/src/extra.rs"],
        "source_ready",
    );

    let diff = run(&project, &["diff", "--path", repo.to_str().unwrap()]);
    assert!(diff.status.success(), "diff failed: {}", text(&diff.stderr));
    assert!(text(&diff.stdout).contains("Component panel (recorded 1.0.0, current 1.1.0)"));
    assert!(text(&diff.stdout).contains("upstream added"));
    assert_eq!(fs::read(project.join("mkit.toml")).unwrap(), lock_before);
    assert_eq!(fs::read(project.join("Cargo.toml")).unwrap(), manifest_before);
    assert_eq!(fs::read(project.join("src/ui/panel.rs")).unwrap(), source_before);
    assert_eq!(fs::read(project.join(".mkit/base/panel/src/lib.rs")).unwrap(), base_before);
    assert!(!project.join("src/ui/panel/src/extra.rs").exists());

    let check = run(&project, &["update", "--check", "--path", repo.to_str().unwrap()]);
    assert!(!check.status.success(), "update --check should report available updates");
    assert!(text(&check.stdout).contains("panel: 1.0.0 → 1.1.0"));
    assert!(text(&check.stderr).contains("component updates are available"));
    assert_eq!(fs::read(project.join("mkit.toml")).unwrap(), lock_before);
    assert_eq!(fs::read(project.join("Cargo.toml")).unwrap(), manifest_before);
    assert_eq!(fs::read(project.join("src/ui/panel.rs")).unwrap(), source_before);
    assert_eq!(fs::read(project.join(".mkit/base/panel/src/lib.rs")).unwrap(), base_before);
    assert!(!project.join("src/ui/panel/src/extra.rs").exists());

    let update = run(&project, &["update", "--path", repo.to_str().unwrap()]);
    assert!(update.status.success(), "update failed: {}", text(&update.stderr));
    assert!(project.join("src/ui/panel/src/extra.rs").is_file());
    assert!(text(&fs::read(project.join("src/ui/panel.rs")).unwrap()).contains("upstream"));
    assert!(text(&fs::read(project.join("mkit.toml")).unwrap()).contains("version = \"1.1.0\""));

    let doctor = run(&project, &["doctor", "--path", repo.to_str().unwrap()]);
    assert!(doctor.status.success(), "doctor failed: {}", text(&doctor.stderr));
    assert!(text(&doctor.stdout).contains("info [gpui-kit-absent]"));
    assert!(!text(&doctor.stdout).contains("warning ["));

    let manifest_path = project.join("Cargo.toml");
    let mut manifest = fs::read_to_string(&manifest_path).unwrap();
    manifest = manifest.replace("version = \"=0.1.0\"", "version = \"=0.2.0\"");
    fs::write(&manifest_path, manifest).unwrap();
    let doctor_warning = run(&project, &["doctor", "--path", repo.to_str().unwrap()]);
    assert!(
        !doctor_warning.status.success(),
        "doctor should fail when a required version mismatches"
    );
    assert!(text(&doctor_warning.stdout).contains("warning [dependency-version-mismatch]"));
    assert!(text(&doctor_warning.stderr).contains("doctor found 1 warning(s)"));

    let lock_before_draft = fs::read(project.join("mkit.toml")).unwrap();
    let draft = run(&project, &["add", "draft", "--path", repo.to_str().unwrap()]);
    assert!(!draft.status.success(), "draft component should be refused");
    assert!(text(&draft.stderr).contains("not installable"));
    assert_eq!(fs::read(project.join("mkit.toml")).unwrap(), lock_before_draft);
    assert!(!project.join("src/ui/draft.rs").exists());
}

#[test]
fn cli_update_reports_conflicts_and_advances_the_upstream_snapshot() {
    let fixture = Fixture::new();
    let repo = fixture.repo();
    let project = fixture.project();
    fs::create_dir_all(&project).unwrap();
    fs::write(
        project.join("Cargo.toml"),
        "[package]\nname = \"conflict-fixture\"\nversion = \"0.1.0\"\nedition = \"2024\"\n",
    )
    .unwrap();
    write_registry(&repo, "1.0.0", &["registry/panel/src/lib.rs"], "source_ready");

    let added = run(&project, &["add", "panel", "--path", repo.to_str().unwrap()]);
    assert!(added.status.success(), "add failed: {}", text(&added.stderr));
    fs::write(project.join("src/ui/panel.rs"), "pub fn panel() { /* local edit */ }\n").unwrap();
    let upstream = "pub fn panel() { /* upstream edit */ }\n";
    fs::write(repo.join("registry/panel/src/lib.rs"), upstream).unwrap();
    write_registry(&repo, "1.1.0", &["registry/panel/src/lib.rs"], "source_ready");

    let update = run(&project, &["update", "--path", repo.to_str().unwrap()]);
    assert!(!update.status.success(), "conflicting update should return a nonzero exit status");
    assert!(text(&update.stdout).contains("panel: 1.0.0 → 1.1.0"));
    assert!(text(&update.stderr).contains("resolve the markers"));
    let installed = fs::read_to_string(project.join("src/ui/panel.rs")).unwrap();
    assert!(installed.contains("<<<<<<< local\n"));
    assert!(installed.contains("=======\n"));
    assert!(installed.contains(">>>>>>> upstream\n"));
    let base = fs::read_to_string(project.join(".mkit/base/panel/src/lib.rs")).unwrap();
    assert_eq!(base, upstream);
    let lock = fs::read_to_string(project.join("mkit.toml")).unwrap();
    assert!(lock.contains("version = \"1.1.0\""));
}
