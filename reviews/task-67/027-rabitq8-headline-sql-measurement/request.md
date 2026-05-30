# Task 67 Packet 027: RaBitQ8 Headline SQL Measurement

## Summary

This packet addresses the packet-025 reviewer request for Slice J headline SQL
measurements of the four `rabitq8*` sidecar variants on the AWS Intel
`10k-intel` host. It uses packet-local `ecaz bench suite` configs and isolated
scalar/auto table prefixes.

Result: this fills the missing SQL evidence, but it does not close the 4x
headline gate. Auto-SIMD is approximately parity at `nprobe=16/32` and about
1.08-1.09x faster at `nprobe=64` on `total_bound_p50`.

## Result

Headline `total_bound_p50` speedup, scalar divided by auto:

| variant | nprobe=16 | nprobe=32 | nprobe=64 |
| --- | ---: | ---: | ---: |
| `rabitq8` | 0.92x | 0.90x | 1.09x |
| `rabitq8ls` | 0.94x | 0.95x | 1.09x |
| `rabitq8c3` | 0.93x | 0.94x | 1.09x |
| `rabitq8c4` | 0.92x | 0.93x | 1.08x |

Scalar `total_bound_p50`:

| variant | nprobe=16 | nprobe=32 | nprobe=64 |
| --- | ---: | ---: | ---: |
| `rabitq8` | 2.490 ms | 2.907 ms | 4.013 ms |
| `rabitq8ls` | 2.575 ms | 3.049 ms | 4.028 ms |
| `rabitq8c3` | 2.575 ms | 3.024 ms | 4.023 ms |
| `rabitq8c4` | 2.550 ms | 2.993 ms | 4.054 ms |

Auto `total_bound_p50`:

| variant | nprobe=16 | nprobe=32 | nprobe=64 |
| --- | ---: | ---: | ---: |
| `rabitq8` | 2.717 ms | 3.231 ms | 3.691 ms |
| `rabitq8ls` | 2.735 ms | 3.219 ms | 3.705 ms |
| `rabitq8c3` | 2.771 ms | 3.227 ms | 3.707 ms |
| `rabitq8c4` | 2.782 ms | 3.231 ms | 3.738 ms |

Auto recall is unchanged from scalar for every variant/nprobe row. The per-row
`sidecar_score_p50` values are 0.024-0.026 ms in the auto run, while
`candidate_sql_p50` is 2.419 / 2.912 / 3.406 ms for nprobe 16 / 32 / 64.
That means the headline SQL path is dominated by candidate SQL and sidecar I/O,
not the scorer kernel.

## Interpretation

This packet proves the bits=8 SQL headline gate is still open under the strict
Task 67 reading. Kernel-layer packets 020 and 023 remain valid and accepted,
but the Slice J headline SQL surface does not reach 4x across the four
`rabitq8*` variants.

The measurement also makes the next engineering direction clearer: further
RaBitQ8 kernel work is unlikely to move this particular headline metric unless
the benchmark isolates scoring more strongly. The current SQL headline is
bounded by candidate collection and sidecar access costs.

## Validation

- `ecaz bench suite audit` passed for scalar and auto configs.
- AWS `10k-intel` installed `b0c1403a2`.
- AWS scalar suite completed and synced artifacts from
  `s3://ecaz-cloud-10k-intel-f85c5b34/bench-artifacts/task67-rabitq8-headline-scalar/20260530T143737Z/`.
- AWS auto suite completed and synced artifacts from
  `s3://ecaz-cloud-10k-intel-f85c5b34/bench-artifacts/task67-rabitq8-headline-auto/20260530T143808Z/`.
- AWS `10k-intel` was paused after the run; see
  `artifacts/preflight/cloud-status-final.log`.

## Artifacts

- `artifacts/manifest.md`
- `artifacts/task67-rabitq8-headline-scalar-suite.json`
- `artifacts/task67-rabitq8-headline-auto-suite.json`
- `artifacts/scalar/results.jsonl`
- `artifacts/scalar/results-report.jsonl`
- `artifacts/scalar/suite-manifest.json`
- `artifacts/scalar/suite-run.log`
- `artifacts/scalar/cloud-bench-rabitq8-headline-scalar.log`
- `artifacts/scalar/sidecar-10k-rabitq8-headline-scalar.log`
- `artifacts/auto/results.jsonl`
- `artifacts/auto/results-report.jsonl`
- `artifacts/auto/suite-manifest.json`
- `artifacts/auto/suite-run.log`
- `artifacts/auto/cloud-bench-rabitq8-headline-auto.log`
- `artifacts/auto/sidecar-10k-rabitq8-headline-auto.log`
- `artifacts/local/suite-audit-scalar.log`
- `artifacts/local/suite-audit-auto.log`
- `artifacts/preflight/cloud-resume.log`
- `artifacts/preflight/cloud-install-b0c1403a2.log`
- `artifacts/preflight/cloud-pause.log`
- `artifacts/preflight/cloud-status-final.log`
