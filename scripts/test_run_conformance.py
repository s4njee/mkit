import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent))
import run_conformance


class RunConformanceTests(unittest.TestCase):
    def test_adapter_pass_response_is_accepted(self):
        ok, message = run_conformance.run_case(
            [sys.executable, "-c", "import json,sys; json.load(sys.stdin); print(json.dumps({'passed': True, 'actual': {'state': 'on', 'event': 'activate'}}))"],
            {"id": "keyboard-01", "kind": "keyboard_cases", "assert": {"state": "on", "event": "activate"}},
        )
        self.assertTrue(ok)
        self.assertEqual(message, "passed")

    def test_unsupported_adapter_response_is_a_pending_failure(self):
        ok, message = run_conformance.run_case(
            [sys.executable, "-c", "import json; print(json.dumps({'status': 'unsupported', 'reason': 'inactive'}))"],
            {"id": "a11y-idle"},
        )
        self.assertFalse(ok)
        self.assertTrue(message.startswith("unsupported_pending:"))

    def test_missing_pass_is_a_failure(self):
        ok, message = run_conformance.run_case(
            [sys.executable, "-c", "print('{}')"], {"id": "keyboard-01"}
        )
        self.assertFalse(ok)
        self.assertIn("adapter reported failure", message)

    def test_keyboard_mismatch_fails_even_when_adapter_claims_pass(self):
        ok, message = run_conformance.run_case(
            [sys.executable, "-c", "import json; print(json.dumps({'passed': True, 'actual': {'state': 'off', 'event': 'activate'}}))"],
            {"id": "keyboard-01", "kind": "keyboard_cases", "assert": {"state": "on", "event": "activate"}},
        )
        self.assertFalse(ok)
        self.assertIn("keyboard state mismatch", message)

    def test_accessibility_semantics_are_compared(self):
        case = {
            "kind": "accessibility_cases",
            "expected_semantics": {"role": "switch", "properties": [{"name": "checked", "value": True}]},
        }
        self.assertEqual(
            run_conformance.verify_actual(case, {"role": "switch", "properties": {"checked": False}})[0],
            False,
        )
        self.assertTrue(run_conformance.verify_actual(
            case, {"role": "switch", "properties": {"checked": True}}
        )[0])

    def test_screenshot_requires_expected_baseline_and_match_evidence(self):
        case = {"kind": "screenshot_cases", "baseline": "toggle/on/light-1x.png"}
        self.assertFalse(run_conformance.verify_actual(case, {"baseline": case["baseline"], "matched": False})[0])
        self.assertTrue(run_conformance.verify_actual(case, {"baseline": case["baseline"], "matched": True})[0])

    def test_bare_pass_boolean_is_not_evidence(self):
        ok, message = run_conformance.run_case(
            [sys.executable, "-c", "import json; print(json.dumps({'passed': True}))"],
            {"id": "keyboard-01", "kind": "keyboard_cases", "assert": {"state": "on"}},
        )
        self.assertFalse(ok)
        self.assertIn("actual must be a JSON object", message)

    def test_case_filters_select_only_requested_ids_and_kinds(self):
        manifest = {
            "component": "sample",
            "keyboard_cases": [{"id": "keyboard-01"}, {"id": "keyboard-02"}],
            "accessibility_cases": [{"id": "a11y-idle"}],
            "screenshot_cases": [{"id": "screen-idle-light-1x"}],
        }
        selected = run_conformance.select_cases(
            manifest, {"keyboard_cases"}, {"keyboard-02"}
        )
        self.assertEqual([case["id"] for case in selected], ["keyboard-02"])
        self.assertEqual(selected[0]["kind"], "keyboard_cases")


if __name__ == "__main__":
    unittest.main()
