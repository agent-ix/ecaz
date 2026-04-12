#!/usr/bin/env python3
"""Regression tests for the inline bench_sql_latency summary helper."""

from __future__ import annotations

from pathlib import Path
import subprocess
import sys
import tempfile
import unittest


REPO_ROOT = Path(__file__).resolve().parents[2]
BENCH_SCRIPT = REPO_ROOT / "scripts" / "bench_sql_latency.sh"


def extract_summary_python() -> str:
    content = BENCH_SCRIPT.read_text(encoding="utf-8")
    marker = "<<'PY'\n"
    start = content.index(marker) + len(marker)
    end = content.index("\nPY", start)
    return content[start:end]


SUMMARY_PYTHON = extract_summary_python()


class BenchSqlLatencySummaryTests(unittest.TestCase):
    def _run_summary(
        self,
        *,
        timing_mode: str,
        samples: list[str],
    ) -> subprocess.CompletedProcess[str]:
        with tempfile.TemporaryDirectory() as tmpdir:
            results_file = Path(tmpdir) / "results.txt"
            results_file.write_text("\n".join(samples) + "\n", encoding="utf-8")
            return subprocess.run(
                [
                    sys.executable,
                    "-",
                    str(results_file),
                    "8",
                    "40",
                    "0.0",
                    "1.0",
                    "",
                    timing_mode,
                ],
                input=SUMMARY_PYTHON,
                text=True,
                capture_output=True,
                check=False,
            )

    def test_negative_server_side_sample_is_rejected(self) -> None:
        result = self._run_summary(
            timing_mode="cached-plan",
            samples=["11.0", "-799.355", "12.0"],
        )

        self.assertNotEqual(result.returncode, 0, result.stdout)
        self.assertIn("invalid negative per-query timings parsed", result.stderr)
        self.assertIn("cached-plan", result.stderr)

    def test_positive_server_side_samples_still_summarize(self) -> None:
        result = self._run_summary(
            timing_mode="plain-server",
            samples=["10.0", "12.0", "11.0"],
        )

        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn("mean=11.000ms", result.stdout)
        self.assertIn("min=10.000ms", result.stdout)


if __name__ == "__main__":
    unittest.main()
