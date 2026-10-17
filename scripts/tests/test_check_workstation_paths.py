#!/usr/bin/env python3

from __future__ import annotations

import importlib.util
import sys
import unittest
from pathlib import Path


SCRIPT = Path(__file__).parents[1] / "check_workstation_paths.py"
SPEC = importlib.util.spec_from_file_location("check_workstation_paths", SCRIPT)
assert SPEC is not None and SPEC.loader is not None
MODULE = importlib.util.module_from_spec(SPEC)
sys.modules[SPEC.name] = MODULE
SPEC.loader.exec_module(MODULE)


class WorkstationPathCheckTests(unittest.TestCase):
    def test_added_workstation_path_is_reported_without_its_value(self) -> None:
        private_path = "/" + "home" + "/example/project/output.log"
        diff = "\n".join(
            [
                "diff --git a/report.md b/report.md",
                "--- a/report.md",
                "+++ b/report.md",
                "@@ -2,0 +3 @@",
                f'+artifact = "{private_path}"',
            ]
        )

        violations = MODULE.violations_for_diff(diff)
        self.assertEqual(len(violations), 1)
        self.assertEqual(violations[0].path, "report.md")
        self.assertEqual(violations[0].line_number, 3)
        rendered = MODULE.render_violations(violations)
        self.assertNotIn(private_path, rendered)

    def test_removed_paths_and_portable_values_are_ignored(self) -> None:
        private_path = "/" + "Users" + "/example/project/output.log"
        diff = "\n".join(
            [
                "diff --git a/report.md b/report.md",
                "--- a/report.md",
                "+++ b/report.md",
                "@@ -3 +3 @@",
                f'-artifact = "{private_path}"',
                '+artifact = "${ARTIFACT_ROOT}/output.log"',
            ]
        )

        self.assertEqual(MODULE.violations_for_diff(diff), [])

    def test_root_tilde_and_windows_home_paths_are_detected(self) -> None:
        root_path = "/" + "root" + "/work/output.log"
        tilde_path = "~" + "/work/output.log"
        windows_path = "C:" + "\\" + "Users\\example\\output.log"
        diff = "\n".join(
            [
                "diff --git a/report.md b/report.md",
                "--- a/report.md",
                "+++ b/report.md",
                "@@ -0,0 +1,3 @@",
                f"+{root_path}",
                f"+{tilde_path}",
                f"+{windows_path}",
            ]
        )

        self.assertEqual(len(MODULE.violations_for_diff(diff)), 3)


if __name__ == "__main__":
    unittest.main()
