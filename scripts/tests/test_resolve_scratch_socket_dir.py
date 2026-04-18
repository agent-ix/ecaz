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
        self.preferred_dir = self.tmp_dir / "preferred"
        self.fallback_dir = self.home_dir / ".pgrx"

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
        env["TQV_SCRATCH_TEST_PREFERRED_SOCKET_DIR"] = str(self.preferred_dir)
        env["TQV_SCRATCH_TEST_FALLBACK_SOCKET_DIR"] = str(self.fallback_dir)
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
        self._touch_socket_marker(self.fallback_dir)

        result = self._run_helper()

        self.assertNotEqual(result.returncode, 0)
        self.assertEqual(result.stdout.strip(), "")
        self.assertIn("refusing to fall back", result.stderr)
        self.assertIn(str(self.fallback_dir), result.stderr)

    def test_reports_missing_preferred_socket(self) -> None:
        result = self._run_helper()

        self.assertNotEqual(result.returncode, 0)
        self.assertEqual(result.stdout.strip(), "")
        self.assertIn(
            f"scratch wrapper expected socket at {self.preferred_dir}/.s.PGSQL.28817",
            result.stderr,
        )

    def test_pghost_override_wins_and_warns_when_preferred_socket_exists(self) -> None:
        self._touch_socket_marker(self.preferred_dir)
        override_dir = self.tmp_dir / "override"

        result = self._run_helper(PGHOST=str(override_dir))

        self.assertEqual(result.returncode, 0)
        self.assertEqual(result.stdout.strip(), str(override_dir))
        self.assertIn("using PGHOST", result.stderr)
        self.assertIn(str(self.preferred_dir), result.stderr)


if __name__ == "__main__":
    unittest.main()
