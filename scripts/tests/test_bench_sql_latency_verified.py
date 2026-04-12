#!/usr/bin/env python3
"""Regression tests for per-cell planner verification in verified latency runs.

These tests drive the real shell launchers against a fake `psql` binary so the
planner guard can be exercised without a live Postgres cluster. The fake server
models a small real-corpus fixture with a single query row and a single HNSW
index; individual `ef_search` values can be configured to fall back to a
`Seq Scan + Sort` plan so the test can assert that verified mode aborts before
timing the bad cell.
"""

from __future__ import annotations

import os
from pathlib import Path
import subprocess
import tempfile
import textwrap
import unittest


REPO_ROOT = Path(__file__).resolve().parents[2]
VERIFIED_SCRIPT = REPO_ROOT / "scripts" / "bench_sql_latency_verified.sh"


_FAKE_PSQL = """#!/usr/bin/env python3
from __future__ import annotations

import json
import os
import re
import sys


def load_sql(argv: list[str]) -> str:
    if "-c" in argv:
        idx = argv.index("-c")
        return argv[idx + 1]
    return sys.stdin.read()


def extract_ef(sql: str) -> int | None:
    match = re.search(r"SET\\s+tqhnsw\\.ef_search\\s*=\\s*(\\d+)", sql)
    if match:
        return int(match.group(1))
    return None


sql = load_sql(sys.argv[1:])
normalized = " ".join(sql.split())
fallback_ef = os.environ.get("TQV_FAKE_PSQL_FALLBACK_EF")
expected_index = "tqhnsw_real_test_m8_idx"

if "SHOW shared_buffers" in normalized:
    print("128MB")
elif "SHOW work_mem" in normalized:
    print("4MB")
elif "SHOW max_parallel_workers_per_gather" in normalized:
    print("2")
elif "SELECT count(*) FROM tqhnsw_real_test_queries;" in normalized:
    print("1")
elif "SELECT source FROM tqhnsw_real_test_queries ORDER BY id LIMIT 1;" in normalized:
    print("{0.1,0.2,0.3,0.4}")
elif "SELECT source FROM tqhnsw_real_test_queries ORDER BY id LIMIT 1" in normalized:
    print("{0.1,0.2,0.3,0.4}")
elif "SELECT source FROM tqhnsw_real_test_queries ORDER BY id;" in normalized:
    print("{0.1,0.2,0.3,0.4}")
elif "SELECT source FROM tqhnsw_real_test_queries ORDER BY id" in normalized:
    print("{0.1,0.2,0.3,0.4}")
elif "SELECT to_regclass('tqhnsw_real_test_m8_idx') IS NOT NULL;" in normalized:
    print("t")
elif "SELECT to_regclass('tqhnsw_real_test_m8_idx') IS NOT NULL" in normalized:
    print("t")
elif "EXPLAIN (ANALYZE, TIMING, FORMAT JSON)" in normalized:
    ef = extract_ef(sql) or 0
    print(json.dumps([{"Execution Time": float(100 + ef)}]))
elif "EXPLAIN" in normalized:
    ef = extract_ef(sql) or 0
    if fallback_ef and ef == int(fallback_ef):
        print(
            "Limit\\n"
            "  ->  Sort\\n"
            "        Sort Key: ((tqhnsw_real_test_corpus.embedding <#> '{0.1,0.2,0.3,0.4}'::real[]))\\n"
            "        ->  Seq Scan on tqhnsw_real_test_corpus"
        )
    else:
        print(
            "Limit\\n"
            f"  ->  Index Scan using {expected_index} on tqhnsw_real_test_corpus\\n"
            "        Order By: (embedding <#> '{0.1,0.2,0.3,0.4}'::real[])"
        )
else:
    print(f"unhandled fake psql SQL: {normalized}", file=sys.stderr)
    sys.exit(1)
"""


class BenchSqlLatencyVerifiedTests(unittest.TestCase):
    def setUp(self) -> None:
        self._tmp = tempfile.TemporaryDirectory()
        self.tmp_dir = Path(self._tmp.name)
        self.fake_psql = self.tmp_dir / "fake_psql.py"
        self.fake_psql.write_text(_FAKE_PSQL, encoding="utf-8")
        self.fake_psql.chmod(0o755)

    def tearDown(self) -> None:
        self._tmp.cleanup()

    def _run_verified(self, *, ef_search: str, fallback_ef: str | None) -> subprocess.CompletedProcess[str]:
        summary_file = self.tmp_dir / "summary.txt"
        env = os.environ.copy()
        env["TQV_PSQL_BIN"] = str(self.fake_psql)
        if fallback_ef is not None:
            env["TQV_FAKE_PSQL_FALLBACK_EF"] = fallback_ef
        else:
            env.pop("TQV_FAKE_PSQL_FALLBACK_EF", None)
        return subprocess.run(
            [
                "bash",
                str(VERIFIED_SCRIPT),
                "--prefix",
                "tqhnsw_real_test",
                "--m",
                "8",
                "--ef-search",
                ef_search,
                "--query-limit",
                "1",
                "--output",
                str(summary_file),
            ],
            cwd=REPO_ROOT,
            env=env,
            text=True,
            capture_output=True,
            check=False,
        )

    def test_verified_launcher_aborts_before_timing_fallback_cell(self) -> None:
        summary_file = self.tmp_dir / "summary.txt"
        result = self._run_verified(ef_search="40,200", fallback_ef="200")

        self.assertNotEqual(result.returncode, 0, result.stderr)
        self.assertIn(
            "planner verification failed for tqhnsw_real_test_m8_idx at ef_search=200",
            result.stderr,
        )

        lines = summary_file.read_text(encoding="utf-8").splitlines()
        self.assertEqual(len(lines), 1, lines)
        self.assertIn("ef_search=40", lines[0])
        self.assertNotIn("ef_search=200", "\n".join(lines))

    def test_verified_launcher_runs_all_cells_when_each_plan_uses_index(self) -> None:
        summary_file = self.tmp_dir / "summary.txt"
        result = self._run_verified(ef_search="40,128", fallback_ef=None)

        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn(
            "[verified] planner uses tqhnsw_real_test_m8_idx at ef_search=40",
            result.stderr,
        )
        self.assertIn(
            "[verified] planner uses tqhnsw_real_test_m8_idx at ef_search=128",
            result.stderr,
        )

        lines = summary_file.read_text(encoding="utf-8").splitlines()
        self.assertEqual(len(lines), 2, lines)
        self.assertIn("ef_search=40", lines[0])
        self.assertIn("ef_search=128", lines[1])


if __name__ == "__main__":
    unittest.main()
