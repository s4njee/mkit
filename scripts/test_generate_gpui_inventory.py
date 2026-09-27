"""Regression checks for the pinned, root-reachable GPUI inventory."""

import json
import unittest
from pathlib import Path

from generate_gpui_inventory import destination, markdown


DATA = json.loads((Path(__file__).resolve().parents[1] / "book/gpui-api-inventory.json").read_text())
WASM = json.loads((Path(__file__).resolve().parents[1] / "book/gpui-api-inventory-wasm.json").read_text())
LINUX_FEATURE_AUDIT = json.loads((Path(__file__).resolve().parents[1] / "book/gpui-api-inventory-linux-feature-audit.json").read_text())
WASM_TEST_SUPPORT = json.loads((Path(__file__).resolve().parents[1] / "book/gpui-api-inventory-wasm-test-support.json").read_text())
LINUX_ALL_FEATURES = json.loads((Path(__file__).resolve().parents[1] / "book/gpui-api-inventory-linux-all-features.json").read_text())


class InventoryTests(unittest.TestCase):
    def test_reexports_and_name_collisions_survive(self):
        pairs = {(item["path"], item["kind"]) for item in DATA["items"]}
        self.assertEqual(len(pairs), DATA["count"])
        for pair in (("gpui::AppContext", "trait"), ("gpui::AppContext", "macro"),
                     ("gpui::SharedString", "struct"), ("gpui::private", "module"),
                     ("gpui::test", "module"), ("gpui::test", "macro")):
            self.assertIn(pair, pairs)

    def test_links_hidden_ancestry_and_classification(self):
        revision = DATA["zed_revision"]
        for item in DATA["items"]:
            self.assertIn(f"/blob/{revision}/", item["source_url"])
            self.assertTrue(destination(item))
            if item["path"].startswith("gpui::private"):
                self.assertTrue(item["doc_hidden"])
        page = markdown(DATA)
        self.assertEqual(page.count("| [source]("), DATA["count"])

    def test_linux_all_features_observes_screen_capture_module(self):
        entry = next(x for x in DATA["items"] if x["path"] == "gpui::scap_screen_capture")
        self.assertEqual(entry["kind"], "module")
        self.assertIn("aarch64-unknown-linux-gnu", entry["available_targets"])
        self.assertIn("Linux all features (isolated rustdoc)", entry["observed_profiles"])
        self.assertIn("Windows, Linux, or FreeBSD", entry["availability_condition"])
        self.assertEqual(DATA["source_audited_unverified"], [])

    def test_wasm_target_profile_is_merged_with_explicit_feature_scope(self):
        self.assertEqual(WASM["target"], "wasm32-unknown-unknown")
        self.assertEqual(WASM["feature_profile"], "no default features (test-support omitted because wait-timeout 0.2.1 does not compile on wasm)")
        self.assertIn("E0433", WASM["attempted_target_blocker"])
        wasm_names = {(item["path"], item["kind"]) for item in WASM["items"]}
        union = {(item["path"], item["kind"]): item for item in DATA["items"]}
        self.assertTrue(wasm_names <= union.keys())
        priority_sender = union[("gpui::PriorityQueueSender", "struct")]
        self.assertIn("wasm32-unknown-unknown", priority_sender["available_targets"])
        self.assertIn("WebAssembly", markdown(DATA))
        self.assertEqual(DATA["per_target_counts"]["wasm32-unknown-unknown"], WASM["count"])

    def test_linux_optional_feature_audit_is_reproducible_and_adds_no_names(self):
        self.assertEqual(LINUX_FEATURE_AUDIT["target"], "aarch64-unknown-linux-gnu")
        self.assertIn("bench,inspector,leak-detection,profiler,stacker,wayland", LINUX_FEATURE_AUDIT["feature_profile"])
        self.assertEqual(LINUX_FEATURE_AUDIT["count"], 573)
        audit_names = {(item["path"], item["kind"]) for item in LINUX_FEATURE_AUDIT["items"]}
        inventory_names = {(item["path"], item["kind"]) for item in DATA["items"]}
        self.assertTrue(audit_names <= inventory_names)
        self.assertEqual(DATA["linux_feature_profile_audit"]["new_names_added"], 0)
        self.assertIn("Linux broad optional-feature audit", DATA["per_profile_counts"])
        self.assertIn("x11", DATA["linux_feature_profile_audit"]["excludes"])

    def test_isolated_blocked_profiles_are_snapshotted_with_shim_limits(self):
        self.assertEqual(WASM_TEST_SUPPORT["target"], "wasm32-unknown-unknown")
        self.assertEqual(WASM_TEST_SUPPORT["count"], 566)
        self.assertEqual(LINUX_ALL_FEATURES["target"], "aarch64-unknown-linux-gnu")
        self.assertEqual(LINUX_ALL_FEATURES["count"], 632)
        self.assertEqual(len(WASM_TEST_SUPPORT["dependency_shims"]), 2)
        self.assertTrue(all("never executed" in item for item in WASM_TEST_SUPPORT["dependency_shims"]))
        self.assertIn("no linking", LINUX_ALL_FEATURES["dependency_shims"][0])
        self.assertIn("Linux all features (isolated rustdoc)", DATA["per_profile_counts"])
        self.assertIn("WebAssembly test-support (isolated rustdoc)", DATA["per_profile_counts"])
        names = {(x["path"], x["kind"]) for x in LINUX_ALL_FEATURES["items"]}
        self.assertIn(("gpui::scap_screen_capture", "module"), names)

    def test_cfg_adjacent_public_declarations_have_reviewed_reachability(self):
        dispositions = {(item["name"], item["source_file"]): item["disposition"]
                        for item in DATA["cfg_public_declaration_candidates"]}
        self.assertEqual(set(dispositions), {("TaffyLayoutEngine", "src/taffy.rs")})
        self.assertIn("not root-reachable", dispositions[("TaffyLayoutEngine", "src/taffy.rs")])


if __name__ == "__main__":
    unittest.main()
