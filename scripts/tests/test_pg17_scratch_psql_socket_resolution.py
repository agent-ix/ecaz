#!/usr/bin/env python3
"""Smoke tests for pg17 scratch psql socket resolution."""

from __future__ import annotations

import json
import os
from pathlib import Path
import subprocess
import tempfile
import unittest


REPO_ROOT = Path(__file__).resolve().parents[2]
WRAPPER = REPO_ROOT / "scripts" / "pg17_scratch_psql.sh"


_FAKE_PSQL = """#!/usr/bin/env python3
from __future__ import annotations

import json
import os
import sys


log_path = os.environ["TQV_FAKE_PSQL_LOG"]
with open(log_path, "w", encoding="utf-8") as fh:
    json.dump(sys.argv[1:], fh)
"""


class Pg17ScratchPsqlSocketResolutionTests(unittest.TestCase):
    def setUp(self) -> None:
        self._tmp = tempfile.TemporaryDirectory()
        self.tmp_dir = Path(self._tmp.name)
        self.home_dir = self.tmp_dir / "home"
        self.home_dir.mkdir()
        self.preferred_dir = self.tmp_dir / "preferred"
        self.override_dir = self.tmp_dir / "override"
        self.log_file = self.tmp_dir / "fake_psql_args.json"
        self.fake_psql = self.tmp_dir / "fake_psql.py"
        self.fake_psql.write_text(_FAKE_PSQL, encoding="utf-8")
        self.fake_psql.chmod(0o755)

    def tearDown(self) -> None:
        self._tmp.cleanup()

    def _touch_socket_marker(self, directory: Path, port: int = 28817) -> None:
        directory.mkdir(parents=True, exist_ok=True)
        (directory / f".s.PGSQL.{port}").touch()

    def _run_wrapper(
        self, *args: str, **env_overrides: str
    ) -> subprocess.CompletedProcess[str]:
        env = os.environ.copy()
        env["HOME"] = str(self.home_dir)
        env["TQV_PG_PORT"] = "28817"
        env["TQV_PSQL_BIN"] = str(self.fake_psql)
        env["TQV_FAKE_PSQL_LOG"] = str(self.log_file)
        env["TQV_SCRATCH_TEST_ACCEPT_FILES"] = "1"
        env["TQV_SCRATCH_TEST_PREFERRED_SOCKET_DIR"] = str(self.preferred_dir)
        env.pop("PGHOST", None)
        env.pop("TQV_PG_SOCKET_DIR", None)
        env.update(env_overrides)
        return subprocess.run(
            ["bash", str(WRAPPER), *args, "--sql", "SELECT 1"],
            cwd=REPO_ROOT,
            env=env,
            text=True,
            capture_output=True,
            check=False,
        )

    def test_wrapper_uses_resolved_preferred_socket_by_default(self) -> None:
        self._touch_socket_marker(self.preferred_dir)

        result = self._run_wrapper()

        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(result.stderr.strip(), "")
        argv = json.loads(self.log_file.read_text(encoding="utf-8"))
        self.assertEqual(argv[:6], ["-h", str(self.preferred_dir), "-p", "28817", "-d", "postgres"])
        self.assertEqual(argv[-2:], ["-c", "SELECT 1"])

    def test_wrapper_honors_explicit_socket_override(self) -> None:
        result = self._run_wrapper(TQV_PG_SOCKET_DIR=str(self.override_dir))

        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(result.stderr.strip(), "")
        argv = json.loads(self.log_file.read_text(encoding="utf-8"))
        self.assertEqual(argv[:2], ["-h", str(self.override_dir)])
        self.assertEqual(argv[-2:], ["-c", "SELECT 1"])

    def test_wrapper_honors_socket_dir_argument(self) -> None:
        result = self._run_wrapper("--socket-dir", str(self.override_dir))

        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(result.stderr.strip(), "")
        argv = json.loads(self.log_file.read_text(encoding="utf-8"))
        self.assertEqual(argv[:2], ["-h", str(self.override_dir)])
        self.assertEqual(argv[-2:], ["-c", "SELECT 1"])


if __name__ == "__main__":
    unittest.main()
