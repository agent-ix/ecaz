# Task 123 Review Request: Multi-Instance 100k Timeline Rerun

## Scope

This packet addresses the reviewer feedback that Task 123 did not have enough multi-instance testing. It adds a small suite-runner option to request production-read projection columns, then records 200-query local four-instance PG18 evidence for the two requested 100k cells:

- `n128 b4/tr50/f8`, nprobe `8,96`
- `n1024 b2/tr50/f8`, nprobe `8,64`

This is still local multi-instance, not true cross-network measurement, matching the narrowed scope for core algorithm validation.

## Code Change

Commit `641c853e792ee7c713049467c1e43b46a42481e1` adds:

- `bench_query_metric_projection_columns` to `spire-local-multinode` suite steps.
- `--bench-query-metric-projection-columns` to `dev spire-multicluster local-multinode-pg18`.

Validation:

- `cargo test -p ecaz-cli spire_local_multinode_step_expands_local_four_instance_lane -- --nocapture` passed.
- `cargo build -p ecaz-cli --bin ecaz` passed.

## Evidence Summary

Primary artifact metadata is in `artifacts/manifest.md`.

### n128 b4/tr50/f8, id-only, 200 queries

| mode | nprobe | p50 | p95 | recall@10 |
| --- | ---: | ---: | ---: | ---: |
| default | 8 | 662.821 ms | 923.969 ms | 0.9900 |
| default | 96 | 5408.521 ms | 5815.967 ms | 1.0000 |
| rowcap25k | 8 | 660.048 ms | 928.136 ms | 0.9900 |
| rowcap25k | 96 | 5409.689 ms | 5767.709 ms | 1.0000 |

Storage: total `1.9 GiB`; coordinator index `392.2 MiB`.

### n1024 b2/tr50/f8, id-only, 200 queries

| mode | nprobe | p50 | p95 | recall@10 |
| --- | ---: | ---: | ---: | ---: |
| default | 8 | 555.397 ms | 581.701 ms | 0.9290 |
| default | 64 | 770.595 ms | 860.296 ms | 1.0000 |
| rowcap25k | 8 | 557.193 ms | 582.105 ms | 0.9290 |
| rowcap25k | 64 | 766.879 ms | 845.695 ms | 1.0000 |

Storage: total `1.8 GiB`; coordinator index `246.1 MiB`.

## Realistic Projection Finding

The clean n1024 run with `id,source` projection failed before producing timing rows:

```text
ERROR: EcSpireDistributedScan production executor blocked: status remote_heap_resolution_failed, next_blocker remote_heap_resolution
```

That failure is preserved in `artifacts/n1024-b2-200q-source/bench-suite/suite-run.log` and `artifacts/n1024-b2-200q-source/coord-postgres.log`.

The completed performance evidence uses id-only projection so this packet answers the core routing/recall question without conflating it with the current realistic-payload projection failure.

## Notes

- No TSV corpus or assignment files remain in this packet.
- The packet is trimmed to the 200-query `*-200q-source/bench-suite/*-idonly*` artifacts and the failed `id,source` projection logs.
