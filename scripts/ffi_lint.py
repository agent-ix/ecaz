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
    RawApiRule(
        "tuple slot allocation/release APIs",
        re.compile(
            r"pg_sys::(?:MakeSingleTupleTableSlot|table_slot_create|ExecDropSingleTupleTableSlot)\b"
        ),
        frozenset({"src/storage/slot_guard.rs"}),
    ),
]

READ_STREAM_NEXT_RE = re.compile(r"pg_sys::read_stream_next_buffer\b")
READ_STREAM_ADOPTION_RE = re.compile(
    r"(?:PinnedBufferGuard::from_pinned|LockedBufferGuard::lock_pinned|"
    r"buffer_guard::PinnedBufferGuard::from_pinned|buffer_guard::LockedBufferGuard::lock_pinned)"
)


def iter_rs_files() -> list[Path]:
    return sorted(path for path in SRC.rglob("*.rs") if path.is_file())


def iter_rs_sources() -> list[tuple[str, str]]:
    return [(path.relative_to(ROOT).as_posix(), path.read_text()) for path in iter_rs_files()]


def line_number(text: str, offset: int) -> int:
    return text.count("\n", 0, offset) + 1


def check_raw_api_boundaries(sources: list[tuple[str, str]] | None = None) -> list[str]:
    violations: list[str] = []
    source_list = sources if sources is not None else iter_rs_sources()
    for rel, text in source_list:
        for rule in RAW_API_RULES:
            if rel in rule.allowed_paths:
                continue
            for match in rule.pattern.finditer(text):
                violations.append(
                    f"{rel}:{line_number(text, match.start())}: raw {rule.name} "
                    f"must stay in {', '.join(sorted(rule.allowed_paths))}"
                )
    return violations


def check_read_stream_adoption(sources: list[tuple[str, str]] | None = None) -> list[str]:
    violations: list[str] = []
    source_list = sources if sources is not None else iter_rs_sources()
    for rel, text in source_list:
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


def self_test() -> int:
    sources = [
        (
            "src/am/leaky_buffer.rs",
            "fn leaked_pin() {\n"
            "    unsafe { pg_sys::ReadBufferExtended(rel, fork, block, mode, strategy) };\n"
            "}\n",
        ),
        (
            "src/storage/buffer_guard.rs",
            "fn wrapper_only(buffer: pg_sys::Buffer) {\n"
            "    unsafe { pg_sys::ReleaseBuffer(buffer) };\n"
            "}\n",
        ),
        (
            "src/am/raw_lwlock.rs",
            "fn leaked_lock(lock: *mut pg_sys::LWLock) {\n"
            "    unsafe { pg_sys::LWLockAcquire(lock, pg_sys::LW_EXCLUSIVE as i32) };\n"
            "}\n",
        ),
        (
            "src/storage/lock_guard.rs",
            "fn wrapper_only(lock: *mut pg_sys::LWLock) {\n"
            "    unsafe { pg_sys::LWLockRelease(lock) };\n"
            "}\n",
        ),
        (
            "src/am/raw_spi.rs",
            "fn leaked_tuptable(table: *mut pg_sys::SPITupleTable) {\n"
            "    unsafe { pg_sys::SPI_freetuptable(table) };\n"
            "}\n",
        ),
        (
            "src/storage/spi_guard.rs",
            "fn wrapper_only(table: *mut pg_sys::SPITupleTable) {\n"
            "    unsafe { pg_sys::SPI_freetuptable(table) };\n"
            "}\n",
        ),
        (
            "src/am/raw_slot.rs",
            "fn leaked_slot(relation: pg_sys::Relation) {\n"
            "    unsafe { pg_sys::MakeSingleTupleTableSlot((*relation).rd_att, pg_sys::table_slot_callbacks(relation)) };\n"
            "}\n",
        ),
        (
            "src/storage/slot_guard.rs",
            "fn wrapper_only(relation: pg_sys::Relation) {\n"
            "    unsafe { pg_sys::ExecDropSingleTupleTableSlot(pg_sys::table_slot_create(relation, std::ptr::null_mut())) };\n"
            "}\n",
        ),
        (
            "src/am/read_stream_leak.rs",
            "fn leaked_stream(stream: *mut pg_sys::ReadStream) {\n"
            "    let _buffer = unsafe { pg_sys::read_stream_next_buffer(stream) };\n"
            "}\n",
        ),
        (
            "src/am/read_stream_adopted.rs",
            "fn adopted_stream(stream: *mut pg_sys::ReadStream) {\n"
            "    let buffer = unsafe { pg_sys::read_stream_next_buffer(stream) };\n"
            "    let _guard = PinnedBufferGuard::from_pinned(buffer);\n"
            "}\n",
        ),
    ]
    violations = check_raw_api_boundaries(sources) + check_read_stream_adoption(sources)
    expected_fragments = [
        "src/am/leaky_buffer.rs:2: raw buffer pin/lock APIs",
        "src/am/raw_lwlock.rs:2: raw LWLock APIs",
        "src/am/raw_spi.rs:2: raw SPI tuptable release API",
        "src/am/raw_slot.rs:2: raw tuple slot allocation/release APIs",
        "src/am/read_stream_leak.rs:2: read_stream_next_buffer result must be adopted",
    ]
    missing = [
        fragment
        for fragment in expected_fragments
        if not any(fragment in violation for violation in violations)
    ]
    unexpected_allowed = [
        violation
        for violation in violations
        if violation.startswith("src/storage/buffer_guard.rs:")
        or violation.startswith("src/storage/lock_guard.rs:")
        or violation.startswith("src/storage/spi_guard.rs:")
        or violation.startswith("src/storage/slot_guard.rs:")
        or violation.startswith("src/am/read_stream_adopted.rs:")
    ]
    if missing or unexpected_allowed or len(violations) != len(expected_fragments):
        print("ffi lint self-test failed", file=sys.stderr)
        for fragment in missing:
            print(f"missing expected violation: {fragment}", file=sys.stderr)
        for violation in unexpected_allowed:
            print(f"unexpected allowed-fixture violation: {violation}", file=sys.stderr)
        for violation in violations:
            print(f"observed violation: {violation}", file=sys.stderr)
        return 1
    print("ffi lint self-test passed")
    return 0


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--check", action="store_true", help="run lint checks")
    parser.add_argument("--self-test", action="store_true", help="run built-in verifier fixtures")
    args = parser.parse_args()
    if args.self_test:
        return self_test()
    if args.check:
        return run_check()
    parser.print_help()
    return 1


if __name__ == "__main__":
    raise SystemExit(main())
