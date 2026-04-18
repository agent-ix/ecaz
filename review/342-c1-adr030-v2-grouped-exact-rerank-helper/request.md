# Review Request: C1 ADR-030 V2 Grouped Exact Rerank Helper

## Context

Packets `339` and `340` established the grouped-v2 cold rerank path:

- graph-side cold rerank tuple fetch
- scorer-local hot+cold payload composition

The grouped runtime still returned `ADR030_GROUPED_V2_SCAN_UNSUPPORTED` before exercising any real
score computation from that cold rerank payload.

## Problem

Without a real exact-rerank helper, the future grouped pipeline would still need to prove several
things at once:

1. grouped hot/cold payload composition
2. exact rerank score math from the cold scalar payload
3. grouped scorer integration

That is too much surface area for the next scoring step.

## Planned Slice

Add the exact-rerank scoring core behind the existing grouped-v2 runtime gate:

1. helper to score a merged grouped rerank payload with the current prepared query and quantizer
2. helper to read the scan state and invoke that score path
3. make the grouped scorer stub compute the exact rerank score before still returning the grouped-v2
   unsupported error
4. add unit coverage that the helper matches the production quantizer score path

This slice intentionally excludes:

- no grouped approximate scorer yet
- no gate lift
- no end-to-end grouped runtime execution

## Implementation

Updated:

- `src/am/scan.rs`

Concrete changes:

1. added `score_grouped_rerank_payload_result(...)`
2. added `score_grouped_rerank_payload_from_scan_state(...)`
3. changed `score_grouped_candidate_context(...)` to compute the exact rerank score through that
   helper before returning the existing grouped-v2 unsupported error
4. added a unit test proving the helper matches `ProdQuantizer::score_ip_from_parts(...)`

## Measurements

This packet is a scoring-core seam, so there are no new latency or recall measurements.

Known validation results for this attempt:

- focused validation:
  - `cargo test score_grouped_rerank_payload_result_matches_prod_quantizer_path --lib`: passed
  - `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`: passed
- full checkpoint:
  - `cargo test`: passed
  - `PGRX_HOME=/tmp/tqvector_pgrx_home cargo pgrx test pg17`: passed
  - `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`: passed

## Outcome

ADR-030 grouped-v2 now has the exact-rerank score core wired behind the grouped runtime gate.

What this de-risks:

1. the eventual tiny-rerank stage already has working score math on grouped-v2 cold payloads
2. future grouped runtime work can reuse the same exact-rerank helper instead of rebuilding score
   math at integration time
3. the remaining grouped runtime work is now more clearly “how do we approximate and dispatch,” not
   “can we score the cold rerank payload at all”

## Next Slice

The next narrow slice should finally move the approximate side forward:

1. grouped approximate score helper using grouped search codes
2. then end-to-end grouped candidate scoring with exact rerank comparison still behind the gate
