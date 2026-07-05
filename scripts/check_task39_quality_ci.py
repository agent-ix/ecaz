#!/usr/bin/env python3
"""Audit Task 39 test-quality CI wiring."""

from __future__ import annotations

import argparse
import re
import sys
from pathlib import Path


def job_block(workflow: str, job: str) -> str:
    pattern = re.compile(rf"(?ms)^  {re.escape(job)}:\n(?P<body>.*?)(?=^  [A-Za-z0-9_-]+:\n|\Z)")
    match = pattern.search(workflow)
    if match is None:
        raise ValueError(f"missing CI job: {job}")
    return match.group("body")


def require(text: str, needle: str, label: str) -> None:
    if needle not in text:
        raise ValueError(f"missing {label}: {needle}")


def require_regex(text: str, pattern: str, label: str) -> None:
    if re.search(pattern, text) is None:
        raise ValueError(f"missing {label}: {pattern}")


def audit(workflow: str) -> list[str]:
    require(workflow, "pull_request:", "pull_request trigger")
    require(workflow, "workflow_dispatch:", "manual trigger")
    require(workflow, 'cron: "37 9 * * *"', "nightly flake-hunt schedule")
    require(workflow, 'cron: "37 10 * * 1"', "weekly mutation schedule")

    coverage = job_block(workflow, "test-quality-coverage")
    require(coverage, "make coverage", "coverage lane")
    require(coverage, "make coverage-baseline-check", "coverage baseline completeness check")
    require(coverage, "scripts/check_coverage_delta.sh", "coverage delta gate")
    require(coverage, "target/quality/coverage", "coverage artifact upload")
    require(coverage, "fetch-depth: 0", "coverage PR diff history")

    mutants = job_block(workflow, "test-quality-mutants")
    require_regex(
        mutants,
        r"github\.event_name == 'workflow_dispatch'.*github\.event\.schedule == '37 10 \* \* 1'",
        "mutation manual/weekly condition",
    )
    require(mutants, "make mutants-full MUTANTS_JOBS=2", "weekly mutation sweep")
    require(mutants, "target/quality/mutants", "mutation artifact upload")

    flake = job_block(workflow, "test-quality-flake-hunt")
    require_regex(
        flake,
        r"github\.event_name == 'workflow_dispatch'.*github\.event\.schedule == '37 9 \* \* \*'",
        "flake-hunt manual/nightly condition",
    )
    require(flake, "make flake-hunt FLAKE_HUNT_SEEDS=8 FLAKE_HUNT_FUZZ_SECONDS=10", "nightly seed sweep")
    require(flake, "target/quality/flake-hunt", "flake-hunt artifact upload")

    return [
        "coverage: per-PR make coverage + baseline completeness + delta gate + artifact upload",
        "mutation: workflow_dispatch and weekly make mutants-full + artifact upload",
        "flake-hunt: workflow_dispatch and nightly 8-seed sweep + seed artifact upload",
    ]


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("workflow", nargs="?", type=Path, default=Path(".github/workflows/ci.yml"))
    args = parser.parse_args()

    try:
        findings = audit(args.workflow.read_text(encoding="utf-8"))
    except ValueError as error:
        print(f"Task 39 CI audit failed: {error}", file=sys.stderr)
        return 1

    print("Task 39 CI audit passed")
    for finding in findings:
        print(f"- {finding}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
