# Review Request: C1 ADR-029 Rank Correlation Study

## Context

After packets `265` through `273`, the warm verified `10K`, `m=8`,
`ef_search=40`, `per-cell`, `warmup=3` surface sits around:

- `p50=10.9ms`
- `mean=11.0ms`

The launcher-side hypotheses are closed:

- `EXPLAIN` overhead is negligible
- statement planning overhead is negligible
- scheduler micro-optimizations are low yield

The remaining question is whether ADR-029 can cut the cost of the no-QJL
`1536x4-bit` scoring lane enough to matter by using a compressed-domain
approximate scorer as a filter before the exact scorer.

## Problem

ADR-029 is high-upside, but it is still a research question. The current
repository does not have a narrow, reproducible way to answer the prerequisite
questions:

1. does an approximate score on the existing packed 4-bit representation
   correlate strongly enough with the exact score to be useful as a filter?
2. is the approximate score materially cheaper than the current exact scorer on
   the real production lane?

Without those answers, wiring ADR-029 into beam search would be premature.

## Planned work

1. Add a repo-native study seam for the `1536x4-bit`, QJL-disabled production
   lane that can compute both exact and approximate scores on the same encoded
   candidates.
2. Report score-order correlation and top-k overlap between the approximate and
   exact score orderings on a representative query/candidate sample.
3. Add a microbenchmark seam so the study also reports the approximate scorer's
   per-score cost against the current exact scorer.
4. Keep this slice out of scan execution. This packet is about deciding whether
   ADR-029 is promising enough to justify runtime integration.

## Exit criteria

- the repo can run an exact-vs-approximate scoring study for the no-QJL
  `1536x4-bit` lane without touching scan execution
- the study reports correlation / overlap data that is sufficient to accept or
  reject ADR-029 as the next runtime experiment
- the required checkpoint gate is green:
  - `cargo test`
  - `PGRX_HOME=/tmp/tqvector_pgrx_home cargo pgrx test pg17`
  - `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`
