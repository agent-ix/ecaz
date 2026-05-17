# Review Request: C1 Ordered-Scan Runtime Fix

## Context

Branch:
- `main`

Prior packets:
- `review/246-c1-latency-launcher-plan-verification/request.md`
- `review/247-c1-real-corpus-latency-10k-verified-run/request.md`

Packet `246` added the planner-verified launcher so C1 would stop timing
`Sort -> Seq Scan`.

Packet `247` was opened before the first real verified `10k` run after planner
activation landed on `main`.

That first verified run still failed immediately at runtime:

```text
ERROR:  tqhnsw scan does not support index quals yet
```

The benchmark query shape is `ORDER BY embedding <#> ... LIMIT k` with no
`WHERE` clause, so this was not a real unsupported index-qual case. The actual
executor shape is a pure ordered scan with `nkeys == 0`, but PostgreSQL still
arrives at `amrescan` with an allocated key buffer. Our old guard rejected
that benign zero-qual buffer and prevented planner-routed tqhnsw scans from
executing at all.

## Scope

- `src/am/scan.rs`
- `src/am/scan_debug.rs`
- `src/am/mod.rs`
- `src/lib.rs`
- `scripts/pg17_scratch_psql.sh`
- `scripts/bench_sql_latency_scratch.sh`
- `scripts/bench_sql_latency_verified_scratch.sh`
- `scripts/load_real_corpus_scratch.sh`

## What Landed

### 1. `amrescan` now rejects real index quals, not zero-qual buffers

`src/am/scan.rs` now treats the executor contract correctly:

- `nkeys != 0` is still rejected with
  `tqhnsw scan does not support index quals yet`
- a non-null key pointer with `nkeys == 0` is now accepted

That preserves the current unsupported boundary while allowing planner-routed
ordered tqhnsw scans to execute.

### 2. New regression for the executor-style zero-qual path

`src/am/scan_debug.rs` adds a helper that calls `amrescan` with:

- `nkeys == 0`
- a deliberately non-null key buffer
- one valid `ORDER BY` query

`src/lib.rs` now verifies that this shape initializes scan state exactly like
the existing ordered-scan scaffold instead of panicking.

### 3. New end-to-end SQL execution regression

`src/lib.rs` also adds a planner-plus-runtime regression that:

- creates a small tqhnsw index
- forces seqscan off
- confirms `EXPLAIN` chooses `Index Scan`
- executes the ordered SQL query itself
- verifies the nearest row is returned first

This closes the gap where prior coverage only proved planner selection, not
successful execution through the ordered tqhnsw runtime path.

### 4. Scratch wrappers now auto-detect the active local pg17 socket

The scratch helpers now prefer `/tmp/tqvector_pgrx_home` when its socket is
live and otherwise fall back to `${HOME}/.pgrx`.

That keeps the repo-local `psql` and latency launchers aligned with the active
local cluster without requiring ad hoc env wiring for each run.

## Validation

Focused regressions:

- `cargo pgrx test pg17 test_tqhnsw_rescan_scaffold_accepts_unused_zero_key_buffer`
- `cargo pgrx test pg17 test_tqhnsw_sql_ordered_index_scan_executes`

Required checkpoint validation:

- `cargo test`
- `cargo pgrx test pg17`
- `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`

Shell sanity:

- `bash -n scripts/pg17_scratch_psql.sh`
- `bash -n scripts/bench_sql_latency_scratch.sh`
- `bash -n scripts/bench_sql_latency_verified_scratch.sh`
- `bash -n scripts/load_real_corpus_scratch.sh`

All green on this checkpoint.

## Current Status

At this checkpoint:

- planner-visible tqhnsw ordered scans execute locally again
- the verified latency launcher has not yet been rerun for the full real `10k`
  sweep on top of this fix
- packet `247` remains the measurement surface for that follow-on run

## Review Focus

- Is `nkeys == 0` the right exact boundary for the currently supported
  executor shape, while keeping all real index quals rejected?
- Is the new SQL regression sufficient to guard against future “planner picks
  the index but runtime still panics” failures?
- Is the scratch-wrapper socket auto-detection scoped narrowly enough for local
  operator use, or should it be centralized behind one shared helper later?
