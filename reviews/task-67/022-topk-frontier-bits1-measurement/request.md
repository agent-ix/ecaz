# Task 67 Packet 022: Top-K Frontier Candidate Rejection

## Summary

This packet adds a narrow `ec_ivf` scan-path optimization:
`record_scored_posting_candidates` now skips dedup-map candidate work when a
full pre-rerank top-K frontier proves the posting score is strictly worse than
the current worst retained score.

The change preserves tie handling by rejecting only scores strictly worse than
the full frontier. It does not change SIMD kernels or RaBitQ math.

## Validation

- `cargo test -p ecaz candidate_top_k_rejects_only_scores_worse_than_full_worst`
  passed locally.
- `cargo fmt --check` passed; it emitted the repo's existing stable-rustfmt
  warnings for nightly-only import options.
- Both packet-local suite configs passed `ecaz bench suite audit`.
- AWS `10k-intel` install of `a0cb83cb3` passed.
- AWS scalar and auto scratch-SoA suites passed and synced artifacts.
- AWS `10k-intel` was paused after measurement; final status is
  `state: paused`, `~$0.00/hr running`.

## Results

Packet 017 no-scratch scalar baseline:

| nprobe | recall@10 | mean latency |
| --- | ---: | ---: |
| 16 | 0.9985 | 2.28 ms |
| 32 | 1.0000 | 3.70 ms |
| 64 | 1.0000 | 6.57 ms |

Packet 022 scratch-SoA auto lane after `a0cb83cb3`:

| nprobe | recall@10 | mean latency | speedup vs packet 017 scalar |
| --- | ---: | ---: | ---: |
| 16 | 0.9985 | 1.08 ms | 2.11x |
| 32 | 1.0000 | 1.47 ms | 2.52x |
| 64 | 1.0000 | 2.14 ms | 3.07x |

Packet 022 scratch-SoA scalar and auto:

| nprobe | scalar mean | auto mean | auto/scalar |
| --- | ---: | ---: | ---: |
| 16 | 1.07 ms | 1.08 ms | 0.99x |
| 32 | 1.48 ms | 1.47 ms | 1.01x |
| 64 | 2.20 ms | 2.14 ms | 1.03x |

Source artifacts:

- `artifacts/scalar/recall-10k-rabitq1-topk-frontier-scalar.log`
- `artifacts/scalar/latency-10k-rabitq1-topk-frontier-scalar.log`
- `artifacts/auto/recall-10k-rabitq1-topk-frontier-auto.log`
- `artifacts/auto/latency-10k-rabitq1-topk-frontier-auto.log`
- `artifacts/scalar/results.jsonl`
- `artifacts/auto/results.jsonl`

## Interpretation

The optimization improves the scratch-SoA bits=1 SQL lane and clears 3x at
nprobe 64 versus packet 017's scalar baseline. It does not make all measured
nprobe points reach 3x, and scratch-SoA scalar remains close to scratch-SoA
auto, so the remaining Task 67 gap is still scan-path overhead above the raw
RaBitQ kernels.

Packet 020 remains the kernel-throughput evidence: the AVX-512 bits=1 batch
kernel itself measured 5.59x scalar. This packet narrows but does not eliminate
the SQL-level gap.

Please review whether the top-K frontier rejection is acceptable and whether
the Task 67 performance gate should be judged against the kernel-throughput
evidence or the stricter total SQL wall-time interpretation.
