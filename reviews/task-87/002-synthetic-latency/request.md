# Task 87 Review Request: Synthetic SPIRE Latency Slice

Code commit under review: `40c36f73982459f6fec39590482878445b5b187a`

This packet adds the Task 87 scan-latency evidence requested by
`plan/tasks/87-turboquant-candidate-batching.md` implementation order item 5.
It uses `ecaz bench suite` only; no ad hoc benchmark sweeper was added.

## Suite

Config:

- `reviews/task-87/002-synthetic-latency/suite-synthetic-spire.json`

Shape:

- PG18 local socket: `/home/peter/.pgrx`, port `28818`
- profile: `ec_spire`
- storage format: `turboquant`
- fixture: deterministic synthetic 2,000-row corpus, 50 queries, dim 1536
- index reloptions: `storage_format=turboquant`, `nlists=16`, `nprobe=8`,
  `rerank_width=25`
- latency sweep: `nprobe = 4, 8`, 50 iterations each

The generated TSV intermediates were not committed because the corpus TSV was
28 MB. The suite config fully regenerates them, and the load log records hashes:

- corpus SHA-256:
  `82ce5809a55fc5e2167becd55bc42f97c78d1f0c1abd5148f4317331e346e2a0`
- queries SHA-256:
  `ae3adb391a30fd2a52bfcd8e6dc53b1bd8f918b697e09b4a82023a2b4a611d0c`

## Results

Key latency rows from `artifacts/latency-synth2k-spire-turboquant.log`:

- `nprobe=4`: count `50`, p50 `27.5 ms`, p95 `32.1 ms`, p99 `59.5 ms`
- `nprobe=8`: count `50`, p50 `36.4 ms`, p95 `41.8 ms`, p99 `62.3 ms`

Suite status:

- `completed=2 failed=0 skipped=4 dry_run=0 missing_artifacts=0 stale=0`

The skipped steps in the final manifest are the precheck/generate steps from
the selected rerun. They ran successfully in the initial full-suite invocation;
the corrected rerun selected only load and latency after fixing the load input
paths from `${artifact_dir}/...` to explicit packet-local paths.

## Notes

This is a small synthetic latency slice, not a claim of product-scale speedup.
It proves the changed scan path builds and executes on PG18 through a real
SPIRE TurboQuant index while preserving the Task 87 no-format-change boundary.
