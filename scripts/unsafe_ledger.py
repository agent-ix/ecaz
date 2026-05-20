#!/usr/bin/env python3
"""Generate and check a Task 50 direct-unsafe ledger."""

from __future__ import annotations

import argparse
import hashlib
import json
import re
import sys
from dataclasses import asdict, dataclass
from pathlib import Path


UNSAFE_RE = re.compile(r"unsafe\s*\{")
ITEM_RE = re.compile(
    r"^\s*(?:pub(?:\([^)]*\))?\s+)?(?:async\s+)?(?:unsafe\s+)?"
    r"(?:extern\s+\"[^\"]+\"\s+)?(?:fn|mod|impl|trait|struct|enum)\s+"
    r"([A-Za-z0-9_]+)"
)


@dataclass
class UnsafeLedgerRow:
    id: str
    file: str
    line_at_capture: int
    column_at_capture: int
    enclosing_item: str
    category: str
    program: str
    disposition: str
    status: str
    residual_reason: str
    packet: str
    source_excerpt: str


def iter_rust_files(paths: list[Path]) -> list[Path]:
    files: list[Path] = []
    for path in paths:
        if path.is_file() and path.suffix == ".rs":
            files.append(path)
            continue
        if not path.is_dir():
            continue
        for child in path.rglob("*.rs"):
            parts = set(child.parts)
            if "target" in parts or ".git" in parts:
                continue
            files.append(child)
    return sorted(set(files))


def enclosing_item(lines: list[str], line_index: int) -> str:
    for idx in range(line_index, -1, -1):
        match = ITEM_RE.search(lines[idx])
        if match:
            return match.group(1)
    return "<module>"


def categorize(path: str, excerpt: str, item: str) -> tuple[str, str]:
    haystack = f"{path}\n{item}\n{excerpt}"

    if path.startswith("src/tests/") or path.startswith("hardening/"):
        return "tests-debug-hardening", "P13"
    if path.startswith("crates/") or path.startswith("vendor/"):
        return "crate-or-vendor-disposition", "P13"
    if "quant/" in path or any(
        token in haystack
        for token in (
            "_mm",
            "vld1",
            "vst1",
            "target_feature",
            "read_unaligned",
            "write_unaligned",
            "align_to",
        )
    ):
        return "simd-quant-raw-memory", "P12"
    if any(token in haystack for token in ("pgrx_extern_c_guard", "pg_am_callback")):
        return "ffi-callback-boundary", "P1"
    if any(
        token in haystack
        for token in (
            "PlannerInfo",
            "Query",
            "RawStmt",
            "List",
            "list_nth",
            "Node",
            "custom_scan",
            "dml_frontdoor",
        )
    ):
        return "planner-node-list-view", "P11"
    if any(
        token in haystack
        for token in ("Box::from_raw", "Box::into_raw", "palloc", "palloc0", "ptr::write")
    ):
        return "scan-opaque-raw-ownership", "P10"
    if any(
        token in haystack
        for token in ("read_stream_", "PrefetchBuffer", "per_buffer_data")
    ):
        return "read-stream-prefetch", "P9"
    if any(token in haystack for token in ("DSM", "dsm", "Atomic", "pg_atomic", "LWLock")):
        return "dsm-atomic-lock", "P8"
    if any(token in haystack for token in ("rd_options", "reloption", "CStr::from_ptr", "format_type_be")):
        return "reloptions-c-string", "P7"
    if any(
        token in haystack
        for token in ("Datum", "from_datum", "pg_detoast", "varlena", "from_raw_parts")
    ):
        return "datum-varlena-vector", "P6"
    if any(
        token in haystack
        for token in (
            "table_tuple_fetch_row_version",
            "slot_getsomeattrs",
            "tts_values",
            "tts_isnull",
            "ExecClearTuple",
        )
    ):
        return "heap-slot-source-scorer", "P5"
    if any(token in haystack for token in ("PageGetItem", "ItemId", "line", "tuple_bytes")):
        return "page-tuple-line-pointer", "P4"
    if any(
        token in haystack
        for token in (
            "ReadBuffer",
            "LockBuffer",
            "ReleaseBuffer",
            "BufferGetPage",
            "PageGet",
            "PageAddItem",
            "GenericXLog",
            "wal_txn",
            "RecordPageWithFreeSpace",
        )
    ):
        return "buffer-page-wal", "P3"
    if "pg_sys::" in haystack or "(*" in haystack:
        return "postgres-handle-view", "P2"
    return "unclassified-direct-unsafe", "P0"


