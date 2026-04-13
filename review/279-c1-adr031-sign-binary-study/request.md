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

## Implementation

Completed work:

1. Extended `src/bin/approx_score_study.rs` with a `--study-mode binary-sign`
   path for the no-QJL `1536x4-bit` production lane.
2. Derived query binary codes from the sign of the rotated query dimensions.
3. Derived candidate binary codes from the sign of the existing 4-bit centroid
   values selected by each packed code nibble.
4. Reported:
   - rank correlation
   - top-k overlap
   - exact top-k capture inside binary survivor sets
   - microbenchmarks for both cached binary scoring and on-the-fly binary-code
     derivation

This stays entirely out of ordered-scan runtime.

## Outcome

Kept.

Real-corpus release run:

- `cargo run --release --bin approx_score_study -- --study-mode binary-sign --corpus-file /tmp/tqhnsw_real_10k_corpus.tsv --queries-file /tmp/tqhnsw_real_10k_queries.tsv --query-count 20`

Observed on exported `tqhnsw_real_10k` source vectors:

- `spearman_rho mean=0.9320 min=0.8514`
- `pearson_r mean=0.9468 min=0.8702`
- `top10_overlap mean=0.8550`
- `exact_top10_captured_by_approx_top20 mean=0.9650`
- `exact_top10_captured_by_approx_top50 mean=0.9850`
- `exact_top10_captured_by_approx_top100 mean=0.9950`
- `exact_top10_captured_by_approx_top200 mean=1.0000`
- `microbench exact_ns_per_score=1436.9`
- `microbench binary_cached_ns_per_score=9.8`
- `microbench binary_derived_ns_per_score=5820.4`
- `cached_speedup=146.23x`
- `derived_speedup=0.25x`

Interpretation:

- the sign-derived binary score is strong enough to stay alive as a real
  candidate filter on the current corpus
- it is **not** strong enough to use aggressively at small survivor budgets;
  top-50 survivors still miss some exact top-10 results
- it becomes credible at more conservative survivor budgets (`top100`/`top200`)
- the cached/stored binary-code path is the only viable runtime form; on-the-fly
  binary derivation is slower than the current exact scorer and should not be
  integrated as-is

## Decision

`ADR-031` is more promising than the current `ADR-030` grouped comparison seam
on tqvector's existing scalar-coded format, but only if binary codes are
cached or stored. The next runtime slice, if we choose to integrate it, should
assume one of:

1. scan-local cached binary codes
2. persisted binary sidecar codes

Do not wire an on-the-fly sign-derivation filter into beam search.

## Validation

- `cargo test`
- `PGRX_HOME=/tmp/tqvector_pgrx_home cargo pgrx test pg17`
- `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`
- `cargo run --release --bin approx_score_study -- --study-mode binary-sign --corpus-file /tmp/tqhnsw_real_10k_corpus.tsv --queries-file /tmp/tqhnsw_real_10k_queries.tsv --query-count 20`

## Exit Criteria

- the repo can run a real-corpus sign-derived binary-prefilter study for the
  no-QJL `1536x4-bit` lane without touching scan execution
- the packet records whether `ADR-031` sign-derived filtering is promising
  enough to justify runtime integration ahead of `ADR-030`
- the required checkpoint gate is green:
  - `cargo test`
  - `PGRX_HOME=/tmp/tqvector_pgrx_home cargo pgrx test pg17`
  - `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`
