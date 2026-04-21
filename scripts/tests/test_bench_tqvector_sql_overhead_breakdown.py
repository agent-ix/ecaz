#!/usr/bin/env python3
"""Regression tests for the ecvector SQL overhead breakdown launcher."""

from __future__ import annotations

import os
from pathlib import Path
import subprocess
import tempfile
import unittest


REPO_ROOT = Path(__file__).resolve().parents[2]
BENCH_SCRIPT = REPO_ROOT / "scripts" / "bench_tqvector_sql_overhead_breakdown.sh"


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
    match = re.search(r"SET\\s+ec_hnsw\\.ef_search\\s*=\\s*(\\d+)", sql)
    if match:
        return int(match.group(1))
    return None


def strip_output_redirect(stmt: str) -> str:
    normalized = " ".join(stmt.split())
    if normalized == "\\o":
        return ""
    if normalized.startswith("\\o "):
        rest = normalized[3:].lstrip()
        if not rest:
            return ""
        first = rest.split(None, 1)[0].upper()
        if first in {"SET", "SELECT", "WITH", "EXPLAIN"}:
            return rest
        parts = rest.split(None, 1)
        if len(parts) == 2:
            return parts[1]
        return ""
    return normalized


def log_event(kind: str, ef: int | None) -> None:
    log_path = os.environ.get("TQV_FAKE_PSQL_LOG")
    if not log_path:
        return
    with open(log_path, "a", encoding="utf-8") as fh:
        fh.write(f"{kind}:{ef if ef is not None else 'none'}\\n")


def plan_output(expected_index: str, ef: int, fallback_ef: str | None) -> str:
    corpus_table = os.environ.get("TQV_FAKE_PSQL_CORPUS_TABLE", "ec_hnsw_real_test_corpus")
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
expected_index = os.environ.get("TQV_FAKE_PSQL_EXPECTED_INDEX", "ec_hnsw_real_test_m16_idx")
corpus_table = os.environ.get("TQV_FAKE_PSQL_CORPUS_TABLE", "ec_hnsw_real_test_corpus")
query_table = os.environ.get("TQV_FAKE_PSQL_QUERY_TABLE", "ec_hnsw_real_test_queries")
statements = [stmt.strip() for stmt in sql.split(";") if stmt.strip()]

if len(statements) > 1:
    current_ef = None
    outputs = []
    for stmt in statements:
        normalized_stmt = strip_output_redirect(stmt)
        if not normalized_stmt:
            continue
        if normalized_stmt.startswith("SET ec_hnsw.ef_search ="):
            current_ef = extract_ef(stmt)
        elif "EXPLAIN (ANALYZE, TIMING, FORMAT JSON)" in normalized_stmt:
            ef = current_ef or 0
            log_event("measure_sql", ef)
            outputs.append(json.dumps([{"Execution Time": float(100 + ef)}]))
        elif normalized_stmt.startswith("EXPLAIN") and f"SELECT id FROM {corpus_table}" in normalized_stmt:
            ef = current_ef or 0
            log_event("plan", ef)
            outputs.append(plan_output(expected_index, ef, fallback_ef))
        elif "encode_to_ecvector(" in normalized_stmt:
            ef = current_ef or 0
            log_event("measure_encode", ef)
            outputs.append("0.75")
        elif (
            "WITH started AS" in normalized_stmt
            and "encode_to_ecvector(" not in normalized_stmt
            and f"SELECT id FROM {corpus_table}" in normalized_stmt
        ):
            ef = current_ef or 0
            log_event("measure_sql_plain", ef)
            outputs.append(str(float(100 + ef)))
        elif "tests.ec_hnsw_debug_scan_profile_limited" in normalized_stmt:
            ef = current_ef or 0
            log_event("profile", ef)
            outputs.append("2000\\t500\\t2600\\t10\\t10")
        elif "tests.ec_hnsw_debug_scan_hot_path_profile" in normalized_stmt:
            ef = current_ef or 0
            log_event("hot_path", ef)
            outputs.append("1800\\t25\\t30\\t900\\t120\\t450")
        elif "tests.ec_hnsw_debug_scan_heap_fetch_profile" in normalized_stmt:
            ef = current_ef or 0
            log_event("heap_fetch", ef)
            outputs.append("2100\\t700\\t3100\\t700\\t200\\t10\\t10\\t10")
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
elif "to_regprocedure('tests.ec_hnsw_debug_scan_profile_limited(oid,real[],integer)')" in normalized:
    print("t")
