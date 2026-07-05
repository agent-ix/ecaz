#!/usr/bin/env python3
"""Merge cargo-llvm-cov summary.txt files by taking best per-file line coverage."""

from __future__ import annotations

import argparse
from dataclasses import dataclass
from pathlib import Path


@dataclass
class Row:
    path: str
    regions: int
    missed_regions: int
    region_cover: str
    functions: int
    missed_functions: int
    function_cover: str
    lines: int
    missed_lines: int
    line_cover: str
    branches: int
    missed_branches: int
    branch_cover: str

    @property
    def line_percent(self) -> float:
        return percent(self.line_cover)


def percent(raw: str) -> float:
    if raw == "-":
        return 0.0
    return float(raw.rstrip("%"))


def cover(total: int, missed: int) -> str:
    if total == 0:
        return "-"
    return f"{((total - missed) / total) * 100.0:.2f}%"


def canonical_path(raw: str) -> str:
    path = raw.removeprefix("/")
    marker = "/src/"
    if marker in path:
        path = path.split(marker, 1)[1]
    return path.removeprefix("src/")


def parse_summary(path: Path) -> tuple[list[str], dict[str, Row]]:
    header: list[str] = []
    rows: dict[str, Row] = {}
    for line in path.read_text(encoding="utf-8").splitlines():
        stripped = line.strip()
        if not stripped:
            continue
        if stripped.startswith("Filename") or stripped.startswith("---"):
            header.append(line)
            continue
        parts = stripped.split()
        if parts[0] == "TOTAL":
            continue
        if len(parts) < 13:
            continue
        row = Row(
            path=canonical_path(parts[0]),
            regions=int(parts[1]),
            missed_regions=int(parts[2]),
            region_cover=parts[3],
            functions=int(parts[4]),
            missed_functions=int(parts[5]),
            function_cover=parts[6],
            lines=int(parts[7]),
            missed_lines=int(parts[8]),
            line_cover=parts[9],
            branches=int(parts[10]),
            missed_branches=int(parts[11]),
            branch_cover=parts[12],
        )
        existing = rows.get(row.path)
        if existing is None or row.line_percent > existing.line_percent:
            rows[row.path] = row
    return header, rows


def format_row(path: str, row: Row) -> str:
    return (
        f"{path:<80} "
        f"{row.regions:>10} {row.missed_regions:>17} {row.region_cover:>9} "
        f"{row.functions:>11} {row.missed_functions:>17} {row.function_cover:>9} "
        f"{row.lines:>12} {row.missed_lines:>17} {row.line_cover:>9} "
        f"{row.branches:>11} {row.missed_branches:>17} {row.branch_cover:>9}"
    )


def total_row(rows: dict[str, Row]) -> Row:
    regions = sum(row.regions for row in rows.values())
    missed_regions = sum(row.missed_regions for row in rows.values())
    functions = sum(row.functions for row in rows.values())
    missed_functions = sum(row.missed_functions for row in rows.values())
    lines = sum(row.lines for row in rows.values())
    missed_lines = sum(row.missed_lines for row in rows.values())
    branches = sum(row.branches for row in rows.values())
    missed_branches = sum(row.missed_branches for row in rows.values())
    return Row(
        path="TOTAL",
        regions=regions,
        missed_regions=missed_regions,
        region_cover=cover(regions, missed_regions),
        functions=functions,
        missed_functions=missed_functions,
        function_cover=cover(functions, missed_functions),
        lines=lines,
        missed_lines=missed_lines,
        line_cover=cover(lines, missed_lines),
        branches=branches,
        missed_branches=missed_branches,
        branch_cover=cover(branches, missed_branches),
    )


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("summaries", nargs="+", type=Path)
    args = parser.parse_args()

    header: list[str] = []
    merged: dict[str, Row] = {}
    for summary in args.summaries:
        candidate_header, rows = parse_summary(summary)
        if not header:
            header = candidate_header
        for path, row in rows.items():
            existing = merged.get(path)
            if existing is None or row.line_percent > existing.line_percent:
                merged[path] = row

    if header:
        print(header[0])
        print(header[1])
    for path in sorted(merged):
        print(format_row(path, merged[path]))
    if header:
        print(header[1])
    print(format_row("TOTAL", total_row(merged)))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
