#!/usr/bin/env python3
"""Task 41 static lint checks for raw PostgreSQL resource APIs."""

from __future__ import annotations

import argparse
import dataclasses
import re
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
SRC = ROOT / "src"


@dataclasses.dataclass(frozen=True)
class RawApiRule:
    name: str
    pattern: re.Pattern[str]
    allowed_paths: frozenset[str]


RAW_API_RULES = [
    RawApiRule(
        "buffer pin/lock APIs",
        re.compile(
            r"pg_sys::(?:ReadBufferExtended|LockBuffer|UnlockReleaseBuffer|ReleaseBuffer|"
            r"BufferGetPage|BufferGetPageSize|BufferGetBlockNumber)\b"
        ),
        frozenset({"src/storage/buffer_guard.rs"}),
    ),
    RawApiRule(
        "LWLock APIs",
        re.compile(r"pg_sys::(?:LWLockAcquire|LWLockRelease)\b"),
        frozenset({"src/storage/lock_guard.rs"}),
    ),
    RawApiRule(
        "snapshot registration APIs",
        re.compile(r"pg_sys::(?:RegisterSnapshot|UnregisterSnapshot)\b"),
        frozenset({"src/storage/snapshot_guard.rs"}),
    ),
    RawApiRule(
        "index relation open/close APIs",
        re.compile(r"pg_sys::(?:index_open|index_close)\b"),
        frozenset({"src/storage/relation_guard.rs"}),
    ),
    RawApiRule(
        "SPI tuptable release API",
        re.compile(r"pg_sys::SPI_freetuptable\b"),
        frozenset({"src/storage/spi_guard.rs"}),
    ),
]

READ_STREAM_NEXT_RE = re.compile(r"pg_sys::read_stream_next_buffer\b")
READ_STREAM_ADOPTION_RE = re.compile(
    r"(?:PinnedBufferGuard::from_pinned|LockedBufferGuard::lock_pinned|"
    r"buffer_guard::PinnedBufferGuard::from_pinned|buffer_guard::LockedBufferGuard::lock_pinned)"
)


def iter_rs_files() -> list[Path]:
    return sorted(path for path in SRC.rglob("*.rs") if path.is_file())


def line_number(text: str, offset: int) -> int:
    return text.count("\n", 0, offset) + 1


def check_raw_api_boundaries() -> list[str]:
    violations: list[str] = []
    for path in iter_rs_files():
        rel = path.relative_to(ROOT).as_posix()
        text = path.read_text()
        for rule in RAW_API_RULES:
            if rel in rule.allowed_paths:
                continue
            for match in rule.pattern.finditer(text):
                violations.append(
                    f"{rel}:{line_number(text, match.start())}: raw {rule.name} "
                    f"must stay in {', '.join(sorted(rule.allowed_paths))}"
                )
    return violations


def check_read_stream_adoption() -> list[str]:
    violations: list[str] = []
    for path in iter_rs_files():
        rel = path.relative_to(ROOT).as_posix()
        text = path.read_text()
        lines = text.splitlines()
        for match in READ_STREAM_NEXT_RE.finditer(text):
            line = line_number(text, match.start())
            window = "\n".join(lines[line - 1 : min(len(lines), line + 12)])
            if READ_STREAM_ADOPTION_RE.search(window):
                continue
            violations.append(
                f"{rel}:{line}: read_stream_next_buffer result must be adopted by "
                "PinnedBufferGuard or LockedBufferGuard in the local block"
            )
    return violations


def run_check() -> int:
    violations = check_raw_api_boundaries() + check_read_stream_adoption()
    if violations:
        for violation in violations:
            print(violation, file=sys.stderr)
        return 1
    print("ffi lint passed: raw PostgreSQL resource APIs are confined to guard modules")
    return 0


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--check", action="store_true", help="run lint checks")
    args = parser.parse_args()
    if args.check:
        return run_check()
    parser.print_help()
    return 1


if __name__ == "__main__":
    raise SystemExit(main())
