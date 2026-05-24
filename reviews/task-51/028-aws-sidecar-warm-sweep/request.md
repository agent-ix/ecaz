# Review Request: AWS Sidecar Warm Sweep Evidence

- Code commit: `6a0338e40` (`ecaz cloud install` checkout cleanup)
- Measurement host SHA: `7325e3bb123924cd79ccfd09a55db6cebbb72c86`
- Benchmark packet: `benchmarks/task51-aws-rabitq8-sidecar-warm-sweep`
- Scope: warm AWS rerun for the new RaBitQ8 sidecar variants only

## Why

The cold sidecar sweep reported `candidate_sql_p50=1759.456 ms`, which
was not a valid steady-state IVF characterization. This packet reruns the
same new-sidecar cells with explicit warmup and records the completed
evidence in the benchmark packet.

## Change

`ecaz cloud install` now cleans the remote build checkout before switching
refs. The preserved cloud checkout had packet-local benchmark artifacts in
the worktree, and exact-SHA install attempts failed until the checkout was
reset/cleaned. This is an operator-path cleanup only; it does not change
IVF, RaBitQ, or sidecar benchmark scoring behavior.

## Benchmark Result

Warm AWS sweep parameters:

- Variants: `rabitq8`, `rabitq8ls`, `rabitq8c3`, `rabitq8c4`
- Excluded: vchord, pgvectorscale/DiskANN, unchanged baselines
- Fixture: `real_1m_ivf_rabitq1_rerank`, `990000` rows
- nprobe: `128`
- Candidate K: `50`
- Timed queries: `200`
- Warmup queries: `200`
- Read mode: `tid-sorted`
- Concurrency: `1`
- Sidecar table rebuild: `false`

| Variant | recall@k | candidate_sql_p50 | sidecar_io_p50 | total_bound_p50 |
| --- | ---: | ---: | ---: | ---: |
| `rabitq8` | 0.9455 | 35.095 ms | 0.184 ms | 35.511 ms |
| `rabitq8ls` | 0.9405 | 35.095 ms | 0.178 ms | 35.517 ms |
| `rabitq8c3` | 0.9700 | 35.095 ms | 0.179 ms | 35.515 ms |
| `rabitq8c4` | 0.9800 | 35.095 ms | 0.180 ms | 35.506 ms |

This confirms the 1.76s candidate number was cold/materialization
behavior, not the warm IVF path. The result should be cited as a hot-cache
sidecar check only; it is not evidence for cold-start tails.

## Validation

- `artifacts/cargo-test-ecaz-cloud-install.log`: `ecaz-cloud` focused test target passed.
- `artifacts/cargo-test-ecaz-cli-sidecar.log`: sidecar/suite focused tests passed, `7 passed; 0 failed`.
- `benchmarks/task51-aws-rabitq8-sidecar-warm-sweep/artifacts/suite-manifest.json`: AWS suite command includes `--warmup-queries 200`.
- `benchmarks/task51-aws-rabitq8-sidecar-warm-sweep/artifacts/results.jsonl`: structured warm sweep results.

AWS profile `10k-medium` was paused after artifact sync: `$0.00/hr` running compute, retained snapshot `snap-0b72153293b0b749b`.
