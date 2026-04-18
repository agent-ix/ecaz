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
    if "-f" in argv:
        idx = argv.index("-f")
        with open(argv[idx + 1], "r", encoding="utf-8") as fh:
            return fh.read()
    return sys.stdin.read()


def extract_ef(sql: str) -> int | None:
    match = re.search(r"SET\\s+tqhnsw\\.ef_search\\s*=\\s*(\\d+)", sql)
    if match:
        return int(match.group(1))
    return None


def strip_meta_commands(sql: str) -> str:
    lines = []
    for line in sql.splitlines():
        if line.lstrip().startswith("\\o"):
            continue
        lines.append(line)
    return "\\n".join(lines).strip()


def log_event(kind: str, ef: int | None) -> None:
    log_path = os.environ.get("TQV_FAKE_PSQL_LOG")
    if not log_path:
        return
    with open(log_path, "a", encoding="utf-8") as fh:
        fh.write(f"{kind}:{ef if ef is not None else 'none'}\\n")


def plan_output(expected_index: str, ef: int, fallback_ef: str | None) -> str:
    corpus_table = os.environ.get("TQV_FAKE_PSQL_CORPUS_TABLE", "tqhnsw_real_test_corpus")
    if fallback_ef and ef == int(fallback_ef):
        return (
            "Limit\\n"
            "  ->  Sort\\n"
            f"        Sort Key: (({corpus_table}.embedding <#> '{{0.1,0.2,0.3,0.4}}'::real[]))\\n"
            f"        ->  Seq Scan on {corpus_table}"
        )
    return (
        "Limit\\n"
        f"  ->  Index Scan using {expected_index} on {corpus_table}\\n"
        "        Order By: (embedding <#> '{0.1,0.2,0.3,0.4}'::real[])"
    )


sql = load_sql(sys.argv[1:])
normalized = " ".join(sql.split())
fallback_ef = os.environ.get("TQV_FAKE_PSQL_FALLBACK_EF")
expected_index = os.environ.get("TQV_FAKE_PSQL_EXPECTED_INDEX", "tqhnsw_real_test_m8_idx")
corpus_table = os.environ.get("TQV_FAKE_PSQL_CORPUS_TABLE", "tqhnsw_real_test_corpus")
query_table = os.environ.get("TQV_FAKE_PSQL_QUERY_TABLE", "tqhnsw_real_test_queries")
statements = [stmt.strip() for stmt in sql.split(";") if stmt.strip()]

if len(statements) > 1:
    current_ef = None
    outputs = []
    for stmt in statements:
        stmt = strip_meta_commands(stmt)
        if not stmt:
            continue
        normalized_stmt = " ".join(stmt.split())
        if normalized_stmt.startswith("SET tqhnsw.ef_search ="):
            current_ef = extract_ef(stmt)
        elif "EXPLAIN (ANALYZE, TIMING, FORMAT JSON)" in normalized_stmt:
            ef = current_ef or 0
            log_event("measure", ef)
            outputs.append(json.dumps([{"Execution Time": float(100 + ef)}]))
        elif normalized_stmt.startswith("EXPLAIN") and f"SELECT id FROM {corpus_table}" in normalized_stmt:
            ef = current_ef or 0
            log_event("plan", ef)
            outputs.append(plan_output(expected_index, ef, fallback_ef))
        elif f"SELECT id FROM {corpus_table}" in normalized_stmt:
            ef = current_ef or 0
            log_event("warmup", ef)
            outputs.append("1")
        else:
            print(f"unhandled fake psql SQL statement: {normalized_stmt}", file=sys.stderr)
            sys.exit(1)
    print("\\n".join(outputs))
    sys.exit(0)

if "SHOW shared_buffers" in normalized:
    print("128MB")
elif "SHOW work_mem" in normalized:
    print("4MB")
elif "SHOW max_parallel_workers_per_gather" in normalized:
    print("2")
elif f"SELECT count(*) FROM {query_table};" in normalized:
    print("1")
elif f"SELECT source FROM {query_table} ORDER BY id LIMIT 1;" in normalized:
    print("{0.1,0.2,0.3,0.4}")
elif f"SELECT source FROM {query_table} ORDER BY id LIMIT 1" in normalized:
    print("{0.1,0.2,0.3,0.4}")
elif f"SELECT source FROM {query_table} ORDER BY id;" in normalized:
    print("{0.1,0.2,0.3,0.4}")
elif f"SELECT source FROM {query_table} ORDER BY id" in normalized:
    print("{0.1,0.2,0.3,0.4}")
elif f"SELECT to_regclass('{expected_index}') IS NOT NULL;" in normalized:
    print("t")
elif f"SELECT to_regclass('{expected_index}') IS NOT NULL" in normalized:
    print("t")