def make_id(path: str, line_number: int, column: int, excerpt: str, ordinal: int) -> str:
    digest = hashlib.sha1(
        f"{path}\0{line_number}\0{column}\0{ordinal}\0{excerpt.strip()}".encode("utf-8")
    ).hexdigest()[:16]
    return f"unsafe-{digest}"


def generate_rows(paths: list[Path], packet: str) -> list[UnsafeLedgerRow]:
    rows: list[UnsafeLedgerRow] = []
    ordinal = 0
    for path in iter_rust_files(paths):
        display = path.as_posix()
        lines = path.read_text(encoding="utf-8").splitlines()
        for line_index, line in enumerate(lines):
            for match in UNSAFE_RE.finditer(line):
                ordinal += 1
                item = enclosing_item(lines, line_index)
                category, program = categorize(display, line, item)
                rows.append(
                    UnsafeLedgerRow(
                        id=make_id(display, line_index + 1, match.start() + 1, line, ordinal),
                        file=display,
                        line_at_capture=line_index + 1,
                        column_at_capture=match.start() + 1,
                        enclosing_item=item,
                        category=category,
                        program=program,
                        disposition="planned",
                        status="open",
                        residual_reason="",
                        packet=packet,
                        source_excerpt=line.strip(),
                    )
                )
    return rows


def write_jsonl(path: Path, rows: list[UnsafeLedgerRow]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    with path.open("w", encoding="utf-8") as handle:
        for row in rows:
            handle.write(json.dumps(asdict(row), sort_keys=True))
            handle.write("\n")


def load_jsonl(path: Path) -> list[dict[str, object]]:
    rows: list[dict[str, object]] = []
    with path.open("r", encoding="utf-8") as handle:
        for line_number, line in enumerate(handle, start=1):
            line = line.strip()
            if not line:
                continue
            try:
                value = json.loads(line)
            except json.JSONDecodeError as exc:
                raise SystemExit(f"{path}:{line_number}: invalid JSON: {exc}") from exc
            if not isinstance(value, dict) or "id" not in value:
                raise SystemExit(f"{path}:{line_number}: ledger row must be an object with id")
            rows.append(value)
    return rows


def command_generate(args: argparse.Namespace) -> int:
    rows = generate_rows([Path(path) for path in args.paths], args.packet)
    write_jsonl(Path(args.output), rows)
    print(f"wrote {len(rows)} unsafe ledger rows to {args.output}")
    return 0


def command_check(args: argparse.Namespace) -> int:
    current_rows = generate_rows([Path(path) for path in args.paths], packet="")
    current_ids = {row.id for row in current_rows}
    ledger_rows = load_jsonl(Path(args.ledger))
    ledger_by_id = {str(row["id"]): row for row in ledger_rows}

    unledgered = sorted(current_ids - set(ledger_by_id))
    stale_open = sorted(
        row_id
        for row_id, row in ledger_by_id.items()
        if row.get("status") not in {"removed", "residual", "superseded"}
        and row_id not in current_ids
    )

    if unledgered or stale_open:
        if unledgered:
            print(f"unledgered unsafe rows: {len(unledgered)}", file=sys.stderr)
            for row_id in unledgered[:20]:
                row = next(row for row in current_rows if row.id == row_id)
                print(
                    f"  {row.id} {row.file}:{row.line_at_capture}:{row.column_at_capture}",
                    file=sys.stderr,
                )
        if stale_open:
            print(f"stale open ledger rows: {len(stale_open)}", file=sys.stderr)
            for row_id in stale_open[:20]:
                row = ledger_by_id[row_id]
                print(
                    f"  {row_id} {row.get('file')}:{row.get('line_at_capture')}",
                    file=sys.stderr,
                )
        return 1

    print(f"ledger covers {len(current_rows)} current unsafe rows")
    return 0


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description=__doc__)
    subcommands = parser.add_subparsers(dest="command", required=True)

    generate = subcommands.add_parser("generate", help="write unsafe ledger JSONL")
    generate.add_argument("--output", required=True)
    generate.add_argument("--packet", default="")
    generate.add_argument("paths", nargs="*", default=["src"])
    generate.set_defaults(func=command_generate)

    check = subcommands.add_parser("check", help="verify current unsafe rows are ledgered")
    check.add_argument("--ledger", required=True)
    check.add_argument("paths", nargs="*", default=["src"])
    check.set_defaults(func=command_check)

    return parser


def main() -> int:
    parser = build_parser()
    args = parser.parse_args()
    return args.func(args)


if __name__ == "__main__":
    raise SystemExit(main())
