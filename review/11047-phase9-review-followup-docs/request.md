# Review Request: Phase 9 Review Follow-up Docs

Branch: `adr034-diskann-rebased`
Author: coder-2
Target:

- `src/am/ec_diskann/mod.rs`
- `src/am/ec_diskann/cost.rs`

## What this packet is

This is a narrow follow-up to packet 11045 after reviewer feedback landed in
`review/11045-phase9-planner-cost-activation/feedback/2026-04-20-01-reviewer.md`.

It addresses the two small in-scope items the reviewer called out as safe to
fold in immediately:

1. refresh the stale `src/am/ec_diskann/mod.rs` module header
2. document why planner metadata-read failure is allowed to `pgrx::error!`

It does **not** try to close the larger follow-up items from that review
packet:

- Recall@10 measurement capture still wants a separate artifact packet.
- Strict-clippy baseline cleanup still wants a dedicated hygiene packet.

## Why this slice

Before this change, `src/am/ec_diskann/mod.rs` still described the module as
the old Phase 1A skeleton where scan, insert, build, and vacuum callbacks were
not implemented. That was now false and actively misleading after Phase 9.

`src/am/ec_diskann/cost.rs` also used `pgrx::error!` on metadata decode
failure during planning without any local rationale. The behavior itself was
intentional, but the control-flow choice needed to be named so a later cleanup
does not silently soften index-corruption handling into planner gating.

## What changed

### `mod.rs`

Replaced the stale Phase 1A header with a current module description:

- `ec_diskann` is the Vamana-based secondary access method
- the module owns build, ordered scan, live insert, vacuum repair, and planner
  costing for grouped-PQ-backed `ecvector` indexes

### `cost.rs`

Added two short maintenance comments:

- `DISKANN_SINGLE_LAYER_TREE_HEIGHT = 1.0` now states why Vamana is modeled as
  a single-layer graph instead of an HNSW-style multi-level descent
- the metadata read path now documents why structural metadata corruption
  should fail loudly during planning instead of being hidden behind a gated
  fallback estimate

No runtime behavior changed in this packet.

## Boundary after this packet

This packet only resolves the narrow review-follow-up readability issues.

Still deferred:

- Recall@10 measurement capture packet
- strict-clippy baseline cleanup packet for existing `ec_diskann` warnings

## Verification

```text
cargo fmt -- src/am/ec_diskann/mod.rs src/am/ec_diskann/cost.rs
cargo build --lib
cargo clippy --lib --no-deps
cargo test --lib ec_diskann
cargo test
bash scripts/run_pgrx_pg17_test.sh
cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings
```

Observed:

- `cargo fmt -- src/am/ec_diskann/mod.rs src/am/ec_diskann/cost.rs` — passed
- `cargo build --lib` — passed
- `cargo clippy --lib --no-deps` — passed with only the known baseline
  warnings in untouched `reader.rs`, `scan.rs`, and `vamana.rs`
- `cargo test --lib ec_diskann` — passed with `143 passed`, `0 failed`
- `cargo test` — passed with `660 passed`, `0 failed`, `4 ignored`
- `bash scripts/run_pgrx_pg17_test.sh` — passed
- `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`
  — still fails only on the existing untouched strict-clippy baseline in:
  - `src/am/ec_diskann/reader.rs`
  - `src/am/ec_diskann/scan.rs`
  - `src/am/ec_diskann/vamana.rs`
  - `src/am/ec_diskann/vacuum.rs` tests

## Reviewer notes

- This packet intentionally does not move any Phase 9 planner behavior.
- The `pgrx::error!` planner path remains intentional because structural
  metadata failure means the index is broken, not merely unattractive to the
  planner.
- The header refresh is purely to keep the top-of-module description aligned
  with the now-live callback surface.
