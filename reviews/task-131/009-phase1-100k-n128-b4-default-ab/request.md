# Task 131 Phase 1 100k n128/b4 Default A/B

This packet adds the 100k local multi-instance `n128/b4` Phase 1 A/B result for global pre-heap merge.

## Result

- Recall matched at `1.0000`.
- Query p50/p95/p99 changed from `5366.063 / 6413.783 / 6711.519 ms` to `5357.062 / 6199.141 / 6616.327 ms`.
- Production-read total p50/p95/p99 changed from `5341 / 6259 / 6620 ms` to `2668 / 3283 / 3784 ms`.
- Remote heap rows changed from `6000` to `2000`.
- Payload bytes remained `0` in both arms because the run used no-payload timelines.
- Safety counters remained clean: strict failures, timeouts, cancels, and degraded skips were all `0`.
- Coordinator storage for the 100k surface was `1.9 GiB` total, with `394.5 MiB` of indexes.

## Evidence

- Manifest: `artifacts/manifest.md`
- Structured results: `artifacts/100k-n128-b4/bench-suite/results.jsonl`
- Suite logs/manifests: `artifacts/100k-n128-b4/bench-suite/`
- Top-level suite outputs copied from packet 004: `artifacts/100k-n128-b4-results.jsonl`, `artifacts/100k-n128-b4-suite-manifest.json`, `artifacts/100k-n128-b4-suite-run.log`

Generated split TSVs and temporary PG target directories were removed before packaging.
