#!/usr/bin/env python3
"""Reject newly added machine-specific workstation paths without echoing them."""

from __future__ import annotations

import argparse
import re
import subprocess
import sys
from dataclasses import dataclass


HUNK = re.compile(r"^@@ -\d+(?:,\d+)? \+(\d+)(?:,\d+)? @@")
PATH_PATTERNS = (
    ("tilde workstation home path", re.compile(r"(?:^|(?<=\s)|(?<=[\"'`]))~/(?=\S)")),
    (
        "Unix workstation home path",
        re.compile(r"/(?:home|Users)/[^/\s\"'`]+(?=/|\s|[\"'`]|$)"),
    ),
    ("root workstation path", re.compile(r"/root(?=/|\s|[\"'`]|$)")),
    (
        "Windows workstation home path",
        re.compile(
            r"(?i)\b[A-Z]:[\\/](?:Users|Documents and Settings)[\\/]"
            r"[^\\/\s\"'`]+(?=[\\/]|\s|[\"'`]|$)"
        ),
    ),
)


@dataclass(frozen=True)
class AddedLine:
    path: str
    line_number: int
    text: str


@dataclass(frozen=True)
class Violation:
    path: str
    line_number: int
    label: str


def parse_added_lines(diff: str) -> list[AddedLine]:
    """Return added text lines from a zero-context unified diff."""

    path: str | None = None
    line_number: int | None = None
    added: list[AddedLine] = []

    for raw in diff.splitlines():
        if raw.startswith("+++ "):
            header_path = raw[4:].strip()
            path = header_path[2:] if header_path.startswith("b/") else header_path
            if path == "/dev/null":
                path = None
            continue

        match = HUNK.match(raw)
        if match:
            line_number = int(match.group(1))
            continue

        if path is None or line_number is None:
            continue
        if raw.startswith("+"):
            added.append(AddedLine(path, line_number, raw[1:]))
            line_number += 1
        elif raw.startswith("-") or raw.startswith("\\ No newline"):
            continue
        else:
            line_number += 1

    return added


def violations_for_diff(diff: str) -> list[Violation]:
    violations: list[Violation] = []
    for line in parse_added_lines(diff):
        for label, pattern in PATH_PATTERNS:
            if pattern.search(line.text):
                violations.append(Violation(line.path, line.line_number, label))
    return violations


def render_violations(violations: list[Violation]) -> str:
    lines = [
        "workstation-path check failed: newly added machine-specific paths found"
    ]
    lines.extend(
        f"{violation.path}:{violation.line_number}: {violation.label}"
        for violation in violations
    )
    lines.append("Use an environment variable, repository-relative path, or durable URL.")
    return "\n".join(lines)


def git_diff(diff_range: str) -> str:
    completed = subprocess.run(
        [
            "git",
            "diff",
            "--unified=0",
            "--no-color",
            "--no-ext-diff",
            diff_range,
            "--",
        ],
        check=True,
        stdout=subprocess.PIPE,
        text=True,
    )
    return completed.stdout


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--diff-range",
        required=True,
        help="Git revision range to inspect, for example BASE...HEAD",
    )
    args = parser.parse_args()

    violations = violations_for_diff(git_diff(args.diff_range))
    if violations:
        print(render_violations(violations), file=sys.stderr)
        return 1

    print("workstation-path check passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
