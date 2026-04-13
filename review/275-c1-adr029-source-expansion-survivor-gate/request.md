# Review Request: C1 ADR-029 Source Expansion Survivor Gate

## Context

Packet `274` established that the study-only int8 approximate scorer is
promising on both the clustered synthetic surface and the exported
`tqhnsw_real_10k` source vectors:

- synthetic `top10_overlap mean=0.9650`
- real-corpus `top10_overlap mean=0.9950`
- exact top-10 fully captured inside approximate top-20 on all `20` real
  queries
- scalar approximate scorer is about `1.7x` cheaper per score than the current
  exact scorer on the `1536x4-bit`, QJL-disabled production lane

The next question is no longer whether ADR-029 is plausible. The question is
whether a conservative approximate-first filter can reduce real warm ordered
scan latency without breaking runtime correctness.

## Problem

The hot live seam is the per-source successor scoring loop in
`src/am/scan.rs`, inside `cached_scan_successor_candidates_for_layer(...)`.
Today it:

1. loads each live neighbor element
2. exact-scores every candidate with `PreparedQuery`
3. pushes every scored candidate into the beam expansion result

That means ADR-029 has no runtime leverage yet. We need the narrowest possible
experiment that can measure whether an approximate-first pass saves enough exact
scoring to move the warm verified surface.

## Planned work

1. Add a tightly scoped runtime experiment in the source-expansion seam only.
2. Compute approximate scores for the source's candidate neighbors first.
3. Keep only a conservative survivor budget for exact rescoring and beam
   insertion.
4. Preserve correctness and fall back cleanly when the lane is not the
   `1536x4-bit`, QJL-disabled production path.
5. Validate on the real warm `10K` C1 surface and record whether the slice is a
   keep or a failed experiment.

## Exit criteria

- the approximate-first runtime experiment is isolated to one beam-expansion
  seam
- the exact scorer remains the final ranking path for surviving candidates
- the checkpoint gate is green:
  - `cargo test`
  - `PGRX_HOME=/tmp/tqvector_pgrx_home cargo pgrx test pg17`
  - `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`
- the packet records whether the experiment improved the verified warm real
  corpus surface or was discarded
