# Task 71 Review Request: IVF Stage Subphase Timing

## Scope

Code commit under review:

- `3e2fe08fb Split IVF build stage timing`

This slice splits the broad `stage_build_plan_us` counter into actionable
subphase timings:

- `stage_pq_train_us`
- `stage_centroids_us`
- `stage_assign_us`
- `stage_postings_us`
- `stage_directory_us`

The existing `stage_build_plan_us` total remains intact. `ecaz corpus load`,
`ecaz dev test ivf-parallel-build-probe`, the stable
`ec_ivf_last_build_timing()` SQL surface, and the pg_test helper all expose the
expanded timing row.

This is still instrumentation, not Task 71 completion. It narrows the next
implementation target: the real10k w8 probe shows stage time is mostly posting
encode/page staging, while the overall full build is still dominated by leader
training plus posting staging once heap ingest is parallelized.

## Validation

Packet-local artifacts are under
`reviews/task-71/005-stage-subphase/artifacts/`.

- `cargo check --no-default-features --features pg18`
  - passed.
- `cargo check -p ecaz-cli`
  - passed with the existing `LoadedDistributedPlacementConfig.path` warning.
- `cargo test -p ecaz-cli parses_ec_ivf_build_timing_rows`
  - passed.
- `./target/debug/ecaz --log-file reviews/task-71/005-stage-subphase/artifacts/install-after-stage-subphase-timing.log dev install ecaz-pg-test --pg 18`
  - passed; backend artifact assertion passed.
  - Installed dylib SHA:
    `d76af3edbd7a31676e703fba8009a51b6676a95bd4f60d68cd9aa37d7f819a68`.
- `./target/debug/ecaz --host /Users/peter/.pgrx --port 28818 --log-file reviews/task-71/005-stage-subphase/artifacts/probe-stage-subphase-w8.log dev test ivf-parallel-build-probe --drop-first --workers 8 --prefix task71_probe_w8_stage_subphase`
  - passed without approval escalation.
  - Loader artifact:
    `artifacts/probe-load-real10k-w8-stage-subphase.log`.
  - Key build line:
    `built task71_probe_w8_stage_subphase_idx in 463.36ms`.
  - Key timing row:
    `requested_workers=8 workers_launched=7 heap_ingest_us=35347 train_model_us=276084 stage_build_plan_us=144585 stage_pq_train_us=18071 stage_centroids_us=314 stage_assign_us=33218 stage_postings_us=92930 stage_directory_us=3 flush_build_plan_us=3477`.

## Interpretation

At real10k/w8:

- Heap ingest is already reduced to `35.3ms`.
- Leader training is `276.1ms`.
- Stage total is `144.6ms`.
- Within stage, postings dominate at `92.9ms`, followed by assignment at
  `33.2ms`; pq codebook training is only `18.1ms`.

This points the next Task 71 implementation work at deterministic posting
staging/encoding and/or avoiding the current all-core training baseline
confounding the PG worker curve. It does not change the packet 003 conclusion:
the current implementation still does not satisfy the multi-x full-build
speedup criterion.

## Review Focus

- Whether these subphase names and SQL columns are stable enough for packet 005
  and future worker-curve evidence.
- Whether the next implementation slice should target posting staging directly,
  given it is the dominant stage subphase in the w8 probe.
