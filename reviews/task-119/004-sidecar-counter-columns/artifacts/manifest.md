---
head_sha: 4614d4c0ef8dbf4b8072aaa60773325f4a74b7f5
task: task-119
packet: reviews/task-119/004-sidecar-counter-columns
date: 2026-06-24
---

# Task 119 Sidecar Counter Columns Manifest

## Scope

This packet supports Task 119 closeout evidence by making
`ecaz bench sidecar-rerank` report explicit counter columns instead of relying
on `candidate_count_*` as an overloaded proxy.

New sidecar-rerank table/result columns:

- `frontier_p50`
- `frontier_p95`
- `reranked_p50`
- `reranked_p95`
- `sidecar_reads_p50`
- `sidecar_reads_p95`
- `heap_source_reads_p50`
- `heap_source_reads_p95`
- `emitted_p50`
- `emitted_p95`

For `read_mode=free`, sidecar/source read counters are `0` because the harness
does not fetch payloads through PostgreSQL. For DB-backed modes,
`random-id` and `tid-sorted`, sidecar/source read counters are the number of
payload rows fetched from the sidecar table per query.

## Code

- Commit: `4614d4c0ef8dbf4b8072aaa60773325f4a74b7f5`
- File: `crates/ecaz-cli/src/commands/bench/sidecar_rerank.rs`

## Validation

| Artifact | Command | Result |
| --- | --- | --- |
| `cargo-test-ecaz-cli-sidecar.log` | `cargo test -p ecaz-cli sidecar -- --nocapture` | passed, 10 tests |
| `cargo-check-ecaz-cli.log` | `cargo check -p ecaz-cli` | passed, one pre-existing dead-code warning |

`cargo fmt --check` was also run, but it still fails on pre-existing unrelated
formatting drift in `src/am/ec_hnsw/scan.rs`. The Task 119 diff itself passed:

```sh
git diff --check -- crates/ecaz-cli/src/commands/bench/sidecar_rerank.rs reviews/task-119/004-sidecar-counter-columns
```

## Closeout Status

This does not close Task 119 by itself. It gives the sidecar matrix runner the
explicit per-representation counters needed for the next measurement packet.
