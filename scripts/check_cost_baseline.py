#!/usr/bin/env python3
"""Compare Task 47 planner-cost result rows against a committed baseline."""

from __future__ import annotations

import argparse
import json
import math
from pathlib import Path
from typing import Any


NUMERIC_FIELDS = (
    "modeled_startup_cost",
    "modeled_total_cost",
    "modeled_selectivity",
    "modeled_correlation",
    "index_pages",
    "reltuples",
)


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("results_jsonl", type=Path)
    parser.add_argument("baseline_json", type=Path)
    parser.add_argument(
        "--accept-drift",
        action="store_true",
        help="rewrite the baseline from the current result rows",
    )
    return parser.parse_args()


def load_rows(path: Path) -> dict[str, dict[str, str]]:
    rows: dict[str, dict[str, str]] = {}
    with path.open("r", encoding="utf-8") as fh:
        for line in fh:
            if not line.strip():
                continue
            record = json.loads(line)
            if record.get("metric") != "planner_cost":
                continue
            step = record["step"]
            rows[step] = record["values"]
    if not rows:
        raise SystemExit(f"no planner_cost rows in {path}")
    return rows


def numeric(value: str, *, field: str, step: str) -> float:
    try:
        parsed = float(value)
    except ValueError as exc:
        raise SystemExit(f"{step}.{field} is not numeric: {value!r}") from exc
    if not math.isfinite(parsed):
        raise SystemExit(f"{step}.{field} is not finite: {value!r}")
    return parsed


def baseline_from_rows(rows: dict[str, dict[str, str]]) -> dict[str, Any]:
    baseline_rows: dict[str, dict[str, float]] = {}
    for step, values in sorted(rows.items()):
        baseline_rows[step] = {
            field: numeric(values[field], field=field, step=step)
            for field in NUMERIC_FIELDS
            if field in values
        }
    return {
        "schema_version": 1,
        "description": "Task 47 small-fixture planner cost baseline",
        "drift": {"relative": 0.15, "absolute": 0.05},
        "fields": list(NUMERIC_FIELDS),
        "rows": baseline_rows,
    }


def compare_rows(
    actual_rows: dict[str, dict[str, str]], baseline: dict[str, Any]
) -> int:
    drift = baseline.get("drift", {})
    relative = float(drift.get("relative", 0.0))
    absolute = float(drift.get("absolute", 0.0))
    failures = 0
    for step, expected_values in baseline["rows"].items():
        actual_values = actual_rows.get(step)
        if actual_values is None:
            print(f"cost baseline missing result row: {step}")
            failures += 1
            continue
        for field, expected in expected_values.items():
            if field not in actual_values:
                print(f"cost baseline missing field: {step}.{field}")
                failures += 1
                continue
            actual = numeric(actual_values[field], field=field, step=step)
            expected = float(expected)
            allowed = max(absolute, abs(expected) * relative)
            delta = abs(actual - expected)
            if delta > allowed:
                print(
                    "cost drift: "
                    f"{step}.{field} actual={actual:.6g} "
                    f"baseline={expected:.6g} allowed={allowed:.6g}"
                )
                failures += 1
            else:
                print(
                    "cost ok: "
                    f"{step}.{field} actual={actual:.6g} "
                    f"baseline={expected:.6g}"
                )
    return failures


def main() -> int:
    args = parse_args()
    rows = load_rows(args.results_jsonl)
    if args.accept_drift:
        baseline = baseline_from_rows(rows)
        args.baseline_json.parent.mkdir(parents=True, exist_ok=True)
        args.baseline_json.write_text(
            json.dumps(baseline, indent=2, sort_keys=True) + "\n",
            encoding="utf-8",
        )
        print(f"wrote {args.baseline_json}")
        return 0

    baseline = json.loads(args.baseline_json.read_text(encoding="utf-8"))
    failures = compare_rows(rows, baseline)
    return 1 if failures else 0


if __name__ == "__main__":
    raise SystemExit(main())
