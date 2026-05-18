#!/usr/bin/env python3
"""Inventory direct PostgreSQL FFI callbacks and pgrx-managed SQL entrypoints."""

from __future__ import annotations

import argparse
import dataclasses
import re
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
SRC = ROOT / "src"
INVENTORY = ROOT / "docs" / "ffi-inventory.md"


EXTERN_FN_RE = re.compile(
    r'(?m)^\s*(?:(?:pub(?:\([^)]*\))?)\s+)?(?:unsafe\s+)?extern\s+"C(?:-unwind)?"\s+fn\s+([A-Za-z_][A-Za-z0-9_]*)'
)
PGRX_ATTR_RE = re.compile(r"(?m)^\s*#\[(pg_extern|pg_operator|pg_aggregate)(?:[^\]]*)\]\s*$")
FN_NAME_RE = re.compile(r"\bfn\s+([A-Za-z_][A-Za-z0-9_]*)\b")


@dataclasses.dataclass(frozen=True)
class Entry:
    path: str
    line: int
    name: str
    kind: str
    status: str
    detail: str

    def key(self) -> tuple[str, int, str]:
        return (self.path, self.line, self.name)


def iter_rs_files() -> list[Path]:
    return sorted(path for path in SRC.rglob("*.rs") if path.is_file())


def line_number(text: str, offset: int) -> int:
    return text.count("\n", 0, offset) + 1


def attribute_block_before(lines: list[str], line_index: int) -> list[str]:
    attrs: list[str] = []
    idx = line_index - 1
    while idx >= 0:
        stripped = lines[idx].strip()
        if not stripped:
            idx -= 1
            continue
        if stripped.startswith("#["):
            attrs.append(stripped)
            idx -= 1
            continue
        break
    attrs.reverse()
    return attrs


def function_excerpt(lines: list[str], line_index: int, limit: int = 80) -> str:
    return "\n".join(lines[line_index : min(len(lines), line_index + limit)])


def cfg_detail(attrs: list[str]) -> str:
    cfgs = [attr for attr in attrs if attr.startswith("#[cfg")]
    return ", ".join(cfgs)


def classify_extern(path: str, name: str, attrs: list[str], excerpt: str) -> tuple[str, str]:
    if any("pg_guard" in attr for attr in attrs):
        return ("guarded", "`#[pg_guard]`")
    if "pgrx::pgrx_extern_c_guard" in excerpt:
        return ("guarded", "`pgrx::pgrx_extern_c_guard`")
    if "std::panic::catch_unwind" in excerpt:
        return ("guarded", "`std::panic::catch_unwind`")
    if name.startswith("pg_finfo_"):
        return (
            "documented exception",
            "metadata-only `pg_finfo_*` symbol returns a static `Pg_finfo_record`",
        )
    if path == "src/standalone_pg_backend_stubs.rs" and name == "ecaz_test_pg_backend_panic":
        return (
            "documented exception",
            "test-only backend stub intentionally raises a Rust panic for local unit tests",
        )
    return ("unguarded", "missing `#[pg_guard]`, `pgrx_extern_c_guard`, or `catch_unwind`")


def collect_extern_entries(path: Path) -> list[Entry]:
    text = path.read_text()
    rel = path.relative_to(ROOT).as_posix()
    lines = text.splitlines()
    entries: list[Entry] = []
    for match in EXTERN_FN_RE.finditer(text):
        line = line_number(text, match.start())
        line_index = line - 1
        attrs = attribute_block_before(lines, line_index)
        excerpt = function_excerpt(lines, line_index)
        status, detail = classify_extern(rel, match.group(1), attrs, excerpt)
        cfgs = cfg_detail(attrs)
        if cfgs:
            detail = f"{detail}; {cfgs}"
        entries.append(
            Entry(
                path=rel,
                line=line,
                name=match.group(1),
                kind="direct C ABI",
                status=status,
                detail=detail,
            )
        )
    return entries


def collect_pgrx_entries(path: Path) -> list[Entry]:
    text = path.read_text()
    rel = path.relative_to(ROOT).as_posix()
    entries: list[Entry] = []
    for match in PGRX_ATTR_RE.finditer(text):
        fn_match = FN_NAME_RE.search(text[match.end() :])
        if fn_match is None:
            continue
        entries.append(
            Entry(
                path=rel,
                line=line_number(text, match.start()),
                name=fn_match.group(1),
                kind=match.group(1),
                status="pgrx-managed",
                detail="SQL-callable pgrx entrypoint generated behind the pgrx guard boundary",
            )
        )
    return entries


