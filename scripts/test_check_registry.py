import copy
import json
import unittest

from check_registry import ROOT, validate


class RegistryValidationTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.registry = json.loads((ROOT / "registry/registry.json").read_text())
        cls.required_versions = {"mkit-core": "0.1.0", "gpui-pre": "0.3.5"}

    def test_checked_in_registry_is_valid(self):
        validate(self.registry, "0.1.0", self.required_versions)

    def test_release_version_tracks_workspace(self):
        registry = copy.deepcopy(self.registry)
        registry["release_version"] = "9.9.9"
        with self.assertRaisesRegex(ValueError, "release_version"):
            validate(registry, "0.1.0", self.required_versions)

    def test_draft_cannot_claim_source_files(self):
        registry = copy.deepcopy(self.registry)
        registry["components"][0]["status"] = "draft_spec_only"
        with self.assertRaisesRegex(ValueError, "cannot claim source files"):
            validate(registry, "0.1.0", self.required_versions)

    def test_source_ready_requires_source_files(self):
        registry = copy.deepcopy(self.registry)
        registry["components"][0]["status"] = "source_ready"
        registry["components"][0]["source_files"] = []
        with self.assertRaisesRegex(ValueError, "requires source files"):
            validate(registry, "0.1.0", self.required_versions)

    def test_implementation_in_progress_requires_source_files(self):
        registry = copy.deepcopy(self.registry)
        registry["components"][0]["status"] = "implementation_in_progress"
        registry["components"][0]["source_files"] = []
        with self.assertRaisesRegex(ValueError, "requires source files"):
            validate(registry, "0.1.0", self.required_versions)

    def test_versions_follow_workspace_dependencies(self):
        registry = copy.deepcopy(self.registry)
        registry["components"][0]["dependencies"][1]["version"] = "0.3.4"
        with self.assertRaisesRegex(ValueError, "gpui-pre version"):
            validate(registry, "0.1.0", self.required_versions)


    def test_component_dependency_references_must_exist(self):
        registry = copy.deepcopy(self.registry)
        registry["components"][0]["component_dependencies"] = ["missing-widget"]
        with self.assertRaisesRegex(ValueError, "unknown component dependency"):
            validate(registry, "0.1.0", self.required_versions)

    def test_component_dependency_cycles_are_rejected(self):
        registry = copy.deepcopy(self.registry)
        registry["components"][0]["component_dependencies"] = ["scrubbable-number-field"]
        registry["components"][1]["component_dependencies"] = ["combobox"]
        with self.assertRaisesRegex(ValueError, "cycle"):
            validate(registry, "0.1.0", self.required_versions)

    def test_spec_path_cannot_escape_repository(self):
        registry = copy.deepcopy(self.registry)
        registry["components"][0]["spec"] = "/etc/passwd"
        with self.assertRaisesRegex(ValueError, "inside the repository"):
            validate(registry, "0.1.0", self.required_versions)

    def test_component_dependencies_are_distinct_from_crate_dependencies(self):
        self.assertEqual(self.registry["components"][0]["component_dependencies"], [])
        self.assertEqual(
            {dependency["name"] for dependency in self.registry["components"][0]["dependencies"]},
            {"mkit-core", "gpui-pre"},
        )

    def test_duplicate_names_are_rejected(self):
        registry = copy.deepcopy(self.registry)
        registry["components"].append(copy.deepcopy(registry["components"][0]))
        with self.assertRaisesRegex(ValueError, "duplicate component"):
            validate(registry, "0.1.0", self.required_versions)


if __name__ == "__main__":
    unittest.main()