elif "EXPLAIN (ANALYZE, TIMING, FORMAT JSON)" in normalized:
    ef = extract_ef(sql) or 0
    log_event("measure", ef)
    print(json.dumps([{"Execution Time": float(100 + ef)}]))
elif "EXPLAIN" in normalized:
    ef = extract_ef(sql) or 0
    log_event("plan", ef)
    print(plan_output(expected_index, ef, fallback_ef))
elif f"SELECT id FROM {corpus_table}" in normalized:
    ef = extract_ef(sql) or 0
    log_event("warmup", ef)
    print("1")
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

    def _run_verified(
        self,
        *,
        ef_search: str,
        fallback_ef: str | None,
        warmup_passes: str | None = None,
        log_file: Path | None = None,
        session_mode: str | None = None,
        corpus_table: str | None = None,
        query_table: str | None = None,
        index_name: str | None = None,
        storage_format: str | None = None,
    ) -> subprocess.CompletedProcess[str]:
        summary_file = self.tmp_dir / "summary.txt"
        env = os.environ.copy()
        env["TQV_PSQL_BIN"] = str(self.fake_psql)
        env["TQV_FAKE_PSQL_CORPUS_TABLE"] = corpus_table or "tqhnsw_real_test_corpus"
        env["TQV_FAKE_PSQL_QUERY_TABLE"] = query_table or "tqhnsw_real_test_queries"
        expected_index = index_name or "tqhnsw_real_test_m8_idx"
        if storage_format is not None and index_name is None:
            expected_index = f"tqhnsw_real_test_{storage_format}_m8_idx"
        env["TQV_FAKE_PSQL_EXPECTED_INDEX"] = expected_index
        if fallback_ef is not None:
            env["TQV_FAKE_PSQL_FALLBACK_EF"] = fallback_ef
        else:
            env.pop("TQV_FAKE_PSQL_FALLBACK_EF", None)
        if log_file is not None:
            env["TQV_FAKE_PSQL_LOG"] = str(log_file)
        else:
            env.pop("TQV_FAKE_PSQL_LOG", None)
        args = [
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
        ]
        if corpus_table is not None:
            args.extend(["--corpus-table", corpus_table])
        if query_table is not None:
            args.extend(["--query-table", query_table])
        if index_name is not None:
            args.extend(["--index-name", index_name])
        if storage_format is not None:
            args.extend(["--storage-format", storage_format])
        if warmup_passes is not None:
            args.extend(["--warmup-passes", warmup_passes])
        if session_mode is not None:
            args.extend(["--session-mode", session_mode])
        return subprocess.run(
            args,
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

    def test_verified_launcher_warms_each_cell_before_timing(self) -> None:
        log_file = self.tmp_dir / "events.log"
        result = self._run_verified(
            ef_search="40",
            fallback_ef=None,
            warmup_passes="2",
            log_file=log_file,
            session_mode="per-cell",
        )

        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn("[warmup] m=8 ef_search=40 pass 1/2", result.stderr)
        self.assertIn("[warmup] m=8 ef_search=40 pass 2/2", result.stderr)

        events = log_file.read_text(encoding="utf-8").splitlines()
        self.assertEqual(events, ["plan:40", "warmup:40", "warmup:40", "measure:40"])

    def test_verified_launcher_supports_explicit_real_corpus_overrides(self) -> None:
        summary_file = self.tmp_dir / "summary.txt"
        result = self._run_verified(
            ef_search="64",
            fallback_ef=None,
            corpus_table="tqhnsw_real_test_grouped_corpus",
            query_table="tqhnsw_real_test_queries_50",
            index_name="tqhnsw_real_test_grouped_m8_idx",
        )

        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn(
            "[verified] requiring planner use tqhnsw_real_test_grouped_m8_idx for every measured cell",
            result.stderr,
        )
        self.assertIn(
            "[verified] planner uses tqhnsw_real_test_grouped_m8_idx at ef_search=64",
            result.stderr,
        )

        lines = summary_file.read_text(encoding="utf-8").splitlines()
        self.assertEqual(len(lines), 1, lines)
        self.assertIn("ef_search=64", lines[0])

    def test_verified_launcher_derives_explicit_storage_format_index_name(self) -> None:
        summary_file = self.tmp_dir / "summary.txt"
        result = self._run_verified(
            ef_search="64",
            fallback_ef=None,
            storage_format="pq_fastscan",
        )

        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn(
            "[verified] requiring planner use tqhnsw_real_test_pq_fastscan_m8_idx for every measured cell",
            result.stderr,
        )
        self.assertIn(
            "[verified] planner uses tqhnsw_real_test_pq_fastscan_m8_idx at ef_search=64",
            result.stderr,
        )

        lines = summary_file.read_text(encoding="utf-8").splitlines()
        self.assertEqual(len(lines), 1, lines)
        self.assertIn("ef_search=64", lines[0])


if __name__ == "__main__":
    unittest.main()
