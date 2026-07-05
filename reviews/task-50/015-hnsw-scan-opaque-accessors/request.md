# Task 50 Packet 015: HNSW Scan Opaque Accessors

## Code Under Review

- Commit: `f7c4c9b9ec7d4425ce9bbcc8589bae5fe96ec935`
- Task: `plan/tasks/50-unsafe-structural-reduction.md`
- Packet: `reviews/task-50/015-hnsw-scan-opaque-accessors/`

## Scope

This packet completes the direct Task 50 pass for
`src/am/ec_hnsw/scan.rs`, one of the top-15 residual unsafe modules from the
Task 50 planning map.

The change centralizes repeated scan-opaque and scan-owned raw pointer borrows
behind small local helpers, then consumes those helpers in HNSW scan scoring,
frontier, cache, prepared-query, and test-access paths. The underlying scan
state ownership model is unchanged: the same PostgreSQL scan descriptor owns the
opaque, and the same Box/Arc-backed scan slots remain responsible for lifetime.

This is a structural reduction in a hot scan module. No AM-level benchmark lane
is claimed by this packet; the required HNSW recall/latency comparison remains a
tranche-level closeout item after the remaining HNSW top-15 scan/build/insert/
vacuum/shared slices land, so the local run measures the final HNSW shape rather
than one partial accessor-only slice.

## Unsafe Block Count

Planning baseline:

`reviews/task-50/001-execution-planning/top-15-coverage-map.md`

After command:

`make unsafe-block-count PATHS='src/am/ec_hnsw/scan.rs'`

| File | Planning Baseline | Packet Result | Delta |
| --- | ---: | ---: | ---: |
| `src/am/ec_hnsw/scan.rs` | 226 | 157 | -69 (-30.5%) |

This satisfies the Task 50 per-module target of at least a 30% reduction from
the post-Task-35 starting count.

## Validation

- `make unsafe-block-count PATHS='src/am/ec_hnsw/scan.rs'`: `157`.
- `rustfmt --edition 2021 --check src/am/ec_hnsw/scan.rs`: passed with existing
  stable-rustfmt warnings about unstable config keys.
- `cargo check --all-targets --no-default-features --features pg18,bench`:
  passed with existing warnings.
- `cargo test ec_hnsw --lib --no-default-features --features pg18`: built the
  test binary, then failed to launch outside PostgreSQL with
  `undefined symbol: CacheRegisterRelcacheCallback`.
- `cargo fmt --all --check`: still reports pre-existing formatting drift in
  files outside this slice; touched-file rustfmt check passed.
- `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings`:
  still blocked by the existing repo-wide clippy backlog.
- `git diff --check f7c4c9b9^ f7c4c9b9 -- src/am/ec_hnsw/scan.rs`: passed.

## Artifacts

See `artifacts/manifest.md`.
