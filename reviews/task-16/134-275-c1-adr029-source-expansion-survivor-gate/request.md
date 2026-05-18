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

## Outcome

Discarded.

I prototyped an opt-in ADR-029 runtime gate in the per-source successor scoring
loop, measured it on the verified warm real-corpus seam, and then reverted the
code because it did not beat the disabled baseline.

The discarded prototype:

- added a session-only survivor-budget control for the experiment
- prepared the approximate query alongside the exact query only on the
  `1536x4-bit`, QJL-disabled lane
- approximate-scored uncached neighbors first inside
  `cached_scan_successor_candidates_for_layer(...)`
- exact-scored only the top approximate survivors for each expanded source

No code from that prototype is kept in the branch.

## Real-corpus readout

Verified warm baseline on the same build, with the experiment disabled:

- `scripts/bench_sql_latency_verified_scratch.sh --prefix tqhnsw_real_10k --m 8 --ef-search 40 --cache-state warm-after-prime3 --warmup-passes 3 --session-mode per-cell --timing-mode cached-plan`
- `p50=10.593ms`
- `p95=12.415ms`
- `p99=14.222ms`
- `mean=10.577ms`

Experiment run, conservative survivor budget `12`:

- `scripts/bench_sql_latency_verified_scratch.sh --prefix tqhnsw_real_10k --m 8 --ef-search 40 --cache-state warm-after-prime3 --warmup-passes 3 --session-mode per-cell --timing-mode cached-plan --approx-survivor-budget 12`
- `p50=11.071ms`
- `p95=15.805ms`
- `p99=20.344ms`
- `mean=11.431ms`

Experiment run, more aggressive survivor budget `8`:

- `scripts/bench_sql_latency_verified_scratch.sh --prefix tqhnsw_real_10k --m 8 --ef-search 40 --cache-state warm-after-prime3 --warmup-passes 3 --session-mode per-cell --timing-mode cached-plan --approx-survivor-budget 8`
- `p50=10.870ms`
- `p95=13.307ms`
- `p99=15.725ms`
- `mean=10.945ms`

## Interpretation

- the source-local survivor gate adds more overhead than it removes on the warm
  `10K` C1 seam
- the likely reason is shape mismatch: per-source layer-0 adjacency is already
  small, so the approximate pass adds extra work before exact-score savings can
  accumulate
- packet `274` still stands as evidence that ADR-029 is promising in principle,
  but this specific insertion point is low-yield

## Decision

Do not keep a per-source survivor gate in `cached_scan_successor_candidates_for_layer(...)`.

If ADR-029 is pursued again, the next runtime experiment should move to a seam
with a larger candidate pool than one source expansion, or pair the approximate
pass with a cheaper candidate-materialization path so the extra pass can pay for
itself.

## Validation

- `bash -n scripts/bench_sql_latency.sh`
- `cargo test prepared_query_cache_keeps_approx_payload_for_supported_lane_when_enabled -- --nocapture`
- verified warm real-corpus baseline cell
- verified warm real-corpus budget-12 cell
- verified warm real-corpus budget-8 cell

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