def markdown_table(entries: list[Entry]) -> list[str]:
    rows = ["| Location | Function | Status | Detail |", "| --- | --- | --- | --- |"]
    for entry in entries:
        rows.append(
            f"| `{entry.path}:{entry.line}` | `{entry.name}` | {entry.status} | {entry.detail} |"
        )
    return rows


def render_inventory(extern_entries: list[Entry], pgrx_entries: list[Entry]) -> str:
    unguarded = [entry for entry in extern_entries if entry.status == "unguarded"]
    exceptions = [entry for entry in extern_entries if entry.status == "documented exception"]
    guarded = [entry for entry in extern_entries if entry.status == "guarded"]
    lines: list[str] = [
        "# FFI Inventory",
        "",
        "Generated by `python3 scripts/ffi_audit.py --write`.",
        "",
        "## Summary",
        "",
        f"- Direct C ABI functions: {len(extern_entries)}",
        f"- Guarded direct C ABI functions: {len(guarded)}",
        f"- Documented direct C ABI exceptions: {len(exceptions)}",
        f"- Unguarded direct C ABI functions: {len(unguarded)}",
        f"- pgrx-managed SQL entrypoints: {len(pgrx_entries)}",
        "",
        "The audit fails when the unguarded direct C ABI list is non-empty.",
        "Documented exceptions are metadata/test symbols that are not PostgreSQL",
        "executor, access method, planner, hook, relcache, DSM, or vacuum callbacks.",
        "",
        "## Unguarded Direct C ABI Functions",
        "",
    ]
    if unguarded:
        lines.extend(markdown_table(unguarded))
    else:
        lines.append("None.")
    lines.extend(["", "## Documented Exceptions", ""])
    if exceptions:
        lines.extend(markdown_table(exceptions))
    else:
        lines.append("None.")
    lines.extend(["", "## Direct C ABI Inventory", ""])
    lines.extend(markdown_table(extern_entries))
    lines.extend(["", "## pgrx-Managed SQL Entrypoints", ""])
    lines.extend(markdown_table(pgrx_entries))
    lines.append("")
    return "\n".join(lines)


def collect() -> tuple[list[Entry], list[Entry]]:
    extern_entries: list[Entry] = []
    pgrx_entries: list[Entry] = []
    for path in iter_rs_files():
        extern_entries.extend(collect_extern_entries(path))
        pgrx_entries.extend(collect_pgrx_entries(path))
    return sorted(extern_entries, key=Entry.key), sorted(pgrx_entries, key=Entry.key)


def main() -> int:
    parser = argparse.ArgumentParser()
    group = parser.add_mutually_exclusive_group()
    group.add_argument("--write", action="store_true", help="rewrite docs/ffi-inventory.md")
    group.add_argument("--check", action="store_true", help="verify inventory and fail on gaps")
    args = parser.parse_args()

    extern_entries, pgrx_entries = collect()
    inventory = render_inventory(extern_entries, pgrx_entries)
    unguarded = [entry for entry in extern_entries if entry.status == "unguarded"]

    if args.write:
        INVENTORY.write_text(inventory)
        if unguarded:
            for entry in unguarded:
                print(
                    f"unguarded FFI callback: {entry.path}:{entry.line} {entry.name}",
                    file=sys.stderr,
                )
            return 1
        print(f"wrote {INVENTORY.relative_to(ROOT)}")
        return 0

    if args.check:
        if not INVENTORY.exists():
            print(f"missing {INVENTORY.relative_to(ROOT)}; run with --write", file=sys.stderr)
            return 1
        current = INVENTORY.read_text()
        if current != inventory:
            print(
                f"{INVENTORY.relative_to(ROOT)} is stale; run: python3 scripts/ffi_audit.py --write",
                file=sys.stderr,
            )
            return 1
        if unguarded:
            for entry in unguarded:
                print(
                    f"unguarded FFI callback: {entry.path}:{entry.line} {entry.name}",
                    file=sys.stderr,
                )
            return 1
        print(
            f"ffi audit passed: {len(extern_entries)} direct C ABI functions, "
            f"{len(pgrx_entries)} pgrx-managed SQL entrypoints"
        )
        return 0

    print(inventory)
    return 0 if not unguarded else 1


if __name__ == "__main__":
    raise SystemExit(main())
