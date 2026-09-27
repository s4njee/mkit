import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent))
import generate_conformance as generator


SAMPLE = """---
spec_version: 1
component: demo-toggle
states:
  - id: "off"
    description: Toggle is off.
    fixture: DemoToggle::off
  - id: "on"
    description: Toggle is on.
    fixture: DemoToggle::on
keys:
  - key: Space
    modifiers: []
    when: component is focused
    action: Toggle to on.
    initial_state: "off"
    expect:
      state: "on"
      event: Changed(true)
      focus_target: toggle
accessibility:
  role: switch
  properties:
    - name: checked
      value: false
      when: "off"
    - name: checked
      value: true
      when: "on"
---
"""


class GenerateConformanceTests(unittest.TestCase):
    def test_builds_executable_keyboard_assertions_and_complete_visual_matrix(self):
        spec = generator.extract_front_matter(SAMPLE)
        manifest = generator.build_manifest(spec, "cargo test -p demo-toggle")
        self.assertEqual(manifest["keyboard_cases"][0]["load_fixture"], "DemoToggle::off")
        self.assertEqual(manifest["keyboard_cases"][0]["assert"], {
            "state": "on", "event": "Changed(true)", "focus_target": "toggle"
        })
        self.assertEqual(len(manifest["accessibility_cases"]), 2)
        self.assertEqual(manifest["accessibility_cases"][1]["expected_semantics"]["properties"], [
            {"name": "checked", "value": True}
        ])
        self.assertEqual(len(manifest["screenshot_cases"]), 2 * 3 * 2)
        self.assertTrue(all(case["status"] == "pending_until_supported_platform_run"
                            for case in manifest["screenshot_cases"]))
        self.assertEqual(manifest["adapter_contract"]["custom_tests"], "cargo test -p demo-toggle")

    def test_rejects_keyboard_behavior_without_assertion(self):
        spec = generator.extract_front_matter(SAMPLE).copy()
        spec["keys"] = [dict(spec["keys"][0], expect={})]
        with self.assertRaisesRegex(generator.SpecError, "expect must contain"):
            generator.build_manifest(spec)

    def test_rejects_unknown_expected_state(self):
        spec = generator.extract_front_matter(SAMPLE).copy()
        spec["keys"] = [dict(spec["keys"][0], expect={"state": "missing"})]
        with self.assertRaisesRegex(generator.SpecError, "unknown state"):
            generator.build_manifest(spec)

    def test_rejects_accessibility_property_for_unknown_state(self):
        spec = generator.extract_front_matter(SAMPLE)
        spec["accessibility"]["properties"][0]["when"] = "typo"
        with self.assertRaisesRegex(generator.SpecError, "references unknown state"):
            generator.build_manifest(spec)

    def test_rejects_ambiguous_accessibility_property_for_a_state(self):
        spec = generator.extract_front_matter(SAMPLE)
        spec["accessibility"]["properties"].append(
            {"name": "checked", "value": True, "when": "off"}
        )
        with self.assertRaisesRegex(generator.SpecError, "more than once"):
            generator.build_manifest(spec)

    def test_state_can_override_role_and_exclude_generic_property(self):
        spec = generator.extract_front_matter(SAMPLE)
        spec["states"][1]["accessibility_role"] = "password"
        spec["accessibility"]["properties"][0] = {
            "name": "value", "value": "clear", "except_when": ["on"]
        }
        spec["accessibility"]["properties"][1] = {
            "name": "value", "value": "masked", "when": "on"
        }
        cases = generator.build_manifest(spec)["accessibility_cases"]
        self.assertEqual(cases[0]["expected_semantics"], {
            "role": "switch", "properties": [{"name": "value", "value": "clear"}]
        })
        self.assertEqual(cases[1]["expected_semantics"], {
            "role": "password", "properties": [{"name": "value", "value": "masked"}]
        })

    def test_requires_a_front_block(self):
        with self.assertRaisesRegex(generator.SpecError, "must begin"):
            generator.extract_front_matter("# no front matter")


if __name__ == "__main__":
    unittest.main()
