# Task 65b Packet 013: Worker/Batch Sweep Results

## Summary

This packet records the actual PG18 execution of the Task 65b Slice F/H worker-count and batch-size suite from `reviews/task-65b/011-worker-batch-sweep/suite.json`.

The corrected suite invocation completed all 22 configured steps with no failures:

- real10k load matrix: `w1/b1`, `w2/b4`, `w4/b8`, `w4/b16`, `w8/b16`, `w8/b32`
- real10k recall checks: `w1/b1`, `w4/b16`, `w8/b32`
- real10k graph digest and storage checks: `w4/b16`, `w8/b32`
- real100k load matrix: `w4/b16`, `w8/b32`
- real100k recall, graph digest, and storage checks for both real100k candidates

## Result

The current implementation does not satisfy the Task 65b final performance gate.

| Step | Workers | Batch | Build Time | Effective Workers | Reducer Time | Proposal Time |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| real10k w1/b1 | 1 | 1 | 7.18 s | 1 | 978 ms | 0 ms |
| real10k w2/b4 | 2 | 4 | 5.53 s | 2 | 2021 ms | 171 ms |
| real10k w4/b8 | 4 | 8 | 5.38 s | 4 | 3286 ms | 558 ms |
| real10k w4/b16 | 4 | 16 | 4.70 s | 4 | 3105 ms | 726 ms |
| real10k w8/b16 | 8 | 16 | 4.91 s | 8 | 3552 ms | 546 ms |
| real10k w8/b32 | 8 | 32 | 5.59 s | 8 | 4432 ms | 603 ms |
| real100k w4/b16 | 4 | 16 | 335.82 s | 4 | 286136 ms | 36047 ms |
| real100k w8/b32 | 8 | 32 | 192.38 s | 8 | 170504 ms | 15626 ms |

Task gate comparison:

- real10k target: `<= 3s`; best observed: `4.70s`.
- real100k target: `<= 30s`; best observed: `192.38s`.

Recall remained stable for the selected candidates:

- real10k `list_size=200`: `0.9975` for `w1/b1`, `w4/b16`, and `w8/b32`.
- real100k `list_size=200`: `0.9750` for both `w4/b16` and `w8/b32`.

## Interpretation

The suite confirms that worker accounting and timing capture are wired correctly, but the deterministic reducer is now the dominant bottleneck. The next Task 65b implementation slice should target reducer cost, not more batch-size tuning.

## Evidence

- Suite config: `reviews/task-65b/011-worker-batch-sweep/suite.json`
- Corrected suite manifest: `artifacts/suite-manifest-host.json`
- Corrected suite run log: `artifacts/suite-run-host.log`
- Normalized results: `artifacts/results-host.jsonl`
- Generated report: `artifacts/suite-report.md`
- Setup/install logs: `artifacts/install-current-extension.log`, `artifacts/install-diskann-timing-helper.log`
- Per-step load, recall, graph, and storage logs under `artifacts/`

## Notes

The first suite invocation omitted the global `--host`/`--port` for load steps and failed at `load-real10k-w1-b1` before creating a benchmark result. Its logs are preserved as `artifacts/suite-run.log` and `artifacts/suite-manifest.json`; the authoritative measurement run is the `*-host` manifest and log.
