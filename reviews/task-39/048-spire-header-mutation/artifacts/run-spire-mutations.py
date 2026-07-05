#!/usr/bin/env python3
"""
Generic per-file mutation verification harness for SPIRE storage files
that the careful crate mounts via `include!` (and so cargo-mutants
cannot discover automatically).

Usage:
    python3 /tmp/run_spire_mutations.py <relative-source-path>
"""
from __future__ import annotations

import re
import shutil
import subprocess
import sys
from pathlib import Path
from typing import Optional, Tuple

ROOT = Path("/Users/peter/dev/tqvector")


def parse_args():
    if len(sys.argv) != 2:
        sys.exit("usage: run_spire_mutations.py <relative-source-path>")
    rel = sys.argv[1]
    src = ROOT / rel
    if not src.exists():
        sys.exit(f"missing source: {src}")
    return src


def setup(src: Path):
    name = src.stem
    backup = Path(f"/tmp/{name}_original.rs")
    if not backup.exists():
        shutil.copy(src, backup)
    artifacts = ROOT / f"reviews/task-39/048-spire-{name.replace('_','-')}-mutation/artifacts"
    # Walk reviews to find the matching packet by source file name.
    matching = sorted(
        (p.parent for p in (ROOT / "reviews/task-39").glob("*/artifacts/")),
        key=lambda p: p.name,
    )
    found = None
    for p in matching:
        if name.replace("_", "-") in p.name:
            found = p / "artifacts"
            break
    if found is None:
        sys.exit(f"could not find packet directory for {name}")
    enum_file = found / f"{name.replace('_', '-')}-mutants-enumerated.txt"
    if not enum_file.exists():
        # Try alternate stem
        candidates = list(found.glob("*-mutants-enumerated.txt"))
        if not candidates:
            sys.exit(f"missing enumeration file in {found}")
        enum_file = candidates[0]
    result_file = found / "manual-verification.log"
    return backup, found, enum_file, result_file


LINE_RE = re.compile(r"^([^:]+\.rs):(\d+):(\d+): (.+?)(?: in (\S+))?$")
OP_REPLACE_RE = re.compile(r"^replace (\S+) with (\S+)$")
DELETE_RE = re.compile(r"^delete (\S+)$")
BODY_REPLACE_RE = re.compile(r"^replace (\S+) -> (.+?) with (.+)$")


def parse_mutation(line: str):
    m = LINE_RE.match(line.strip())
    if not m:
        return None
    fname, lineno, col, body, func = m.group(1), m.group(2), m.group(3), m.group(4), m.group(5)
    lineno, col = int(lineno), int(col)
    op_m = OP_REPLACE_RE.match(body)
    if op_m:
        return ("op", lineno, col, op_m.group(1), op_m.group(2), func)
    del_m = DELETE_RE.match(body)
    if del_m:
        return ("delete", lineno, col, del_m.group(1), None, func)
    body_m = BODY_REPLACE_RE.match(body)
    if body_m:
        return ("body", lineno, col, body_m.group(1), body_m.group(3), func)
    return None


def apply_op(src: str, lineno: int, col: int, op1: str, op2: str):
    lines = src.splitlines(keepends=True)
    if lineno - 1 >= len(lines):
        return None
    line = lines[lineno - 1]
    pos = col - 1
    if not line[pos:].startswith(op1):
        return None
    new_line = line[:pos] + op2 + line[pos + len(op1):]
    lines[lineno - 1] = new_line
    return "".join(lines)


def apply_delete(src: str, lineno: int, col: int, tok: str):
    lines = src.splitlines(keepends=True)
    if lineno - 1 >= len(lines):
        return None
    line = lines[lineno - 1]
    pos = col - 1
    if not line[pos:].startswith(tok):
        return None
    new_line = line[:pos] + line[pos + len(tok):]
    lines[lineno - 1] = new_line
    return "".join(lines)


def apply_body(src: str, lineno: int, new_body: str):
    lines = src.splitlines(keepends=True)
    if lineno - 1 >= len(lines):
        return None
    offset = sum(len(l) for l in lines[: lineno - 1])
    brace = src.find("{", offset)
    if brace == -1:
        return None
    depth = 0
    i = brace
    in_str = False
    in_char = False
    in_line_comment = False
    in_block_comment = False
    esc = False
    end = None
    while i < len(src):
        c = src[i]
        nxt = src[i + 1] if i + 1 < len(src) else ""
        if in_line_comment:
            if c == "\n":
                in_line_comment = False
        elif in_block_comment:
            if c == "*" and nxt == "/":
                in_block_comment = False
                i += 1
        elif in_str:
            if esc:
                esc = False
            elif c == "\\":
                esc = True
            elif c == '"':
                in_str = False
        elif in_char:
            if esc:
                esc = False
            elif c == "\\":
                esc = True
            elif c == "'":
                in_char = False
        else:
            if c == "/" and nxt == "/":
                in_line_comment = True
                i += 1
            elif c == "/" and nxt == "*":
                in_block_comment = True
                i += 1
            elif c == '"':
                in_str = True
            elif c == "'":
                if nxt and (nxt.isalpha() or nxt == "_"):
                    pass
                else:
                    in_char = True
            elif c == "{":
                depth += 1
            elif c == "}":
                depth -= 1
                if depth == 0:
                    end = i
                    break
        i += 1
    if end is None:
        return None
    replacement = "{\n        " + new_body + "\n    }"
    return src[:brace] + replacement + src[end + 1:]


def run_full_suite() -> Tuple[bool, str]:
    proc = subprocess.run(
        [
            "cargo",
            "test",
            "--manifest-path",
            "hardening/careful/Cargo.toml",
            "--lib",
            "--quiet",
        ],
        cwd=str(ROOT),
        capture_output=True,
        text=True,
        timeout=180,
    )
    out = proc.stdout + proc.stderr
    if "test result: ok" in out and "test result: FAILED" not in out:
        m = re.search(r"test result: ok\. (\d+) passed", out)
        n = m.group(1) if m else "?"
        return True, f"{n} passed"
    fail = re.search(r"(\d+) failed", out)
    return False, f"{fail.group(1)} failed" if fail else f"exit={proc.returncode}"


def main():
    src = parse_args()
    backup, packet_dir, enum_file, result_file = setup(src)
    print(f"src={src} backup={backup} enum={enum_file} result={result_file}")

    mutations = []
    for ln in enum_file.read_text().splitlines():
        if not ln.strip():
            continue
        m = parse_mutation(ln)
        if not m:
            print(f"UNPARSED: {ln}")
            continue
        mutations.append((ln, m))

    print(f"Parsed {len(mutations)} mutations")
    result_file.write_text("")
    original = backup.read_text()

    for raw, mut in mutations:
        kind = mut[0]
        if kind == "op":
            _, lineno, col, op1, op2, fn = mut
            new_src = apply_op(original, lineno, col, op1, op2)
        elif kind == "delete":
            _, lineno, col, tok, _, fn = mut
            new_src = apply_delete(original, lineno, col, tok)
        else:
            _, lineno, col, fn_path, new_body, fn = mut
            new_src = apply_body(original, lineno, new_body)
        if new_src is None:
            msg = f"PATCH-FAIL  {raw}"
            print(msg)
            with result_file.open("a") as r:
                r.write(msg + "\n")
            continue
        src.write_text(new_src)
        passed, summary = run_full_suite()
        verdict = "MISSED" if passed else "KILLED"
        msg = f"{verdict:7} {raw}   [{summary}]"
        print(msg)
        with result_file.open("a") as r:
            r.write(msg + "\n")
        shutil.copy(backup, src)


if __name__ == "__main__":
    main()
