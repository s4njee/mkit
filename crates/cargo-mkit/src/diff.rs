use crate::{
    Result,
    lock::{LockFile, safe_relative, sha256},
    registry::Registry,
};
use std::{collections::BTreeSet, fs, path::Path};

/// Compare installed files to their recorded bases and the matching repository sources.
pub fn diff_components(project: &Path, repo: &Path, filter: Option<&str>) -> Result<String> {
    let lock_path = project.join("mkit.toml");
    if !lock_path.is_file() {
        return Err("mkit.toml not found; no installed components are recorded".into());
    }
    let lock: LockFile = toml_edit::de::from_str(&fs::read_to_string(lock_path)?)?;
    let selected: Vec<_> =
        lock.components.iter().filter(|c| filter.is_none_or(|n| c.name == n)).collect();
    if selected.is_empty() {
        return Err(match filter {
            Some(name) => format!("component `{name}` is not recorded in mkit.toml").into(),
            None => "mkit.toml contains no recorded components".into(),
        });
    }
    let registry = Registry::load(repo)?;
    let project_root = project.canonicalize()?;
    let repo_root = repo.canonicalize()?;
    let mut output = String::new();
    for component in selected {
        let current = registry.components.iter().find(|entry| entry.name == component.name);
        let current_version = current.map(|entry| entry.version.as_str()).unwrap_or("not present");
        output.push_str(&format!(
            "Component {} (recorded {}, current {})\n",
            component.name, component.version, current_version
        ));
        safe_relative(&component.origin)?;
        if component.origin != format!("registry/{}", component.name) {
            return Err(format!(
                "unexpected recorded origin `{}` for component `{}`",
                component.origin, component.name
            )
            .into());
        }
        if component.files.is_empty() {
            output.push_str("  no tracked source files\n");
        }
        let mut recorded_sources = BTreeSet::new();
        for (installed_rel, record) in &component.files {
            safe_relative(installed_rel)?;
            safe_relative(&record.base)?;
            let expected_base = Path::new(".mkit/base").join(&component.name);
            if !Path::new(&record.base).starts_with(&expected_base) {
                return Err(format!("recorded base for `{installed_rel}` is outside this component's .mkit/base snapshot").into());
            }
            let base = project.join(&record.base);
            let installed = project.join(installed_rel);
            let base_bytes = read_contained_file(&project_root, &base, "recorded base")?;
            if sha256(&base_bytes) != record.sha256 {
                return Err(format!("recorded base hash mismatch for `{}`", record.base).into());
            }
            let source_suffix = Path::new(&record.base).strip_prefix(&expected_base)?;
            let upstream_rel = Path::new(&component.origin).join(source_suffix);
            safe_relative(upstream_rel.to_str().ok_or("invalid upstream path")?)?;
            recorded_sources.insert(upstream_rel.to_string_lossy().to_string());
            let local_bytes =
                read_optional_contained_file(&project_root, &installed, "installed file")?;
            let upstream_bytes = read_optional_contained_file(
                &repo_root,
                &repo.join(&upstream_rel),
                "upstream source",
            )?;
            output.push_str(&format!("  File {installed_rel}\n"));
            match local_bytes {
                Some(bytes) => {
                    match unified_diff(&record.base, &base_bytes, installed_rel, &bytes, "local") {
                        Some(diff) => output.push_str(&diff),
                        None => output.push_str("    local: unchanged from recorded base\n"),
                    }
                }
                None => output.push_str("    local: file deleted\n"),
            }
            match upstream_bytes {
                Some(bytes) => match unified_diff(
                    &record.base,
                    &base_bytes,
                    &upstream_rel.to_string_lossy(),
                    &bytes,
                    "upstream",
                ) {
                    Some(diff) => output.push_str(&diff),
                    None => output.push_str("    upstream: unchanged from recorded base\n"),
                },
                None => output.push_str("    upstream: source deleted\n"),
            }
        }
        if let Some(current) = current {
            for source in &current.source_files {
                safe_relative(source)?;
                if recorded_sources.contains(source) {
                    continue;
                }
                let source_path = Path::new(source);
                if !source_path.starts_with(Path::new(&component.origin)) {
                    return Err(format!(
                        "current source `{source}` is outside component origin `{}`",
                        component.origin
                    )
                    .into());
                }
                let Some(bytes) = read_optional_contained_file(
                    &repo_root,
                    &repo.join(source),
                    "upstream source",
                )?
                else {
                    output.push_str(&format!(
                        "  File {source}\n    upstream: catalog source is missing\n"
                    ));
                    continue;
                };
                output.push_str(&format!("  File {source} (upstream added)\n"));
                output.push_str(
                    &unified_diff("/dev/null", b"", source, &bytes, "upstream")
                        .expect("added files differ from an empty base"),
                );
            }
        }
    }
    Ok(output)
}

