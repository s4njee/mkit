"""Registry-wide conformance manifest freshness checks."""

from pathlib import Path
import tempfile
import unittest

from check_component_specs import check_registry
from generate_conformance import generate


SPEC = """---
spec_version: 1
component: example
states:
  - id: idle
    description: Ready.
    fixture: idle_fixture
keys: []
accessibility:
  role: button
  properties: []
---

# Example

## Purpose
Example.
## Anatomy
Example.
## States
Example.
## Props and events
Example.
## Keyboard map
Example.
## Pointer behaviour
Example.
## Accessibility role and properties
Example.
## Theme tokens used
Example.
## WAI-ARIA pattern reference
Example.
## Platform notes
Example.
## Open questions
None.
"""


class CheckComponentSpecsTests(unittest.TestCase):
    def test_detects_stale_manifest_and_name_mismatch(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            component = root / "example"
            component.mkdir()
            spec = component / "spec.md"
            spec.write_text(SPEC, encoding="utf-8")
            manifest = component / "tests" / "conformance.json"
            generate(spec, manifest)
            self.assertEqual(check_registry(root), [])

            manifest.write_text("{}\n", encoding="utf-8")
            self.assertIn("stale", "\n".join(check_registry(root)))

            generate(spec, manifest)
            spec.write_text(SPEC.replace("component: example", "component: different"), encoding="utf-8")
            generate(spec, manifest)
            self.assertIn("must match directory", "\n".join(check_registry(root)))

    def test_detects_missing_human_section(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            component = root / "example"
            component.mkdir()
            spec = component / "spec.md"
            spec.write_text(SPEC.replace("## Open questions", "## Questions"), encoding="utf-8")
            manifest = component / "tests" / "conformance.json"
            generate(spec, manifest)
            self.assertIn("missing required section 'Open questions'", "\n".join(check_registry(root)))


if __name__ == "__main__":
    unittest.main()
