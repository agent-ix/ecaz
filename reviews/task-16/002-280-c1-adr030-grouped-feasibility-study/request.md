# Review Request: C1 ADR-030 Grouped Scoring Feasibility Study

## Context

Packet `279` covers the ADR-031 sign-derived binary-prefilter study. The user
asked to study both `ADR-031` and `ADR-030` so they can be compared directly on
the same real-corpus surface.

`ADR-030` is higher-risk than `ADR-031` because its grouped-LUT/FastScan story
assumes a quantized layout closer to grouped PQ, while tqvector currently stores
per-dimension scalar `4-bit` codes with a shared global codebook.

## Problem

Before runtime integration, we need to answer the feasibility question:

1. does the grouped-scoring reinterpretation map cleanly onto tqvector's
   current scalar-coded format
2. if not exactly, is there still a grouped approximate scorer on the existing
   encoding that is strong enough to compare meaningfully against ADR-031

Without that check, ADR-030 risks being treated as "obviously applicable" when
it may actually require a stronger encoding/layout change first.

## Implementation

Completed work:

1. Extended `src/bin/approx_score_study.rs` with grouped comparison modes on
   the current no-QJL `1536x4-bit` lane:
   - `--study-mode grouped-f32`
   - `--study-mode grouped-u8`
2. Used a grouped mean-query surrogate over the current scalar-coded format so
   the study could measure whether a grouped approximate scorer is even
   directionally viable on tqvector's existing encoding.
3. Swept representative group sizes `8`, `16`, and `32`.
4. Compared the grouped scorer against the same real-corpus exact surface used
   by packet `279`.

This is deliberately a feasibility seam, not a runtime integration.

## Outcome

Kept as a feasibility study. Rejected as an immediate runtime direction on the
current scalar-coded format.

Real-corpus release runs:

- `cargo run --release --bin approx_score_study -- --study-mode grouped-f32 --group-size 8 --corpus-file /tmp/tqhnsw_real_10k_corpus.tsv --queries-file /tmp/tqhnsw_real_10k_queries.tsv --query-count 20`
- `cargo run --release --bin approx_score_study -- --study-mode grouped-f32 --group-size 16 --corpus-file /tmp/tqhnsw_real_10k_corpus.tsv --queries-file /tmp/tqhnsw_real_10k_queries.tsv --query-count 20`
- `cargo run --release --bin approx_score_study -- --study-mode grouped-f32 --group-size 32 --corpus-file /tmp/tqhnsw_real_10k_corpus.tsv --queries-file /tmp/tqhnsw_real_10k_queries.tsv --query-count 20`
- `cargo run --release --bin approx_score_study -- --study-mode grouped-u8 --group-size 16 --corpus-file /tmp/tqhnsw_real_10k_corpus.tsv --queries-file /tmp/tqhnsw_real_10k_queries.tsv --query-count 20`

Observed:

`group_size=8`
- `spearman_rho mean=0.7980 min=0.5015`
- `top10_overlap mean=0.7350`
- `exact_top10_captured_by_approx_top100 mean=0.9300`
- `grouped_f32_ns_per_score=1440.5`

`group_size=16`
- `spearman_rho mean=0.7024 min=0.4117`
- `top10_overlap mean=0.6500`
- `exact_top10_captured_by_approx_top100 mean=0.9000`
- `grouped_f32_ns_per_score=758.3`
- `grouped_u8_ns_per_score=1020.6`

`group_size=32`
- `spearman_rho mean=0.6249 min=0.3198`
- `top10_overlap mean=0.4700`
- `exact_top10_captured_by_approx_top100 mean=0.8350`
- `grouped_f32_ns_per_score=349.2`

Important comparison point:

- the `grouped-u8` result at `group_size=16` tracked the `grouped-f32` result
  almost exactly, so the main problem is **not** LUT quantization
- the main problem is the grouped reinterpretation itself on tqvector's current
  per-dimension scalar-coded format

## Decision

On tqvector's current encoding, `ADR-030` should not be treated as the next
runtime lane.

The grouped approximate scorer is weaker than `ADR-031` at every useful
survivor metric:

- lower correlation
- lower top-k capture
- only modest speedups unless the grouping becomes so coarse that quality drops
  further

So the study outcome is:

1. keep the feasibility packet
2. do **not** integrate this grouped surrogate into beam search
3. if `ADR-030` is pursued further, it likely needs a different encoding/layout
   closer to true grouped PQ rather than tqvector's current scalar-coded format

## Validation

- `cargo test`
- `PGRX_HOME=/tmp/tqvector_pgrx_home cargo pgrx test pg17`
- `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`
- the four real-corpus release study runs listed above

## Exit Criteria

- the packet records whether ADR-030 is directly compatible with tqvector's
  current scalar-coded format or only as an approximate feasibility seam
- the packet records real-corpus comparison data against the exact scorer
- the required checkpoint gate is green:
  - `cargo test`
  - `PGRX_HOME=/tmp/tqvector_pgrx_home cargo pgrx test pg17`
  - `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`
