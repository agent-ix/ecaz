---
task: 50
packet: reviews/task-50/016-hnsw-parallel-build-dsm-accessors
head_sha: ce00e425bfa46ca4e3910eb847928e3fe5c2f4aa
code_commits:
  - 3c9a1223 Reduce HNSW parallel build DSM unsafe access
  - 8f552553 Fix HNSW DSM facade clippy mutation access
generated_at: 2026-05-20T00:40:29-07:00
storage_surface: n/a
lane: hnsw-parallel-build-structural
shared_table_surface: n/a
---

# Artifact Manifest

## block-count-planning-baseline.log

- Command: `rg -n 'src/am/ec_hnsw/build_parallel.rs' reviews/task-50/001-execution-planning/top-15-coverage-map.md`
- Result: Task 50 planning baseline records `src/am/ec_hnsw/build_parallel.rs` at 203 direct unsafe blocks.

## block-count-after.log

- Command: `make unsafe-block-count PATHS='src/am/ec_hnsw/build_parallel.rs'`
- Result: `139 src/am/ec_hnsw/build_parallel.rs`.
- Delta: 203 -> 139, down 64 blocks / 31.5%; Task 50 per-file target met.

## rustfmt-touched-check.log

- Command: `rustfmt --edition 2021 --check src/am/ec_hnsw/build_parallel.rs`
- Result: passed; emitted existing stable rustfmt warnings for unstable import options.

## cargo-check-pg18-bench.log

- Command: `cargo check --all-targets --no-default-features --features pg18,bench`
- Result: passed.
- Notes: existing warnings remain for `EC_PARALLEL_WORKER_SLOT_CLAIMED` in `src/am/common/parallel.rs` and SPIRE exports in `src/am/mod.rs`.

## cargo-test-hnsw-build-parallel.log

- Command: `cargo test build_parallel --lib --no-default-features --features pg18`
- Result: compiled, then failed at runtime before tests with `undefined symbol: CacheRegisterRelcacheCallback`.
- Interpretation: same local outside-PostgreSQL runtime-link limitation seen in prior Task 50 packets.

## git-diff-check.log

- Command: `git diff --check`
- Result: passed.

## cargo-fmt-check.log

- Command: `cargo fmt --all --check`
- Result: failed on existing repo-wide formatting backlog outside this packet's touched file.

## cargo-clippy-pg18.log

- Command: `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings`
- Result: failed on existing repo-wide clippy backlog.
- Packet-specific note: the intermediate `clippy::mut_from_ref` issue in `src/am/ec_hnsw/build_parallel.rs` was fixed by follow-up commit `8f552553`; regenerated log no longer reports `build_parallel.rs`.

