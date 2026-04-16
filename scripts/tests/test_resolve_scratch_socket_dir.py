#!/usr/bin/env python3
"""Regression tests for scratch socket resolution policy."""

from __future__ import annotations

import os
from pathlib import Path
import subprocess
import tempfile
import unittest


REPO_ROOT = Path(__file__).resolve().parents[2]
HELPER = REPO_ROOT / "scripts" / "resolve_scratch_socket_dir.sh"


class ResolveScratchSocketDirTests(unittest.TestCase):
    def setUp(self) -> None:
        self._tmp = tempfile.TemporaryDirectory()
        self.tmp_dir = Path(self._tmp.name)
        self.home_dir = self.tmp_dir / "home"
        self.home_dir.mkdir()

    def tearDown(self) -> None:
        self._tmp.cleanup()

    def _touch_socket_marker(self, directory: Path, port: int = 28817) -> None:
        directory.mkdir(parents=True, exist_ok=True)
        (directory / f".s.PGSQL.{port}").touch()

    def _run_helper(self, **env_overrides: str) -> subprocess.CompletedProcess[str]:
        env = os.environ.copy()
        env["HOME"] = str(self.home_dir)
        env["TQV_PG_PORT"] = "28817"
        env["TQV_SCRATCH_TEST_ACCEPT_FILES"] = "1"
        env.pop("PGHOST", None)
        env.pop("TQV_PG_SOCKET_DIR", None)
        env.update(env_overrides)
        return subprocess.run(
            [str(HELPER)],
            check=False,
            capture_output=True,
            text=True,
            env=env,
        )

    def test_explicit_socket_override_wins(self) -> None:
        override_dir = self.tmp_dir / "explicit"
        result = self._run_helper(TQV_PG_SOCKET_DIR=str(override_dir))
        self.assertEqual(result.returncode, 0)
        self.assertEqual(result.stdout.strip(), str(override_dir))
        self.assertEqual(result.stderr.strip(), "")

    def test_refuses_home_pgrx_fallback_without_override(self) -> None:
        self._touch_socket_marker(self.home_dir / ".pgrx")

        result = self._run_helper()

        self.assertNotEqual(result.returncode, 0)
        self.assertEqual(result.stdout.strip(), "")
        self.assertIn("refusing to fall back", result.stderr)
        self.assertIn(str(self.home_dir / ".pgrx"), result.stderr)

    def test_reports_missing_preferred_socket(self) -> None:
        result = self._run_helper()

        self.assertNotEqual(result.returncode, 0)
        self.assertEqual(result.stdout.strip(), "")
        self.assertIn(
            "scratch wrapper expected socket at /tmp/tqvector_pgrx_home/.s.PGSQL.28817",
            result.stderr,
        )


if __name__ == "__main__":
    unittest.main()
