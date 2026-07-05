#!/usr/bin/env python3
"""
Apply each cargo-mutants enumeration entry to leaf_v2_parts.rs, run focused
careful tests, record KILLED/MISSED with detail, revert.
"""
from __future__ import annotations

import re
import shutil
import subprocess
import sys
from pathlib import Path
from typing import Optional, Tuple

ROOT = Path("/Users/peter/dev/tqvector")
SRC = ROOT / "src/am/ec_spire/storage/leaf_v2_parts.rs"
BACKUP = Path("/tmp/leaf_v2_parts_original.rs")
ENUM = ROOT / "reviews/task-39/047-spire-leaf-v2-parts-mutation/artifacts/leaf-v2-parts-mutants-enumerated.txt"
RESULT = ROOT / "reviews/task-39/047-spire-leaf-v2-parts-mutation/artifacts/manual-verification.log"

LINE_RE = re.compile(
    r"^leaf_v2_parts\.rs:(\d+):(\d+): (.+?)(?: in (\S+))?$"
)
OP_REPLACE_RE = re.compile(r"^replace (\S+) with (\S+)$")
DELETE_RE = re.compile(r"^delete (\S+)$")
BODY_REPLACE_RE = re.compile(r"^replace (\S+) -> (.+?) with (.+)$")


def parse_mutation(line: str):
    m = LINE_RE.match(line.strip())
    if not m:
        return None
    lineno, col, body, func = m.group(1), m.group(2), m.group(3), m.group(4)
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


def apply_op(src: str, lineno: int, col: int, op1: str, op2: str) -> Optional[str]:
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


def apply_delete(src: str, lineno: int, col: int, tok: str) -> Optional[str]:
    lines = src.splitlines(keepends=True)
    if lineno - 1 >= len(lines):
        return None
    line = lines[lineno - 1]
    pos = col - 1
    if not line[pos:].startswith(tok):
        return None
    # consume optional trailing space
    skip = len(tok)
    new_line = line[:pos] + line[pos + skip:]
    lines[lineno - 1] = new_line
    return "".join(lines)


def apply_body(src: str, lineno: int, fn_path: str, new_body: str) -> Optional[str]:
    """Replace the body of the function whose signature starts at lineno.
    Find the first '{' after the signature start, then find its matching
    '}'. Replace everything between with `<NL>    NEW_BODY<NL>`."""
    lines = src.splitlines(keepends=True)
    if lineno - 1 >= len(lines):
        return None
    # Find first '{' at or after lineno
    offset = sum(len(l) for l in lines[: lineno - 1])
    brace = src.find("{", offset)
    if brace == -1:
        return None
    # Find matching '}': walk forward counting braces, ignoring strings.
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
                # could be lifetime — skip if followed by alphabetic+`
                if nxt and (nxt.isalpha() or nxt == "_"):
                    pass  # lifetime; ignore
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


def detect_function_test_filter(fn_path: str) -> str:
    """Pick a test filter based on which function the mutation lives in."""
    # All leaf_v2_parts mutations are best caught by the leaf v2 suite.
    # Run the broad miri_leaf_v2 / encode_decode set.
    return "leaf_v2"


def run_focused_test(test_filter: str) -> Tuple[bool, str]:
    """Return (passed, summary)."""
    proc = subprocess.run(
        [
            "cargo",
            "test",
            "--manifest-path",
            "hardening/careful/Cargo.toml",
            "--lib",
            "--quiet",
            "--",
            test_filter,
        ],
        cwd=str(ROOT),
        capture_output=True,
        text=True,
        timeout=120,
    )
    out = proc.stdout + proc.stderr
    # Look for "test result: ok"
    if "test result: ok" in out:
        # Extract pass count
        m = re.search(r"test result: ok\. (\d+) passed", out)
        n = m.group(1) if m else "?"
        return True, f"{n} passed"
    # Some failure
    fail_match = re.search(r"(\d+) failed", out)
    if fail_match:
        return False, f"{fail_match.group(1)} failed"
    if "error: test failed" in out:
        return False, "compile or test runner error"
    if proc.returncode != 0:
        return False, f"exit={proc.returncode}"
    return True, "no result line"


def main():
    if not BACKUP.exists():
        sys.exit("missing backup at /tmp/leaf_v2_parts_original.rs")

    mutations = []
    for ln in ENUM.read_text().splitlines():
        if not ln.strip():
            continue
        m = parse_mutation(ln)
        if not m:
            print(f"UNPARSED: {ln}")
            continue
        mutations.append((ln, m))

    print(f"Parsed {len(mutations)} mutations")
    RESULT.write_text("")

    for raw, mut in mutations:
        kind = mut[0]
        original = BACKUP.read_text()
        if kind == "op":
            _, lineno, col, op1, op2, fn = mut
            new_src = apply_op(original, lineno, col, op1, op2)
        elif kind == "delete":
            _, lineno, col, tok, _, fn = mut
            new_src = apply_delete(original, lineno, col, tok)
        else:  # body
            _, lineno, col, fn_path, new_body, fn = mut
            new_src = apply_body(original, lineno, fn_path, new_body)
        if new_src is None:
            msg = f"PATCH-FAIL  {raw}"
            print(msg)
            with RESULT.open("a") as r:
                r.write(msg + "\n")
            continue
        SRC.write_text(new_src)
        passed, summary = run_focused_test(detect_function_test_filter(fn or ""))
        verdict = "MISSED" if passed else "KILLED"
        msg = f"{verdict:7} {raw}   [{summary}]"
        print(msg)
        with RESULT.open("a") as r:
            r.write(msg + "\n")
        # Always revert
        shutil.copy(BACKUP, SRC)


if __name__ == "__main__":
    main()
