# Task 51 Local RaBitQ8 Sidecar Scoring Variants

- head SHA: `b09c8d75c141fc4ae99a6db6e110d77c9d10e902`
- benchmark packet: `benchmarks/task51-local-rabitq8ls-sidecar/`
- SuiteConfig: `benchmarks/task51-local-rabitq8ls-sidecar/suite.json`
- suite manifest: `benchmarks/task51-local-rabitq8ls-sidecar/artifacts/suite-manifest.json`
- results: `benchmarks/task51-local-rabitq8ls-sidecar/artifacts/results.jsonl`
- parsed report: `benchmarks/task51-local-rabitq8ls-sidecar/artifacts/results-report.jsonl`
- timestamp: 2026-05-23

## Surface

- local PG18 socket: `/home/peter/.pgrx`, port `28818`
- fixture: preserved isolated 50k prefix `task51_local_50k_ivf_rabitq1_n128_sidecar_off`
- index shape: one `ec_ivf` index, `storage_format=rabitq`, `rerank=off`, `nlists=128`, `nprobe=128`
- lane: IVF/RaBitQ only
- sidecar bytes/vector: `1548` for every q8 variant
- sidecar size: `73.81 MiB` at 50k rows for every q8 variant
- candidate frontier: `candidate_k=50`
- rerank modes: free-I/O upper bound and real DB `tid-sorted`
- isolated one-index-per-table surface: yes

## Commands

```sh
cargo test -p ecaz-cli --no-default-features sidecar
cargo build -p ecaz-cli --no-default-features
target/debug/ecaz --log-file benchmarks/task51-local-rabitq8ls-sidecar/artifacts/suite-audit.log bench suite audit --config benchmarks/task51-local-rabitq8ls-sidecar/suite.json
target/debug/ecaz --host /home/peter/.pgrx --port 28818 --log-file benchmarks/task51-local-rabitq8ls-sidecar/artifacts/suite-dry-run.log bench suite run --config benchmarks/task51-local-rabitq8ls-sidecar/suite.json --dry-run --manifest-output benchmarks/task51-local-rabitq8ls-sidecar/artifacts/suite-manifest.json
target/debug/ecaz --host /home/peter/.pgrx --port 28818 --log-file benchmarks/task51-local-rabitq8ls-sidecar/artifacts/suite-run.log bench suite run --config benchmarks/task51-local-rabitq8ls-sidecar/suite.json --manifest-output benchmarks/task51-local-rabitq8ls-sidecar/artifacts/suite-manifest.json
target/debug/ecaz --log-file benchmarks/task51-local-rabitq8ls-sidecar/artifacts/suite-status.log bench suite status --manifest benchmarks/task51-local-rabitq8ls-sidecar/artifacts/suite-manifest.json
target/debug/ecaz --log-file benchmarks/task51-local-rabitq8ls-sidecar/artifacts/suite-report.log bench suite report --manifest benchmarks/task51-local-rabitq8ls-sidecar/artifacts/suite-manifest.json --results-output benchmarks/task51-local-rabitq8ls-sidecar/artifacts/results-report.jsonl
```

The plain `cargo test -p ecaz --lib least_squares_estimator_uses_o_dot_as_shrinkage --no-default-features --features pg18` binary compiled but failed to run outside the pgrx loader with `undefined symbol: CacheRegisterRelcacheCallback`; this is recorded as a local validation limitation, not a new test failure in the touched logic.

## Key Results

| variant | clip / scoring | read mode | recall@10 | ndcg@10 | sidecar p50 | sidecar score p50 | total bound p50 |
| --- | --- | --- | ---: | ---: | ---: | ---: | ---: |
| `rabitq8` | 2 sigma, paper scale | `tid-sorted` | 0.9480 | 0.9996 | 2.013 ms | 1.043 ms | 196.637 ms |
| `rabitq8ls` | 2 sigma, least-squares scale | `tid-sorted` | 0.9490 | 0.9996 | 2.005 ms | 1.039 ms | 196.619 ms |
| `rabitq8c3` | 3 sigma, paper scale | `tid-sorted` | 0.9810 | 1.0000 | 2.049 ms | 1.047 ms | 196.930 ms |
| `rabitq8c4` | 4 sigma, paper scale | `tid-sorted` | 0.9950 | 1.0000 | 2.387 ms | 1.209 ms | 197.337 ms |

Conclusion: increasing the q8 scalar clip radius is a credible pure-RaBitQ path. The best local result, `rabitq8c4`, keeps the same sidecar layout size while lifting recall from `0.9480` to `0.9950`.