elif "to_regprocedure('tests.ec_hnsw_debug_scan_hot_path_profile(oid,real[])')" in normalized:
    print("t")
elif "to_regprocedure('tests.ec_hnsw_debug_scan_heap_fetch_profile(oid,real[],integer,integer)')" in normalized:
    print("t")
elif "EXPLAIN (ANALYZE, TIMING, FORMAT JSON)" in normalized:
    ef = extract_ef(sql) or 0
    log_event("measure_sql", ef)
    print(json.dumps([{"Execution Time": float(100 + ef)}]))
elif "EXPLAIN" in normalized:
    ef = extract_ef(sql) or 0
    log_event("plan", ef)
    print(plan_output(expected_index, ef, fallback_ef))
elif "encode_to_ecvector(" in normalized:
    ef = extract_ef(sql) or 0
    log_event("measure_encode", ef)
    print("0.75")
elif (
    "WITH started AS" in normalized
    and "encode_to_ecvector(" not in normalized
    and f"SELECT id FROM {corpus_table}" in normalized
):
    ef = extract_ef(sql) or 0
    log_event("measure_sql_plain", ef)
    print(str(float(100 + ef)))
elif "tests.ec_hnsw_debug_scan_profile_limited" in normalized:
    ef = extract_ef(sql) or 0
    log_event("profile", ef)
    print("2000\\t500\\t2600\\t10\\t10")
elif "tests.ec_hnsw_debug_scan_hot_path_profile" in normalized:
    ef = extract_ef(sql) or 0
    log_event("hot_path", ef)
    print("1800\\t25\\t30\\t900\\t120\\t450")
elif "tests.ec_hnsw_debug_scan_heap_fetch_profile" in normalized:
    ef = extract_ef(sql) or 0
    log_event("heap_fetch", ef)
    print("2100\\t700\\t3100\\t700\\t200\\t10\\t10\\t10")
elif f"SELECT id FROM {corpus_table}" in normalized:
    ef = extract_ef(sql) or 0
    log_event("warmup", ef)
    print("1")
else:
    print(f"unhandled fake psql SQL: {normalized}", file=sys.stderr)
    sys.exit(1)
