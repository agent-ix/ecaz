# Task 55 AWS DiskANN Low-Cost Optimized

Purpose: prove the first `ec_diskann` scan-path optimization on the same
low-cost Graviton lane used by
`benchmarks/task55-aws-diskann-lowcost-config-audit/`.

## Scope

- access method: `ec_diskann`
- hardware lane: low-cost Graviton only; Intel is deferred
- profile: `10k` cloud profile (`m8g.large` database host)
- datasets: DBpedia/OpenAI3 `ec_real_10k` and `ec_real_100k`
- suite runner: `ecaz bench suite` through `ecaz cloud bench`
- before packet: `benchmarks/task55-aws-diskann-lowcost-config-audit/`

## Code Change Under Test

The optimized scan path no longer materializes every DiskANN data-page tuple
into a `DataPageChain` during `ambeginscan` for the normal binary-sidecar scan
path. It reads only visited graph nodes from the live index relation. The
grouped-PQ prefilter path keeps the existing materialized fallback because it
needs persisted codebooks.

Local validation before AWS:

```text
cargo test --lib ec_diskann::scan --no-run
cargo check --all-targets --no-default-features --features pg18
cargo build --release
```

Note: plain `cargo test --lib ec_diskann::scan` compiled but could not execute
outside PostgreSQL because the test binary lacked PostgreSQL FFI symbols such
as `LockBuffer`.

## Suite

Config: `suite.json`

Expected command shape:

```text
target/release/ecaz cloud bench --profile 10k --suite task55-aws-diskann-lowcost-optimized --database postgres --config benchmarks/task55-aws-diskann-lowcost-optimized/suite.json --ecaz-bin /usr/local/bin/ecaz --log-file benchmarks/task55-aws-diskann-lowcost-optimized/artifacts/cloud-bench.log
```

Actual remote command used `/usr/local/bin/ecaz` for `--ecaz-bin`; the local
wrapper command was interrupted after submission, but the remote suite
completed and uploaded artifacts to S3.

The AWS stack should remain up after this run for further optimization cycles.

## Acceptance

- AWS suite completed on optimized commit
  `cbf037334ce0a9f499507d206049574b8278282e`.
- Remote sanity check before the run:
  `/usr/local/bin/ecaz --version` returned `ecaz 0.1.0`, PostgreSQL reported
  extension `ecaz 0.1.1`, and the checkout HEAD was the optimized commit.
- Uploaded artifacts synced from S3 under
  `artifacts/s3-sync/20260524T165309Z/`.
- `suite-manifest.json` records all 21 steps as `succeeded`.

## Result Summary

The primary optimization claim is proven on `ec_real_100k`: the scan path no
longer pays a fixed full-index materialization cost per scan. Mean SQL latency
now scales with `list_size` instead of staying flat around 62-65 ms.

100k latency before vs after:

| list_size | before mean | after mean | speedup |
| ---: | ---: | ---: | ---: |
| 64 | 61.9 ms | 1.72 ms | 36.0x |
| 128 | 63.1 ms | 2.60 ms | 24.3x |
| 200 | 61.7 ms | 3.49 ms | 17.7x |
| 400 | 62.9 ms | 5.88 ms | 10.7x |
| 800 | 64.8 ms | 10.6 ms | 6.1x |

Recall did not regress; the 100k recall@10 sweep is identical to the before
packet at every measured `list_size`:

| list_size | before recall@10 | after recall@10 |
| ---: | ---: | ---: |
| 64 | 0.9165 | 0.9165 |
| 128 | 0.9625 | 0.9625 |
| 200 | 0.9745 | 0.9745 |
| 400 | 0.9855 | 0.9855 |
| 800 | 0.9865 | 0.9865 |

10k latency also improved:

| list_size | before mean | after mean |
| ---: | ---: | ---: |
| 64 | 5.16 ms | 1.03 ms |
| 128 | 5.28 ms | 1.33 ms |
| 200 | 5.46 ms | 1.67 ms |
| 400 | 5.94 ms | 2.38 ms |
| 800 | 6.73 ms | 3.61 ms |

Notes:

- The 100k recall log's `mean q-time` at `list_size=64` shows one anomalous
  cold/first-run value (`547.74 ms`), but the dedicated latency step over the
  same sweep shows stable 1.72 ms mean for `list_size=64`.
- Build-probe graph quality stayed unchanged for the 100k wide-alpha sample:
  `reachable_fraction=1.000000`, `recall@10=0.8630`, and `build_seconds=407.715`.
- Storage stayed unchanged, as expected: 100k `ec_diskann` index size remained
  `46.1 MiB` / `483.1 B` per row.