fn read_contained_file(root: &Path, path: &Path, label: &str) -> Result<Vec<u8>> {
    read_optional_contained_file(root, path, label)?
        .ok_or_else(|| format!("{label} `{}` is missing", path.display()).into())
}
fn read_optional_contained_file(root: &Path, path: &Path, label: &str) -> Result<Option<Vec<u8>>> {
    if !path.exists() {
        return Ok(None);
    }
    let resolved = path.canonicalize()?;
    if !resolved.starts_with(root) {
        return Err(format!("{label} path escapes its root: `{}`", path.display()).into());
    }
    Ok(Some(fs::read(resolved)?))
}

/// Render an all-context unified diff; `None` means the bytes are unchanged.
fn unified_diff(
    old_path: &str,
    old: &[u8],
    new_path: &str,
    new: &[u8],
    side: &str,
) -> Option<String> {
    if old == new {
        return None;
    }
    let (Ok(old_text), Ok(new_text)) = (std::str::from_utf8(old), std::str::from_utf8(new)) else {
        return Some(format!(
            "    {side}: binary content changed ({} bytes → {} bytes)\n",
            old.len(),
            new.len()
        ));
    };
    let old_lines: Vec<&str> = old_text.split_terminator('\n').collect();
    let new_lines: Vec<&str> = new_text.split_terminator('\n').collect();
    let old_final_nl = old_text.is_empty() || old_text.ends_with('\n');
    let new_final_nl = new_text.is_empty() || new_text.ends_with('\n');
    let newline_only = old_lines == new_lines && old_final_nl != new_final_nl;
    // Keep source positions alongside the rendered rows. Text alone is not a
    // sufficient identity here: repeated final lines must not inherit an EOF
    // marker merely because their contents happen to match the last line.
    let mut rows: Vec<(char, &str, Option<usize>, Option<usize>)> = Vec::new();
    if newline_only {
        rows.extend(
            old_lines.iter().enumerate().map(|(index, line)| ('-', *line, Some(index), None)),
        );
        rows.extend(
            new_lines.iter().enumerate().map(|(index, line)| ('+', *line, None, Some(index))),
        );
    } else {
        let mut lcs = vec![vec![0usize; new_lines.len() + 1]; old_lines.len() + 1];
        for i in (0..old_lines.len()).rev() {
            for j in (0..new_lines.len()).rev() {
                lcs[i][j] = if old_lines[i] == new_lines[j] {
                    1 + lcs[i + 1][j + 1]
                } else {
                    lcs[i + 1][j].max(lcs[i][j + 1])
                };
            }
        }
        let (mut i, mut j) = (0, 0);
        while i < old_lines.len() || j < new_lines.len() {
            if i < old_lines.len() && j < new_lines.len() && old_lines[i] == new_lines[j] {
                rows.push((' ', old_lines[i], Some(i), Some(j)));
                i += 1;
                j += 1;
            } else if i < old_lines.len()
                && (j == new_lines.len() || lcs[i + 1][j] >= lcs[i][j + 1])
            {
                rows.push(('-', old_lines[i], Some(i), None));
                i += 1;
            } else {
                rows.push(('+', new_lines[j], None, Some(j)));
                j += 1;
            }
        }
    }
    let old_start = if old_lines.is_empty() { 0 } else { 1 };
    let new_start = if new_lines.is_empty() { 0 } else { 1 };
    let mut out = format!(
        "    {side}: recorded base → {side}\n--- a/{old_path}\n+++ b/{new_path}\n@@ -{old_start},{} +{new_start},{} @@\n",
        old_lines.len(),
        new_lines.len()
    );
    for (prefix, line, old_index, new_index) in rows {
        out.push(prefix);
        out.push_str(line);
        out.push('\n');
        let is_old_final_without_newline =
            old_index == old_lines.len().checked_sub(1) && !old_final_nl;
        let is_new_final_without_newline =
            new_index == new_lines.len().checked_sub(1) && !new_final_nl;
        if is_old_final_without_newline || is_new_final_without_newline {
            out.push_str("\\ No newline at end of file\n");
        }
    }
    Some(out)
}

#[cfg(test)]
mod tests {
    use super::unified_diff;

    #[test]
    fn newline_only_changes_emit_a_valid_replacement_and_marker() {
        let diff =
            unified_diff("base.rs", b"pub fn x() {}", "local.rs", b"pub fn x() {}\n", "local")
                .unwrap();
        assert!(diff.contains("@@ -1,1 +1,1 @@"));
        assert!(diff.contains("-pub fn x() {}\n\\ No newline at end of file"));
        assert!(diff.contains("+pub fn x() {}"));
    }

    #[test]
    fn eof_markers_follow_line_positions_not_duplicate_text() {
        // The first `same` line is context. Only the second occurrence is the
        // unterminated old final line and should receive the marker.
        let diff =
            unified_diff("base", b"same\nsame", "local", b"same\nchanged\n", "local").unwrap();
        assert!(diff.contains(" same\n-same\n\\ No newline at end of file\n+changed\n"));
        assert!(!diff.contains(" same\n\\ No newline at end of file"));
    }

    #[test]
    fn eof_marker_is_emitted_for_an_unterminated_shared_final_line() {
        let diff = unified_diff("base", b"first\nlast", "local", b"first\nlast\nadded\n", "local")
            .unwrap();
        assert!(diff.contains(" last\n\\ No newline at end of file\n+added\n"));
    }
}
