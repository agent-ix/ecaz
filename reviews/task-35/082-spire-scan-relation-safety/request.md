# Task 35 Packet 082: SPIRE Scan Relation Safety Comments

## Code Under Review

- Commit: `2254feb356fe32e15568da4a37dc24ed32138cdd`
- Files under review:
  - `src/am/ec_spire/scan/relation.rs`
  - `scripts/unsafe_comment_baseline.txt`

Note: this commit was produced during overlapping agent activity and also contains reviewer feedback at `reviews/task-35/072-spire-remote-dispatch-safety/feedback/2026-05-19-01-reviewer.md`. The code surface under review for this packet is limited to the two files above.

## Scope

This slice burns down the remaining unsafe-comment baseline for SPIRE scan relation helpers and heap rerank plumbing.

The added comments document safety boundaries for:

- PostgreSQL scan output writes for heap TID and ORDER BY score/null state;
- active epoch manifest, object manifest, placement directory, and local-store config tuple reads;
- ORDER BY ScanKey dereference and `real[]` datum decoding;
- heap relation and snapshot resolution used by heap rerank;
- candidate block prefetch, including PG18 read-stream lifetime and pinned-buffer handoff;
- heap tuple fetch, slot clearing, and indexed vector attribute materialization;
- varlena detoasting/copying before tuple slot reuse.

## Baseline Movement

- Global unsafe-comment baseline: `1799 -> 1768`
- `src/am/ec_spire/scan/relation.rs`: `31 -> 0`

## Validation

- `artifacts/unsafe-audit-after.log`: `bash scripts/check_unsafe_comments.sh` passed.
- `artifacts/unsafe-baseline-report-after.log`: baseline is `1768` entries across `51` files.
- `artifacts/scan-relation-baseline-after.log`: scan relation entries are `0`.
- `artifacts/git-diff-check.log`: `git diff --check` passed.
- `artifacts/cargo-check-pg18-bench.log`: `cargo check --all-targets --no-default-features --features pg18,bench` passed.

Known unrelated warnings remain:

- unused `EC_PARALLEL_WORKER_SLOT_CLAIMED` in `src/am/common/parallel.rs`;
- unused SPIRE imports/re-exports in `src/am/mod.rs`.

`cargo fmt --all` emitted the repo's existing stable-rustfmt warnings for unstable rustfmt options. It also touched `hardening/careful/src/lib.rs` and `src/quant/simd.rs`; those unrelated formatting changes were restored before the code landed.