"""


class BenchTqvectorSqlOverheadBreakdownTests(unittest.TestCase):
    def setUp(self) -> None:
        self._tmp = tempfile.TemporaryDirectory()
        self.tmp_dir = Path(self._tmp.name)
        self.fake_psql = self.tmp_dir / "fake_psql.py"
        self.fake_psql.write_text(_FAKE_PSQL, encoding="utf-8")
        self.fake_psql.chmod(0o755)

    def tearDown(self) -> None:
        self._tmp.cleanup()

    def _run_bench(
        self,
        *,
        ef_search: str,
        fallback_ef: str | None,
        warmup_passes: str | None = None,
        log_file: Path | None = None,
        session_mode: str | None = None,
        timing_mode: str | None = None,
    ) -> subprocess.CompletedProcess[str]:
        summary_file = self.tmp_dir / "summary.txt"
        env = os.environ.copy()
        env["TQV_PSQL_BIN"] = str(self.fake_psql)
        env["TQV_FAKE_PSQL_CORPUS_TABLE"] = "ec_hnsw_real_test_corpus"
        env["TQV_FAKE_PSQL_QUERY_TABLE"] = "ec_hnsw_real_test_queries"
        env["TQV_FAKE_PSQL_EXPECTED_INDEX"] = "ec_hnsw_real_test_m16_idx"
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
            str(BENCH_SCRIPT),
            "--corpus-table",
            "ec_hnsw_real_test_corpus",
            "--query-table",
            "ec_hnsw_real_test_queries",
            "--index-name",
            "ec_hnsw_real_test_m16_idx",
            "--bits",
            "4",
            "--seed",
            "42",
            "--ef-search",
            ef_search,
            "--query-limit",
            "1",
            "--result-limit",
            "10",
            "--output",
            str(summary_file),
        ]
        if warmup_passes is not None:
            args.extend(["--warmup-passes", warmup_passes])
        if session_mode is not None:
            args.extend(["--session-mode", session_mode])
        if timing_mode is not None:
            args.extend(["--timing-mode", timing_mode])

        return subprocess.run(
            args,
            cwd=REPO_ROOT,
            env=env,
            text=True,
            capture_output=True,
            check=False,
        )

    def test_breakdown_aborts_before_timing_fallback_cell(self) -> None:
        result = self._run_bench(ef_search="40,128", fallback_ef="128")

        self.assertNotEqual(result.returncode, 0, result.stderr)
        self.assertIn(
            "planner verification failed for ec_hnsw_real_test_m16_idx at ef_search=128",
            result.stderr,
        )
        summary_lines = (self.tmp_dir / "summary.txt").read_text(encoding="utf-8").splitlines()
        self.assertEqual(len(summary_lines), 1, summary_lines)
        self.assertIn("ef_search=40", summary_lines[0])

    def test_breakdown_reports_summary_fields(self) -> None:
        log_file = self.tmp_dir / "events.log"
        result = self._run_bench(
            ef_search="40,64",
            fallback_ef=None,
            warmup_passes="1",
            log_file=log_file,
        )

        self.assertEqual(result.returncode, 0, result.stderr)
        summary_lines = (self.tmp_dir / "summary.txt").read_text(encoding="utf-8").splitlines()
        self.assertEqual(len(summary_lines), 2, summary_lines)
        self.assertIn("ef_search=40", summary_lines[0])
        self.assertIn("sql_mean=140.000ms", summary_lines[0])
        self.assertIn("encode_mean=0.750ms", summary_lines[0])
        self.assertIn("internal_total_mean=2.600ms", summary_lines[0])
        self.assertIn("executor_like_total_mean=3.100ms", summary_lines[0])
        self.assertIn("slot_fetch_total_mean=0.700ms", summary_lines[0])
        self.assertIn("projection_mean=0.200ms", summary_lines[0])
        self.assertIn("executor_like_over_internal=0.500ms", summary_lines[0])
        self.assertIn("residual_sql_over_internal=137.400ms", summary_lines[0])
        self.assertIn("residual_sql_over_executor_like=136.900ms", summary_lines[0])
        self.assertIn("residual_after_encode=136.650ms", summary_lines[0])

        events = log_file.read_text(encoding="utf-8").splitlines()
        self.assertIn("plan:40", events)
        self.assertIn("warmup:40", events)
        self.assertIn("measure_sql:40", events)
        self.assertIn("measure_encode:0", events)
        self.assertIn("profile:40", events)
        self.assertIn("hot_path:40", events)
        self.assertIn("heap_fetch:40", events)

    def test_breakdown_supports_per_cell_plain_server_sql_timing(self) -> None:
        log_file = self.tmp_dir / "plain_events.log"
        result = self._run_bench(
            ef_search="40",
            fallback_ef=None,
            log_file=log_file,
            session_mode="per-cell",
            timing_mode="plain-server",
        )

        self.assertEqual(result.returncode, 0, result.stderr)
        summary_lines = (self.tmp_dir / "summary.txt").read_text(encoding="utf-8").splitlines()
        self.assertEqual(len(summary_lines), 1, summary_lines)
        self.assertIn("ef_search=40", summary_lines[0])
        self.assertIn("sql_mean=140.000ms", summary_lines[0])
        self.assertIn("executor_like_total_mean=3.100ms", summary_lines[0])

        events = log_file.read_text(encoding="utf-8").splitlines()
        self.assertIn("plan:40", events)
        self.assertIn("measure_sql_plain:40", events)


if __name__ == "__main__":
    unittest.main()
