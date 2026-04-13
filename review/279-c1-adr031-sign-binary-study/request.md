# Review Request: C1 ADR-031 Sign-Derived Binary Prefilter Study

## Context

Packet `278` closes the remaining scan-local zero-copy part-3 seam with another
small keep. That likely exhausts the clearly-defensible copy/payload reductions
in the current ordered-scan cache path.

Priority has now shifted away from `ADR-029`. The new direction is:

- `ADR-031` RaBitQ-style binary prefilter
- `ADR-030` grouped FastScan-style scoring

The cheapest next question is `ADR-031` validation step 1: can a sign-derived
binary code, computed from tqvector's existing `1536x4-bit` representation,
correlate strongly enough with the exact scorer to justify runtime integration?

## Problem

We do not yet have a repo-native study seam for the new binary-prefilter lane.
Before touching beam-search runtime, we need a reproducible way to answer:

1. how well sign-derived binary scores correlate with exact f32 scores on the
   real corpus
2. whether exact top-k remains captured inside conservative binary survivor
   sets
3. what the rough scoring cost looks like relative to the current exact scorer

Without that study, `ADR-031` remains a hand-wavy architecture idea.

## Planned Work

1. extend the existing `approx_score_study` seam so it can evaluate a
   sign-derived binary-prefilter mode on the no-QJL `1536x4-bit` production
   lane
2. report rank correlation, top-k overlap, and exact-top-k capture inside
   binary survivor sets on exported real-corpus vectors
3. report a microbenchmark for the binary-prefilter scoring path against the
   current exact scorer
4. keep this slice out of ordered-scan runtime; this packet is about validating
   `ADR-031`, not integrating it

## Exit Criteria

- the repo can run a real-corpus sign-derived binary-prefilter study for the
  no-QJL `1536x4-bit` lane without touching scan execution
- the packet records whether `ADR-031` sign-derived filtering is promising
  enough to justify runtime integration ahead of `ADR-030`
- the required checkpoint gate is green:
  - `cargo test`
  - `PGRX_HOME=/tmp/tqvector_pgrx_home cargo pgrx test pg17`
  - `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`
