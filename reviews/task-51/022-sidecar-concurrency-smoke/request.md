# Review Request: Sidecar Concurrency Smoke

- task: Task 51 AWS IVF/RaBitQ optimization
- code under review: `4235b7ba12965359453c8229c0bdfa2b651ddf40` (`Add sidecar rerank concurrency option`)
- benchmark packet: `benchmarks/task51-local-ivf-sidecar-concurrency-smoke/`
- artifacts manifest: `benchmarks/task51-local-ivf-sidecar-concurrency-smoke/manifest.md`
- packet artifacts: `reviews/task-51/022-sidecar-concurrency-smoke/artifacts/`
- lane: local PG18 only
- AWS: not used
- competitors: none; IVF/RaBitQ only

## Scope

This checkpoint adds a narrow concurrency control to the measurement-only
sidecar harness:

- `ecaz bench sidecar-rerank --concurrency N`
- `ecaz bench suite` `sidecar-rerank` step field `"concurrency": N`

The implementation preserves the existing default of `1` and only changes DB
sidecar read modes (`random-id`, `tid-sorted`). It pipelines per-query sidecar
fetch/score tasks with bounded concurrency and keeps the result ordering stable
for recall/NDCG accounting.

## Validation

Focused Rust validation:

```text
script -q -c "cargo test -p ecaz-cli --no-default-features sidecar" benchmarks/task51-local-ivf-sidecar-concurrency-smoke/artifacts/cargo-test-sidecar.log
script -q -c "cargo build -p ecaz-cli --no-default-features" benchmarks/task51-local-ivf-sidecar-concurrency-smoke/artifacts/cargo-build-ecaz-cli.log
```

Result:

```text
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 354 filtered out; finished in 0.00s
```

Suite validation:

```text
target/debug/ecaz --log-file benchmarks/task51-local-ivf-sidecar-concurrency-smoke/artifacts/suite-audit.log bench suite audit --config benchmarks/task51-local-ivf-sidecar-concurrency-smoke/suite.json
target/debug/ecaz --host /home/peter/.pgrx --port 28818 --log-file benchmarks/task51-local-ivf-sidecar-concurrency-smoke/artifacts/suite-dry-run.log bench suite run --config benchmarks/task51-local-ivf-sidecar-concurrency-smoke/suite.json --dry-run --manifest-output benchmarks/task51-local-ivf-sidecar-concurrency-smoke/artifacts/suite-manifest.json
target/debug/ecaz --host /home/peter/.pgrx --port 28818 --log-file benchmarks/task51-local-ivf-sidecar-concurrency-smoke/artifacts/suite-run.log bench suite run --config benchmarks/task51-local-ivf-sidecar-concurrency-smoke/suite.json --manifest-output benchmarks/task51-local-ivf-sidecar-concurrency-smoke/artifacts/suite-manifest.json
target/debug/ecaz --log-file benchmarks/task51-local-ivf-sidecar-concurrency-smoke/artifacts/suite-status.log bench suite status --manifest benchmarks/task51-local-ivf-sidecar-concurrency-smoke/artifacts/suite-manifest.json
target/debug/ecaz --log-file benchmarks/task51-local-ivf-sidecar-concurrency-smoke/artifacts/suite-report.log bench suite report --manifest benchmarks/task51-local-ivf-sidecar-concurrency-smoke/artifacts/suite-manifest.json --results-output benchmarks/task51-local-ivf-sidecar-concurrency-smoke/artifacts/results-report.jsonl
```

Suite status:

```text
[suite:task51-local-ivf-sidecar-concurrency-smoke] completed=1 failed=0 skipped=0 dry_run=0 missing_artifacts=0 stale=0
```

## Local Smoke Results

Fixture:

- prefix: `task51_local_50k_ivf_rabitq1_n128_sidecar_off`
- q=20, k=10, candidate_k=50, nprobe=96
- concurrency: 4
- variants: `f16`, `rabitq8`
- read modes: `random-id`, `tid-sorted`

| variant | read mode | recall@10 | sidecar I/O p50 | sidecar score p50 | sidecar total p50 | total p50 |
| --- | --- | ---: | ---: | ---: | ---: | ---: |
| f16 | random-id | 1.0000 | 34.653 ms | 4.969 ms | 39.615 ms | 180.480 ms |
| f16 | tid-sorted | 1.0000 | 18.743 ms | 4.962 ms | 23.733 ms | 164.994 ms |
| rabitq8 | random-id | 0.9450 | 26.470 ms | 1.092 ms | 27.516 ms | 168.099 ms |
| rabitq8 | tid-sorted | 0.9450 | 4.419 ms | 1.063 ms | 5.552 ms | 145.633 ms |

## Reviewer Notes

- This does not claim product sidecar readiness; it only adds and smokes the
  concurrency knob needed for the requested AWS Graviton sidecar cell.
- The local smoke intentionally reuses the preserved 50k sidecar-off fixture
  and existing sidecar tables; it does not rebuild unrelated indexes.
- `sidecar_io_*` still includes DB fetch and `ORDER BY ctid` for `tid-sorted`,
  consistent with packet 020.
