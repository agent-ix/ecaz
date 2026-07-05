# Task 55 Packet 005 - AWS DiskANN Scan Optimization

Status: **proposed**

This packet covers the first AWS-backed DiskANN performance optimization after
the low-cost Graviton config audit.

## Code Change

Commit under review:

- `cbf037334ce0a9f499507d206049574b8278282e` - `Optimize DiskANN scan materialization`

The normal binary-sidecar scan path no longer materializes every DiskANN
data-page tuple into a `DataPageChain` during `ambeginscan`. Instead it reads
only visited graph nodes from the live relation through a `GraphReader`
abstraction. The grouped-PQ prefilter path keeps the materialized fallback
because it needs persisted codebooks.

The change also avoids a per-expansion neighbor-vector clone in greedy descent
by removing the picked entry from the candidate map.

## Validation

Local validation before AWS:

| Command | Result |
| --- | --- |
| `cargo test --lib ec_diskann::scan --no-run` | pass |
| `cargo check --all-targets --no-default-features --features pg18` | pass |
| `cargo build --release` | pass |

Plain `cargo test --lib ec_diskann::scan` was not used as evidence because the
test binary compiled but cannot execute outside PostgreSQL due PG FFI symbols
such as `LockBuffer`.

AWS validation used `ecaz bench suite` through `ecaz cloud bench`, not a
bespoke benchmark script. The 10k low-cost Graviton stack was left running for
follow-up cycles.

## Benchmark Evidence

Source-of-truth benchmark packets:

- Before/config audit: `benchmarks/task55-aws-diskann-lowcost-config-audit/`
- After/optimized: `benchmarks/task55-aws-diskann-lowcost-optimized/`
- Packet-local artifact manifest: `reviews/task-55/005-aws-diskann-scan-optimization/artifacts/manifest.md`

Both AWS suites completed with 21/21 steps succeeded.

## Result

The config audit showed the prior bad DiskANN shape was not a disabled planner
path or obviously bad graph config: the planner path was live, `pq_fastscan`
was active, storage was unchanged at `46.1 MiB` / `483.1 B` per row for 100k,
and recall was healthy.

The optimization removes the fixed per-scan full-index materialization cost.
On the 100k corpus, mean SQL latency moved from a flat `61.9-64.8 ms` band to
`1.72-10.6 ms` across `list_size` 64 through 800:

| list_size | before mean | after mean | speedup |
| ---: | ---: | ---: | ---: |
| 64 | 61.9 ms | 1.72 ms | 36.0x |
| 128 | 63.1 ms | 2.60 ms | 24.3x |
| 200 | 61.7 ms | 3.49 ms | 17.7x |
| 400 | 62.9 ms | 5.88 ms | 10.7x |
| 800 | 64.8 ms | 10.6 ms | 6.1x |

Recall@10 did not regress on the 100k sweep:

| list_size | before recall@10 | after recall@10 |
| ---: | ---: | ---: |
| 64 | 0.9165 | 0.9165 |
| 128 | 0.9625 | 0.9625 |
| 200 | 0.9745 | 0.9745 |
| 400 | 0.9855 | 0.9855 |
| 800 | 0.9865 | 0.9865 |

10k latency also improved from `5.16-6.73 ms` to `1.03-3.61 ms`.

## AWS State

The `10k` profile remains up for additional optimization cycles. The config
audit snapshot is recorded in `docs/aws-bench-workflow.md` as
`snap-0ac2d2a122442fd67`.

## References

- `plan/tasks/55-diskann-unsafe-burndown.md`
- `benchmarks/task55-aws-diskann-lowcost-config-audit/manifest.md`
- `benchmarks/task55-aws-diskann-lowcost-optimized/manifest.md`
- `docs/aws-bench-workflow.md`
