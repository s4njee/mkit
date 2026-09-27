import unittest

import check_e5_dod


class DefinitionOfDoneTests(unittest.TestCase):
    def test_current_template_and_workflow_satisfy_contract(self):
        template = check_e5_dod.TEMPLATE.read_text()
        workflow = check_e5_dod.WORKFLOW.read_text()
        self.assertEqual(check_e5_dod.validate(template, workflow), [])

    def test_missing_human_review_and_ci_gate_are_reported(self):
        template = ""
        workflow = "cargo build --workspace --locked"
        errors = check_e5_dod.validate(template, workflow)
        self.assertTrue(any("Maintainer approved" in error for error in errors))
        self.assertTrue(any("cargo test --workspace" in error for error in errors))
        self.assertTrue(any("check_e5_dod.py" in error for error in errors))

    def test_removing_focused_pilot_screenshot_gate_is_reported(self):
        template = check_e5_dod.TEMPLATE.read_text()
        workflow = check_e5_dod.WORKFLOW.read_text().replace(
            "--kind screenshot --case screen-idle-light-1x", ""
        )
        errors = check_e5_dod.validate(template, workflow)
        self.assertTrue(any("screen-idle-light-1x" in error for error in errors))


if __name__ == "__main__":
    unittest.main()
