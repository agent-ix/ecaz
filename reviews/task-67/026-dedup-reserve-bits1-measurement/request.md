# Task 67 Packet 026: Dedup Reserve Experiment

## Summary

This packet measures the `candidate_dedup` reserve-cap experiment from
`1988ee51fb2b2527f2a6dcbcdc7de17bd0674406` on the AWS Intel `10k-intel`
host. The experiment capped the initial dedup map reservation for bounded
rerank scans to:

```text
min(candidate_bound, rerank_width * HEAPTID_INLINE_CAPACITY)
```

The result is negative. Latency regressed versus the current best packet-022
auto SIMD SQL result, so the experiment was reverted in
`521bbec4bbd58fd9809b18a7261e160728b6c04a`.

## Result

Packet 017 no-scratch scalar baseline:

| nprobe | scalar latency |
| --- | ---: |
| 16 | 2.28 ms |
| 32 | 3.70 ms |
| 64 | 6.57 ms |

Packet 022 current best auto SIMD result:

| nprobe | auto latency | speedup vs packet 017 |
| --- | ---: | ---: |
| 16 | 1.08 ms | 2.11x |
| 32 | 1.47 ms | 2.52x |
| 64 | 2.14 ms | 3.07x |

Packet 026 dedup-reserve experiment:

| nprobe | scalar latency | auto latency | auto/scalar | auto speedup vs packet 017 |
| --- | ---: | ---: | ---: | ---: |
| 16 | 1.33 ms | 1.31 ms | 1.02x | 1.74x |
| 32 | 1.76 ms | 1.73 ms | 1.02x | 2.14x |
| 64 | 2.65 ms | 2.60 ms | 1.02x | 2.53x |

Recall remained preserved for the auto lane:

| nprobe | recall@10 | mean q-time |
| --- | ---: | ---: |
| 16 | 0.9985 | 1.34 ms |
| 32 | 1.0000 | 1.62 ms |
| 64 | 1.0000 | 2.40 ms |

## Interpretation

The reduced initial reservation did not help the bounded rerank path. The most
likely explanation is that the smaller map capacity introduced growth or
allocator overhead that outweighed any benefit from avoiding the larger initial
reserve. The packet-022 top-K frontier result remains the best SQL-level
bits=1 measurement.

## Validation

- `cargo fmt --check` passed before the experiment commit.
- `cargo test -p ecaz --lib candidate_dedup_initial_capacity_caps_when_running_top_prunes` passed before the experiment commit.
- `cargo test -p ecaz --lib candidate_top_k_rejects_only_scores_worse_than_full_worst` passed before the experiment commit.
- `ecaz bench suite audit` passed for both packet-local suite configs.
- AWS `10k-intel` ran scalar and auto `ecaz bench suite` measurements with isolated table prefixes.
- AWS `10k-intel` is paused after the run; see `artifacts/preflight/cloud-status-after-pause.log`.

## Artifacts

- `artifacts/manifest.md`
- `artifacts/task67-dedup-reserve-bits1-scalar-suite.json`
- `artifacts/task67-dedup-reserve-bits1-auto-suite.json`
- `artifacts/scalar/results.jsonl`
- `artifacts/scalar/suite-manifest.json`
- `artifacts/scalar/suite-run.log`
- `artifacts/scalar/latency-10k-rabitq1-dedup-reserve-scalar.log`
- `artifacts/scalar/recall-10k-rabitq1-dedup-reserve-scalar.log`
- `artifacts/auto/results.jsonl`
- `artifacts/auto/suite-manifest.json`
- `artifacts/auto/suite-run.log`
- `artifacts/auto/latency-10k-rabitq1-dedup-reserve-auto.log`
- `artifacts/auto/recall-10k-rabitq1-dedup-reserve-auto.log`
- `artifacts/local/suite-audit-scalar.log`
- `artifacts/local/suite-audit-auto.log`
- `artifacts/preflight/cloud-status-after-pause.log`
