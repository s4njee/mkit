import contextlib
import io
import tempfile
import unittest
from pathlib import Path
from unittest import mock

from check_snapshot_versions import (
    ISSUE_MARKER,
    ROOT,
    collect_pins,
    find_updates,
    latest_version,
    main,
    parse_pins,
    parse_version,
    render_issue,
    requirement_version,
)

WORKSPACE_TOML = """
[workspace]
members = ["apps/*"]

[workspace.dependencies]
gpui_pre = { package = "gpui-pre", version = "=0.3.5"}
serde = "1"
"""

APP_TOML = """
[package]
name = "app"

[dependencies]
gpui_pre.workspace = true
gpui_platform = { package = "gpui-pre-platform", version = "=0.3.5" }
gpui-kit = "=0.6.4"

[dev-dependencies]
gpui-kit = { version = "=0.6.4", features = ["test-support"] }

[target.'cfg(target_os = "macos")'.dependencies]
gpui-kit = "=0.6.2"
"""


def api(*versions, max_stable=None):
    return {
        "crate": {"max_stable_version": max_stable, "max_version": max_stable},
        "versions": [{"num": num, "yanked": yanked} for num, yanked in versions],
    }


class PinParsingTests(unittest.TestCase):
    def test_workspace_rename_resolves_package_name(self):
        pins = parse_pins(WORKSPACE_TOML, "Cargo.toml")
        self.assertEqual(len(pins), 1)
        self.assertEqual(pins[0]["crate"], "gpui-pre")
        self.assertEqual(pins[0]["version"], "0.3.5")
        self.assertEqual(pins[0]["requirement"], "=0.3.5")
        self.assertEqual(pins[0]["table"], "workspace.dependencies")

    def test_member_pins_skip_workspace_inheritance(self):
        pins = parse_pins(APP_TOML, "apps/app/Cargo.toml")
        found = sorted((p["crate"], p["version"], p["table"]) for p in pins)
        self.assertEqual(found, [
            ("gpui-kit", "0.6.2", 'target.cfg(target_os = "macos").dependencies'),
            ("gpui-kit", "0.6.4", "dependencies"),
            ("gpui-kit", "0.6.4", "dev-dependencies"),
            ("gpui-pre-platform", "0.3.5", "dependencies"),
        ])

    def test_requirement_forms(self):
        self.assertEqual(requirement_version("=0.3.5"), "0.3.5")
        self.assertEqual(requirement_version("^0.6"), "0.6.0")
        self.assertEqual(requirement_version("0.4.0-pre.1"), "0.4.0-pre.1")

    def test_repository_pins_are_found(self):
        crates = {p["crate"] for p in collect_pins(ROOT)}
        self.assertTrue({"gpui-pre", "gpui-kit"} <= crates)


class VersionComparisonTests(unittest.TestCase):
    def test_prerelease_sorts_before_release(self):
        self.assertLess(parse_version("0.4.0-pre.2"), parse_version("0.4.0"))
        self.assertLess(parse_version("0.4.0-pre.2"), parse_version("0.4.0-pre.10"))

    def test_yanked_versions_are_ignored(self):
        self.assertEqual(latest_version(api(("0.6.5", True), ("0.6.4", False))), "0.6.4")

    def test_prereleases_need_flag(self):
        response = api(("0.4.0-pre.1", False), ("0.3.6", False))
        self.assertEqual(latest_version(response), "0.3.6")
        self.assertEqual(latest_version(response, include_prerelease=True), "0.4.0-pre.1")

    def test_up_to_date_has_no_updates(self):
        pins = parse_pins(WORKSPACE_TOML, "Cargo.toml")
        self.assertEqual(find_updates(pins, {"gpui-pre": api(("0.3.5", False), ("0.3.4", False))}), [])

    def test_newer_version_is_reported(self):
        pins = parse_pins(WORKSPACE_TOML, "Cargo.toml") + parse_pins(APP_TOML, "apps/app/Cargo.toml")
        responses = {
            "gpui-pre": api(("0.3.6", False), ("0.3.5", False)),
            "gpui-pre-platform": api(("0.3.5", False)),
            "gpui-kit": api(("0.6.6", False), ("0.6.5", True), ("0.6.4", False)),
        }
        updates = find_updates(pins, responses)
        self.assertEqual([(u["crate"], u["pinned"], u["latest"]) for u in updates], [
            ("gpui-kit", ["0.6.2", "0.6.4"], "0.6.6"),
            ("gpui-pre", ["0.3.5"], "0.3.6"),
        ])
        self.assertEqual(updates[1]["url"], "https://crates.io/crates/gpui-pre")

    def test_only_yanked_newer_version_is_not_an_update(self):
        pins = parse_pins(APP_TOML, "apps/app/Cargo.toml")
        responses = {"gpui-kit": api(("0.6.7", True), ("0.6.4", False))}
        self.assertEqual([u["crate"] for u in find_updates(pins, responses)], ["gpui-kit"])
        pins = [p for p in pins if p["version"] == "0.6.4"]
        self.assertEqual(find_updates(pins, responses), [])


class IssueRenderingTests(unittest.TestCase):
    def test_issue_lists_crates_and_checklist(self):
        updates = [{
            "crate": "gpui-pre",
            "pinned": ["0.3.5"],
            "latest": "0.3.6",
            "url": "https://crates.io/crates/gpui-pre",
            "manifests": ["Cargo.toml"],
        }]
        title, body = render_issue(updates)
        self.assertEqual(title, "Snapshot watch: gpui-pre 0.3.6 available")
        self.assertTrue(body.startswith(ISSUE_MARKER))
        self.assertIn("| `gpui-pre` | `0.3.5` | `0.3.6` | [gpui-pre](https://crates.io/crates/gpui-pre) |", body)
        self.assertIn("- [ ] Run the coexistence sample", body)
        self.assertIn("inspect diffs before updating baselines", body)
        self.assertEqual(render_issue(updates), (title, body))

    def test_main_writes_github_output_without_network(self):
        responses = {"gpui-pre": api(("0.3.6", False))}
        with tempfile.TemporaryDirectory() as tmp, \
                mock.patch("check_snapshot_versions.fetch_crate", side_effect=responses.__getitem__), \
                contextlib.redirect_stdout(io.StringIO()), contextlib.redirect_stderr(io.StringIO()):
            root = Path(tmp)
            (root / "Cargo.toml").write_text(WORKSPACE_TOML)
            output, body = root / "out", root / "body.md"
            code = main(["--root", tmp, "--github-output", str(output), "--body-file", str(body)])
            self.assertEqual(code, 0)
            self.assertEqual(output.read_text(), "updates=true\ntitle=Snapshot watch: gpui-pre 0.3.6 available\n")
            self.assertIn(ISSUE_MARKER, body.read_text())


if __name__ == "__main__":
    unittest.main()
