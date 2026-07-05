# Review Request: Task 144 Packet 003 — Phase 0 Release Geometry

## Scope

This packet publishes the Task 144 Phase 0 geometry diagnostic on a release PG18 backend:

- 50k and 100k real-corpus `ec_spire` n1024/b0 indexes.
- Leaf-size variance.
- Per-query true top-10 leaf-list concentration for current single assignment.
- Read-only simulated closure concentration for epsilon `0.05`, `0.10`, and `0.20`.
- A small CLI fix in `ef3e9b420` so geometry truth-source fetches use ADR-069-safe `id = $1` primary-key lookups instead of `id = ANY(...)`.

## Evidence

- Manifest: `reviews/task-144/003-phase0-geometry-release/artifacts/manifest.md`
- Suite config: `reviews/task-144/003-phase0-geometry-release/artifacts/suite-task144-phase0-geometry-release.json`
- Suite manifest/results: `reviews/task-144/003-phase0-geometry-release/artifacts/suite-manifest.json`, `results.jsonl`
- Geometry JSONL: `geometry-50k-n1024.jsonl`, `geometry-100k-n1024.jsonl`
- Validation: `cargo-test-ecaz-cli-spire-pipeline.log` (`30 passed; 0 failed`)

Suite status: `completed=7 failed=0 skipped=0 stale=0`.

## Headline Results

Leaf-size variance is moderate and non-empty:

| scale | CV | p50 rows | p90 rows | max rows |
| --- | ---: | ---: | ---: | ---: |
| 50k | 0.550 | 45 | 85 | 196 |
| 100k | 0.496 | 89 | 161 | 347 |

True top-10 concentration is already above the target 1-4 list regime:

| scale | mode | epsilon | mean leaves/query | p90 | max | mean assignment rows/query |
| --- | --- | ---: | ---: | ---: | ---: | ---: |
| 50k | single | n/a | 5.505 | 8 | 10 | 10.000 |
| 50k | closure sim | 0.05 | 6.405 | 11 | 13 | 11.420 |
| 50k | closure sim | 0.10 | 7.965 | 14 | 25 | 14.120 |
| 50k | closure sim | 0.20 | 16.065 | 33 | 123 | 27.155 |
| 100k | single | n/a | 5.615 | 8 | 10 | 10.000 |
| 100k | closure sim | 0.05 | 6.635 | 10 | 13 | 11.460 |
| 100k | closure sim | 0.10 | 8.935 | 15 | 28 | 14.625 |
| 100k | closure sim | 0.20 | 18.840 | 36 | 96 | 28.755 |

The simulated closure rows use the diagnostic IP-distance proxy documented in packet 002. They are not build-side replicated indexes yet.

## Ask

Please review the Phase 0 measurement and the DML-safe CLI fix. The main decision point is whether this diagnostic is sufficient to proceed into Phase 1/2 as gated defaults-off implementation, given that closure simulation increases assignment rows and does not by itself make the true-neighbor cover fit in 1-4 leaves.
