# Task 67 Packet 024: Scratch Score-Ordering Experiment

## Summary

This packet measures the `34da0492b` experiment that sorted each scratch-SoA
batch by exact score before candidate recording, so the top-K frontier would
fill with the best postings first.

The experiment did not help. It regressed latency versus packet 022 and still
does not satisfy the strict SQL wall-time interpretation of the bits=1
headline gate. The code was reverted in `db821441e`.

## Results

Packet 017 no-scratch scalar baseline:

| nprobe | recall@10 | mean latency |
| --- | ---: | ---: |
| 16 | 0.9985 | 2.28 ms |
| 32 | 1.0000 | 3.70 ms |
| 64 | 1.0000 | 6.57 ms |

Packet 022 best prior scratch-SoA auto lane:

| nprobe | recall@10 | mean latency | speedup vs packet 017 scalar |
| --- | ---: | ---: | ---: |
| 16 | 0.9985 | 1.08 ms | 2.11x |
| 32 | 1.0000 | 1.47 ms | 2.52x |
| 64 | 1.0000 | 2.14 ms | 3.07x |

Packet 024 score-order auto lane:

| nprobe | recall@10 | mean latency | speedup vs packet 017 scalar |
| --- | ---: | ---: | ---: |
| 16 | 0.9985 | 1.14 ms | 2.00x |
| 32 | 1.0000 | 1.64 ms | 2.26x |
| 64 | 1.0000 | 2.35 ms | 2.80x |

Packet 024 score-order scalar and auto:

| nprobe | scalar mean | auto mean | auto/scalar |
| --- | ---: | ---: | ---: |
| 16 | 1.18 ms | 1.14 ms | 1.04x |
| 32 | 1.60 ms | 1.64 ms | 0.98x |
| 64 | 2.23 ms | 2.35 ms | 0.95x |

Source artifacts:

- `artifacts/scalar/recall-10k-rabitq1-score-order-scalar.log`
- `artifacts/scalar/latency-10k-rabitq1-score-order-scalar.log`
- `artifacts/auto/recall-10k-rabitq1-score-order-auto.log`
- `artifacts/auto/latency-10k-rabitq1-score-order-auto.log`
- `artifacts/scalar/results.jsonl`
- `artifacts/auto/results.jsonl`

## Interpretation

Sorting the scratch batch costs more than it saves in dedup/top-K work on this
fixture. Packet 022 remains the better SQL-path result. The next useful Task 67
work should not continue this score-ordering approach; either improve scan
bookkeeping without per-batch sort overhead, or settle the remaining gate
interpretation using the accepted kernel evidence.

## Validation

- `cargo test -p ecaz posting_scratch_soa_sorts_score_indices_best_first_stably`
  passed before measurement.
- `cargo test -p ecaz candidate_top_k_rejects_only_scores_worse_than_full_worst`
  passed.
- `cargo fmt --check` passed; it emitted the repo's existing stable-rustfmt
  warnings for nightly-only import options.
- Both packet-local suite configs passed `ecaz bench suite audit`.
- AWS `10k-intel` install of `34da0492b` passed.
- AWS scalar and auto score-order scratch-SoA suites passed and synced
  artifacts.
- AWS `10k-intel` was paused after measurement; final status is
  `state: paused`, `~$0.00/hr running`.

Please review this as negative evidence for the score-ordering experiment.
