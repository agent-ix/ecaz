# Task 28 IVF Prepared Quantizer Scan Trial

This packet records a negative A/B trial for a proposed scan hot-path cleanup:
carry the cached `ProdQuantizer` inside `IvfPreparedQuery` so posting scoring
does not call `ProdQuantizer::cached(...)` for every candidate.

The idea was plausible, but the local latency result moved the wrong way. The
quantizer change was backed out and is not part of the branch. The only code
landed from this slice is `ecaz bench latency --log-output`, which lets future
bench packets store raw latency tables without shell redirection.

## Measurement Result

Fixture:

- Local PG18 scratch database `postgres`.
- Existing isolated 10k x 1536 surface
  `task28_ivf_postopt10k_n64w25`.
- `ecaz bench latency`, profile `ec_ivf`, `k=10`, `concurrency=1`,
  `iterations=100`, sweep `nprobe=32,48`.
- Cache state: warm local development run; no explicit cache drop.
- Memory high-water mark: not captured.

| run | nprobe | p50 | p95 | p99 |
| --- | ---: | ---: | ---: | ---: |
| prepared-quantizer trial r1 | 32 | 111.8 ms | 147.2 ms | 159.0 ms |
| prepared-quantizer trial r1 | 48 | 156.1 ms | 166.5 ms | 172.8 ms |
| prepared-quantizer trial r2 | 32 | 108.4 ms | 115.6 ms | 120.6 ms |
| prepared-quantizer trial r2 | 48 | 156.0 ms | 165.6 ms | 167.9 ms |
| backed-out A/B baseline | 32 | 97.7 ms | 119.6 ms | 125.6 ms |
| backed-out A/B baseline | 48 | 139.1 ms | 150.1 ms | 175.6 ms |

Packet 30052 previously reported the same surface at p50 98.1 ms for
`nprobe=32` and 140.2 ms for `nprobe=48`. The backed-out A/B baseline matches
that old band; the prepared-quantizer trial does not. Do not land this
quantizer shape as a Task 28 optimization.

## Artifacts

- `artifacts/latency_10k_n64w25_nprobe32_48.log`
- `artifacts/latency_10k_n64w25_nprobe32_48_r2.log`
- `artifacts/latency_10k_n64w25_nprobe32_48_ab_baseline.log`
- `artifacts/manifest.md`

## Validation

For the landed bench logging support:

- `cargo fmt --check`
- `cargo test -p ecaz-cli latency`
- `git diff --check`

For the rejected quantizer trial:

- `cargo test --lib am::ec_ivf::quantizer --no-default-features --features pg18`
- `cargo test --lib am::ec_ivf --no-default-features --features pg18`
- `cargo pgrx install --pg-config /home/peter/.pgrx/18.3/pgrx-install/bin/pg_config --features pg18,pg_test --no-default-features`

## Recommendation

Stop pursuing this prepared-query shape. The next useful IVF scan slice should
target the actual posting-list scoring/layout cost rather than moving the
quantizer cache lookup. DiskANN remains task 29 and is not included.
