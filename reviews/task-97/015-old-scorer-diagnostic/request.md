# Task 97 Packet 015: Old QJL Scorer Diagnostic

This packet answers the remaining packet 004 F1 question directly:
the old pre-`b0efa19d9` multi-accumulator AVX2/FMA per-candidate QJL
scorer was faster, but it failed the ADR-076 tolerance pair at the
Task 97 production QJL fixture (`dim=1024,bits=4,seed=42`).

Code checkpoint: `7c4935018dd3564f3e4b9ffe13bbc9ef2980df3f`

## Change

- Added a `#[cfg(all(any(test, feature = "bench"), target_arch = "x86_64"))]`
  diagnostic reconstruction of the old pre-`b0efa19d9` multi-accumulator
  QJL AVX2 scorer.
- Added a Criterion row comparing the current production
  `score_ip_from_parts` path against that old diagnostic scorer at
  `d1024_b4`.
- Added the packet 005 metadata clarification: current QJL storage keeps
  residual signs in payload bytes; `CandidateMeta::GammaAndResidualSigns`
  currently contributes gamma only.

No production scorer optimization is included in this packet.

## Local Validation

- `cargo fmt --check`
- `git diff --check`
- `cargo test qjl_pre_b0efa19d9_multi_accum_tolerance_diagnostic --lib -- --nocapture --color never`
- `cargo test turboquant_qjl_batch_matches_pre_slice_scalar_reference_and_records_counters --lib -- --nocapture --color never`
- `cargo bench --features bench --bench quant_score 'd1024_b4' -- --sample-size 10 --warm-up-time 1 --measurement-time 2`

No GitHub CI or AWS runs were used.

## Results

Tolerance diagnostic:

- `dim=1024 bits=4 candidates=1000`
- `max_ulp=5920`
- `max_rel=4.106507404e-4`
- `violations=285`
- `worst_seed=924`

Old-vs-new Criterion rows:

- current `quant/score_ip_from_parts/d1024_b4/1024`: `[872.98 ns 877.79 ns 884.19 ns]`
- old diagnostic `quant/score_ip_from_parts/pre_b0efa19d9_multi_accum/d1024_b4`: `[257.62 ns 267.76 ns 276.49 ns]`

The old loop was about `3.28x` faster by median per-candidate latency
(`877.79 / 267.76`), but failed the required tolerance pair
(`max_ulp=5920`, `violations=285/1000`). That justifies keeping the
current scalar-order production scorer as the correctness anchor.

## Reviewer Notes

This packet is diagnostic evidence, not a qjl32 optimization slice.
Packet 011 feedback separately requires one qjl32 AVX2 candidate-parallel
transpose slice before accepting any AVX2 stop-condition disposition.
