use crate::cli::find_repository_root;
use crate::installer::insert_dependency;
use crate::lock::LockFile;
use crate::{add_components, diff_components};
use std::process::Command;
use std::{
    env, fs,
    path::{Path, PathBuf},
    process,
    time::{SystemTime, UNIX_EPOCH},
};
use toml_edit::DocumentMut;
fn temp_dir(label: &str) -> PathBuf {
    let p = env::temp_dir().join(format!(
        "cargo-mkit-{label}-{}-{}",
        process::id(),
        SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos()
    ));
    fs::create_dir_all(&p).unwrap();
    p
}
fn crate_stub(path: &Path, name: &str, version: &str) {
    fs::create_dir_all(path.join("src")).unwrap();
    fs::write(path.join("Cargo.toml"), format!("[package]\nname = \"{name}\"\nversion = \"{version}\"\nedition = \"2024\"\n[lib]\npath = \"src/lib.rs\"\n")).unwrap();
    fs::write(path.join("src/lib.rs"), "pub fn ready() {}\n").unwrap();
}
#[test]
fn add_copies_transitive_components_lock_and_compiles_project() {
    let root = temp_dir("repo");
    let project = temp_dir("project");
    fs::create_dir_all(root.join("registry/alpha/src")).unwrap();
    fs::create_dir_all(root.join("registry/beta/src")).unwrap();
    fs::write(root.join("registry/alpha/src/lib.rs"), "pub fn alpha() { super::beta::beta(); }\n")
        .unwrap();
    fs::write(root.join("registry/beta/src/lib.rs"), "pub fn beta() {}\n").unwrap();
    fs::create_dir_all(root.join("registry/scrubbable-number-field/src")).unwrap();
    fs::write(root.join("registry/scrubbable-number-field/src/lib.rs"), "pub fn component() {}\n")
        .unwrap();
    let registry = r#"{"registry_format_version":1,"release_version":"1.0.0","distribution":{"kind":"repository","path":"registry/","published":false},"components":[{"name":"alpha","version":"1.2.0","status":"source_ready","source_files":["registry/alpha/src/lib.rs"],"dependencies":[{"name":"mkit-core","version":"0.1.0"},{"name":"gpui-pre","version":"0.3.5"}],"component_dependencies":["beta"]},{"name":"beta","version":"1.0.0","status":"source_ready","source_files":["registry/beta/src/lib.rs"],"dependencies":[],"component_dependencies":[]},{"name":"scrubbable-number-field","version":"2.0.0","status":"source_ready","source_files":["registry/scrubbable-number-field/src/lib.rs"],"dependencies":[],"component_dependencies":[]}] }"#;
    fs::write(root.join("registry/registry.json"), registry).unwrap();
    fs::write(project.join("Cargo.toml"), "[package]\nname = \"fixture\"\nversion = \"0.1.0\"\nedition = \"2024\"\n[patch.crates-io]\nmkit-core = { path = \"stubs/mkit-core\" }\ngpui-pre = { path = \"stubs/gpui-pre\" }\n").unwrap();
    crate_stub(&project.join("stubs/mkit-core"), "mkit-core", "0.1.0");
    crate_stub(&project.join("stubs/gpui-pre"), "gpui-pre", "0.3.5");
    add_components(&project, &root, &["alpha".to_owned()], "src/ui").unwrap();
    assert!(project.join("src/ui/alpha.rs").exists());
    assert!(project.join("src/ui/beta.rs").exists());
    add_components(&project, &root, &["scrubbable-number-field".to_owned()], "src/ui").unwrap();
    let lock_text = fs::read_to_string(project.join("mkit.toml")).unwrap();
    let parsed: LockFile = toml_edit::de::from_str(&lock_text).unwrap();
    assert_eq!(parsed.components.len(), 3);
    assert!(parsed.components.iter().any(|c| c.name == "alpha" && c.version == "1.2.0"));
    assert!(parsed.components.iter().any(|c| {
        c.name == "scrubbable-number-field"
            && c.files.values().any(|f| {
                f.base == ".mkit/base/scrubbable-number-field/src/lib.rs" && !f.sha256.is_empty()
            })
    }));
    let manifest = fs::read_to_string(project.join("Cargo.toml")).unwrap();
    assert!(manifest.contains("gpui_pre"));
    assert!(manifest.contains("mkit-core"));
    fs::write(
            project.join("src/main.rs"),
            "mod ui { pub mod alpha; pub mod beta; pub mod scrubbable_number_field; }\nfn main() { ui::alpha::alpha(); ui::scrubbable_number_field::component(); }\n",
        )
        .unwrap();
    let output = Command::new("cargo")
        .args(["check", "--offline", "--quiet"])
        .current_dir(&project)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "cargo check failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let _ = fs::remove_dir_all(root);
    let _ = fs::remove_dir_all(project);
}
#[test]
fn in_progress_is_refused_without_writing() {
    let root = temp_dir("status");
    let project = temp_dir("refuse");
    fs::create_dir_all(root.join("registry/x/src")).unwrap();
    fs::write(root.join("registry/x/src/lib.rs"), "").unwrap();
    fs::write(root.join("registry/registry.json"), r#"{"registry_format_version":1,"release_version":"1","distribution":{"kind":"repository","path":"registry/","published":false},"components":[{"name":"x","version":"1","status":"implementation_in_progress","source_files":["registry/x/src/lib.rs"]}]}"#).unwrap();
    fs::write(
        project.join("Cargo.toml"),
        "[package]\nname=\"x\"\nversion=\"0.1.0\"\nedition=\"2024\"\n",
    )
    .unwrap();
    assert!(add_components(&project, &root, &["x".to_owned()], "src/ui").is_err());
    assert!(!project.join("src/ui").exists());
    let _ = fs::remove_dir_all(root);
    let _ = fs::remove_dir_all(project);
}
#[test]
fn conflicting_existing_dependency_is_refused() {
    let mut manifest: DocumentMut = "[dependencies]\nmkit-core = \"=0.2.0\"\n".parse().unwrap();
    let error = insert_dependency(&mut manifest, "mkit-core", "0.1.0").unwrap_err();
    assert!(error.to_string().contains("requires `0.1.0`"));
}
#[test]
fn gpui_alias_must_point_to_the_expected_package() {
    let mut manifest: DocumentMut =
        "[dependencies]\ngpui_pre = { version = \"=0.3.5\", package = \"different-crate\" }\n"
            .parse()
            .unwrap();
    assert!(
        insert_dependency(&mut manifest, "gpui-pre", "0.3.5")
            .unwrap_err()
            .to_string()
            .contains("must refer to package `gpui-pre`")
    );
}
#[test]
fn repository_search_uses_current_ancestors() {
    let root = temp_dir("search");
    let nested = root.join("a/b");
    fs::create_dir_all(&nested).unwrap();
    fs::create_dir_all(root.join("registry")).unwrap();
    fs::write(root.join("registry/registry.json"), "{}").unwrap();
    assert_eq!(find_repository_root(&nested), Some(root.clone()));
    let _ = fs::remove_dir_all(root);
}
#[test]
fn diff_compares_recorded_base_to_local_and_upstream_without_mutation() {
    let root = temp_dir("diff-repo");
    let project = temp_dir("diff-project");
    for name in ["panel", "steady"] {
        fs::create_dir_all(root.join(format!("registry/{name}/src"))).unwrap();
        fs::write(
            root.join(format!("registry/{name}/src/lib.rs")),
            format!("pub fn {name}() {{}}\n"),
        )
        .unwrap();
    }
    fs::write(root.join("registry/registry.json"), r#"{"registry_format_version":1,"release_version":"1.0.0","distribution":{"kind":"repository","path":"registry/","published":false},"components":[{"name":"panel","version":"1.0.0","status":"source_ready","source_files":["registry/panel/src/lib.rs"],"dependencies":[],"component_dependencies":[]},{"name":"steady","version":"1.0.0","status":"source_ready","source_files":["registry/steady/src/lib.rs"],"dependencies":[],"component_dependencies":[]}] }"#).unwrap();
    fs::write(
        project.join("Cargo.toml"),
        "[package]\nname=\"diff-fixture\"\nversion=\"0.1.0\"\nedition=\"2024\"\n",
    )
    .unwrap();
    add_components(&project, &root, &["panel".into(), "steady".into()], "src/ui").unwrap();
    fs::write(project.join("src/ui/panel.rs"), "pub fn panel() { /* local */ }\n").unwrap();
    fs::write(root.join("registry/panel/src/lib.rs"), "pub fn panel() { /* upstream */ }\n")
        .unwrap();
    fs::write(root.join("registry/panel/src/extra.rs"), "pub fn added_upstream() {}\n").unwrap();
    fs::write(root.join("registry/registry.json"), r#"{"registry_format_version":1,"release_version":"1.1.0","distribution":{"kind":"repository","path":"registry/","published":false},"components":[{"name":"panel","version":"1.1.0","status":"source_ready","source_files":["registry/panel/src/lib.rs","registry/panel/src/extra.rs"],"dependencies":[],"component_dependencies":[]},{"name":"steady","version":"1.0.0","status":"source_ready","source_files":["registry/steady/src/lib.rs"],"dependencies":[],"component_dependencies":[]}] }"#).unwrap();
    let before_manifest = fs::read(project.join("Cargo.toml")).unwrap();
    let before_lock = fs::read(project.join("mkit.toml")).unwrap();
    let report = diff_components(&project, &root, None).unwrap();
    assert!(report.contains("recorded base → local"));
    assert!(report.contains("recorded base → upstream"));
    assert!(report.contains("Component panel (recorded 1.0.0, current 1.1.0)"));
    assert!(report.contains("File registry/panel/src/extra.rs (upstream added)"));
    assert!(report.contains("+pub fn added_upstream() {}"));
    assert!(
        report.contains("-pub fn panel() {}") && report.contains("+pub fn panel() { /* local */ }")
    );
    assert!(report.contains("+pub fn panel() { /* upstream */ }"));
    assert!(report.contains("local: unchanged from recorded base"));
    assert!(report.contains("upstream: unchanged from recorded base"));
    assert_eq!(fs::read(project.join("Cargo.toml")).unwrap(), before_manifest);
    assert_eq!(fs::read(project.join("mkit.toml")).unwrap(), before_lock);
    assert!(
        diff_components(&project, &root, Some("absent"))
            .unwrap_err()
            .to_string()
            .contains("not recorded")
    );
    fs::remove_file(project.join("src/ui/steady.rs")).unwrap();
    fs::remove_file(root.join("registry/steady/src/lib.rs")).unwrap();
    let deleted = diff_components(&project, &root, Some("steady")).unwrap();
    assert!(deleted.contains("local: file deleted"));
    assert!(deleted.contains("upstream: source deleted"));
    let _ = fs::remove_dir_all(root);
    let _ = fs::remove_dir_all(project);
}
