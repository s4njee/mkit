"""Regression checks for generated concept-index coverage and route identity."""

import unittest
from contextlib import redirect_stderr
from io import StringIO
from pathlib import Path
from unittest.mock import patch

import generate_concept_index as generator
from generate_concept_index import (
    EXPECTED_COUNT,
    INVENTORY,
    OUTPUT,
    ROUTES,
    generate,
    parse_inventory,
)


class ConceptIndexTests(unittest.TestCase):
    def test_checked_in_index_is_deterministic_and_covers_every_inventory_row(self):
        rendered = generate()
        self.assertEqual(rendered, OUTPUT.read_text())
        self.assertEqual(rendered.count("| [`gpui::"), EXPECTED_COUNT)
        self.assertEqual(len(ROUTES), 83)
        self.assertIn("| Future coverage |", rendered)

    def test_duplicate_api_names_by_kind_and_source_are_preserved(self):
        source = "https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/x.rs"
        markdown = f"""## duplicate fixtures (3)

| API | Kind | Observed target | Source |
| --- | --- | --- | --- |
| `gpui::Same` | struct | macOS | [source]({source}#L1) |
| `gpui::Same` | trait | macOS | [source]({source}#L2) |
| `gpui::Same` | struct | macOS | [source]({source}#L3) |
"""
        groups = parse_inventory(markdown)
        self.assertEqual(len(groups[0].items), 3)
        self.assertEqual(len({item.route_key for item in groups[0].items}), 3)

    def test_exact_duplicate_path_kind_and_source_is_rejected(self):
        source = "https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/x.rs#L1"
        row = f"| `gpui::Same` | struct | macOS | [source]({source}) |"
        markdown = f"## duplicates (2)\n\n| API | Kind | Observed target | Source |\n| --- | --- | --- | --- |\n{row}\n{row}\n"
        with self.assertRaisesRegex(ValueError, "duplicate API path/kind/source"):
            parse_inventory(markdown)

    def test_published_route_keys_are_unique_and_present_in_inventory(self):
        groups = parse_inventory(INVENTORY.read_text())
        items = {item.route_key for group in groups for item in group.items}
        self.assertEqual(len(items), EXPECTED_COUNT)
        self.assertTrue(set(ROUTES).issubset(items))
        self.assertEqual(len(ROUTES), len(set(ROUTES)))

    def test_check_mode_reports_a_missing_output(self):
        missing = Path(__file__).resolve().parents[1] / "book/src/appendices/__missing_concept_index_test.md"
        message = StringIO()
        with patch.object(generator, "OUTPUT", missing), redirect_stderr(message):
            self.assertEqual(generator.main(["--check"]), 1)
        self.assertIn("is missing", message.getvalue())


if __name__ == "__main__":
    unittest.main()
