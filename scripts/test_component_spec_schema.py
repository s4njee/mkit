"""Focused contract checks for the E5.1 component spec template."""

from pathlib import Path
import unittest

import yaml


ROOT = Path(__file__).resolve().parents[1]
MARKDOWN = ROOT / "docs/component-spec-template.md"
YAML_TEMPLATE = ROOT / "registry/component-spec.template.yaml"
REQUIRED_SECTIONS = (
    "Purpose",
    "Anatomy",
    "States",
    "Props and events",
    "Keyboard map",
    "Pointer behaviour",
    "Accessibility role and properties",
    "Theme tokens used",
    "WAI-ARIA pattern reference",
    "Platform notes",
    "Open questions",
)


def load_front_block():
    markdown = MARKDOWN.read_text(encoding="utf-8")
    pieces = markdown.split("---\n", 2)
    if len(pieces) != 3:
        raise AssertionError("component spec must start with a YAML front block")
    front_block = pieces[1]
    sample = "\n".join(
        line for line in YAML_TEMPLATE.read_text(encoding="utf-8").splitlines()
        if not line.startswith("#")
    ).strip()
    if front_block.strip() != sample:
        raise AssertionError("Markdown and standalone YAML front blocks must match")
    return markdown, yaml.safe_load(front_block)


class ComponentSpecSchemaTest(unittest.TestCase):
    def test_front_block_has_executable_keyboard_contract(self):
        _, spec = load_front_block()
        self.assertEqual(spec["spec_version"], 1)
        self.assertTrue(spec["component"])
        states = {state["id"] for state in spec["states"]}
        self.assertTrue(states)
        self.assertTrue(all(state["fixture"] for state in spec["states"]))

        for key in spec["keys"]:
            self.assertTrue(key["key"])
            self.assertIsInstance(key["modifiers"], list)
            self.assertTrue(key["when"])
            self.assertTrue(key["action"])
            self.assertIn(key["initial_state"], states)
            expected = key["expect"]
            self.assertTrue(expected)
            self.assertLessEqual(set(expected), {"state", "event", "focus_target"})
            self.assertTrue(all(value for value in expected.values()))
            if "state" in expected:
                self.assertIn(expected["state"], states)

    def test_required_human_sections_are_present(self):
        markdown, _ = load_front_block()
        for section in REQUIRED_SECTIONS:
            with self.subTest(section=section):
                self.assertIn(f"## {section}", markdown)


if __name__ == "__main__":
    unittest.main()
