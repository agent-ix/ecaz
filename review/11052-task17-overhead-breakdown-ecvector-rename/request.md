# Review Request: bench_tqvector_sql_overhead_breakdown — ecvector rename

Branch: `adr034-diskann-rebased`
Author: coder-2
Target:

- `scripts/bench_tqvector_sql_overhead_breakdown.sh`
- `scripts/tests/test_bench_tqvector_sql_overhead_breakdown.py`

## What this packet is

Completes the rename work flagged by packet 11049 by updating the last of
the three pre-rename scripts. The file names themselves still carry the
old `tqvector` word; that is intentional — see "Why not rename the file".

## What changed

### `scripts/bench_tqvector_sql_overhead_breakdown.sh`

Six doc-and-SQL occurrences updated:

- Header comment `# Break down tqvector SQL latency …` →
  `# Break down ecvector SQL latency …` (line 2).
- `--help` text describes the corpus table as "ecvector corpus table" and
  `embedding ecvector` (lines 21, 23), matching what
  `scripts/load_real_corpus.py` produces after packet 11049.
- `--help` mentions `encode_to_ecvector` for the `--bits` / `--seed` flag
  explanations (lines 27–28).
- Banner: `=== tqvector SQL overhead breakdown ===` →
  `=== ecvector SQL overhead breakdown ===` (line 266).
- Per-query encode timing SQL: `encode_to_tqvector(...)` →
  `encode_to_ecvector(...)` (line 440).

Behavior is unchanged; the script still emits the same event sequence
(`measure_sql`, `measure_encode`, `plan`, `profile`, `hot_path`,
`heap_fetch`, etc.) in the same order.

### `scripts/tests/test_bench_tqvector_sql_overhead_breakdown.py`

The fake-psql harness embedded in this test matches on `encode_to_tqvector(` /
`encode_to_ecvector(` substrings to classify emitted SQL. Updated four
match sites (two in the multi-statement branch and two in the
single-statement branch) from `encode_to_tqvector(` → `encode_to_ecvector(`,
plus the module docstring.

## Why not rename the file

`bench_tqvector_sql_overhead_breakdown.sh` / its `_scratch.sh` wrapper /
the companion `test_bench_tqvector_sql_overhead_breakdown.py` are
referenced from at least fourteen places across `review/`, `plan/`, and
`scripts/`, including historical review packets that must stay
word-for-word reproducible. A file rename would touch all of those for no
runtime benefit — the script still runs the same benchmark against the
same (ecvector-typed) corpus tables. The filename is historical branding,
not a schema object. Treating filenames as append-only keeps the review
trail auditable while the embedded SQL catches up with the type rename.

## Test evidence

```
$ python3 scripts/tests/test_bench_tqvector_sql_overhead_breakdown.py
...
----------------------------------------------------------------------
Ran 3 tests in 1.398s

OK
```

The regression tests exercise the launcher end-to-end with a fake psql,
so the `encode_to_ecvector(` update is covered: if the launcher emitted a
different string the "unhandled fake psql SQL statement" branch would
fail the run.

Remaining `tqvector` matches inside the script are the script's own
filename at line 11 (the `--help` example) — expected per "Why not
rename the file".

## Follow-ups

None from packet 11049's deferred list remain open. The three scripts
listed there are now all post-rename:

- `scripts/vacuum_concurrency_scratch.sh` — packet 11051.
- `scripts/bench_sql_latency.sh` — packet 11051.
- `scripts/bench_tqvector_sql_overhead_breakdown.sh` — this packet.

Next task-17 slice is DiskANN recall / latency evaluation itself, which
now has a post-rename corpus lane, an AM-generic loader, a documented
`--index-profile ec_diskann` path, and CLI verbs (`ecaz bench recall` /
`ecaz bench latency`) ready to drive.
