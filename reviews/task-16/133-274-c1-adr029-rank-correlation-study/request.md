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

## Outcome

Kept.

This slice adds a repo-native ADR-029 study seam without touching scan
execution:

- `src/quant/prod.rs`
- `src/bin/approx_score_study.rs`
- `src/lib.rs`

The quantizer now exposes a study-only int8 approximate scorer for the
`1536x4-bit`, QJL-disabled production lane:

- `prepare_ip_query_int8_approx_no_qjl_4bit(...)`
- `score_ip_from_parts_int8_approx_no_qjl_4bit(...)`

The new binary:

- `cargo run --release --bin approx_score_study`

can either generate a clustered `10K` / `1536` corpus or load exported real
corpus/query TSVs, score them with both the current exact scorer and the new
int8 approximate scorer, then report:

- full-rank Spearman / Pearson correlation
- exact-vs-approx top-k overlap
- exact top-k capture inside approximate survivor sets
- release-build per-score timing for exact vs approximate scoring

## Study readout

Synthetic release run:

- `cargo run --release --bin approx_score_study`

Observed on the default clustered `10K` / `20`-query study surface:

- `spearman_rho mean=0.9999 min=0.9999`
- `pearson_r mean=1.0000 min=0.9999`
- `top10_overlap mean=0.9650`
- `exact_top10_captured_by_approx_top20 mean=1.0000`
- `exact_top10_captured_by_approx_top50 mean=1.0000`
- `exact_top10_captured_by_approx_top100 mean=1.0000`
- `microbench exact_ns_per_score=1391.5`
- `microbench approx_ns_per_score=807.2`
- `speedup=1.72x`

Interpretation:

- the approximate scorer is **not** good enough to replace the exact scorer for
  final ranking
- it **is** strong enough to justify a two-stage filter experiment, because on
  this study surface the exact top-10 was always contained inside the
  approximate top-20
- even the current scalar int8 approximation is materially cheaper per score on
  the production lane

Real-corpus release run:

- `cargo run --release --bin approx_score_study -- --corpus-file /tmp/tqhnsw_real_10k_corpus.tsv --queries-file /tmp/tqhnsw_real_10k_queries.tsv --query-count 20`

Observed on the exported `tqhnsw_real_10k` source vectors:

- `spearman_rho mean=1.0000 min=1.0000`
- `pearson_r mean=1.0000 min=1.0000`
- `top10_overlap mean=0.9950`
- `exact_top10_captured_by_approx_top20 mean=1.0000`
- `exact_top10_captured_by_approx_top50 mean=1.0000`
- `exact_top10_captured_by_approx_top100 mean=1.0000`
- `exact_top10_captured_by_approx_top200 mean=1.0000`
- `exact_top10_captured_by_approx_top500 mean=1.0000`
- `exact_top10_captured_by_approx_top1000 mean=1.0000`
- `microbench exact_ns_per_score=1411.9`
- `microbench approx_ns_per_score=800.1`
- `speedup=1.76x`

Updated interpretation:

- the real corpus is even more favorable than the clustered synthetic proxy
- the approximate scorer still should **not** replace the exact scorer for final
  ranking, because `top10_overlap` is below `1.0`
- it is strong enough to justify a conservative two-stage filter experiment on
  live scan candidates, because the exact top-10 stayed fully contained inside
  the approximate top-20 for all `20` real queries

Remaining limitation:

- this slice now measures both synthetic and real-corpus full-rank correlation,
  but still not beam-search candidate traces captured from live ordered scans

## Decision

ADR-029 remains viable and is now evidence-backed enough to justify the next
runtime experiment on real scan candidates. The next slice should stay narrow:

1. keep the exact scorer as the final ranking path
2. add an approximate-survivor gate ahead of exact scoring in one local beam
   expansion seam
3. start with a conservative survivor budget informed by this study rather than
   an aggressive reject threshold

ADR-028 remains secondary unless this runtime integration fails to produce a
meaningful end-to-end latency win.

## Validation

- `cargo test`
- `PGRX_HOME=/tmp/tqvector_pgrx_home cargo pgrx test pg17`
- `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`
- `cargo run --release --bin approx_score_study`
- `cargo run --release --bin approx_score_study -- --corpus-file /tmp/tqhnsw_real_10k_corpus.tsv --queries-file /tmp/tqhnsw_real_10k_queries.tsv --query-count 20`

## Exit criteria

- the repo can run an exact-vs-approximate scoring study for the no-QJL
  `1536x4-bit` lane without touching scan execution
- the study reports correlation / overlap data that is sufficient to accept or
  reject ADR-029 as the next runtime experiment
- the required checkpoint gate is green:
  - `cargo test`
  - `PGRX_HOME=/tmp/tqvector_pgrx_home cargo pgrx test pg17`
  - `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`
