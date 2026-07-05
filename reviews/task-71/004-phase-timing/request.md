# Task 71 Review Request: IVF Build Phase Timing

## Scope

Code commit under review:

- `bad1027cf Add IVF build phase timing`

This slice:

- Extends `ec_ivf_last_build_timing()` with leader-side phase counters:
  `train_model_us`, `stage_build_plan_us`, and `flush_build_plan_us`.
- Extends the pg_test timing helper and `ecaz corpus load` timing log row with
  the same fields.
- Updates `ecaz dev test ivf-parallel-build-probe` so it drops and recreates
  the probe timing SQL function before installing the expanded return shape.
- Reuses one resolved pq_fastscan posting quantizer during serial staging
  instead of resolving it for every posting tuple.

This does not claim Task 71 completion. The new evidence explains why packet
003's worker curve remains Amdahl-capped: once heap ingest is reduced,
leader-side training and staging dominate the 10k probe, and the prior 100k
curve still fails the multi-x full-build exit criterion.

## Validation

Packet-local artifacts are under
`reviews/task-71/004-phase-timing/artifacts/`.

- `cargo check --no-default-features --features pg18`
  - passed after the phase-timing changes.
- `cargo check -p ecaz-cli`
  - passed after the CLI timing query/probe-helper changes.
- `cargo test -p ecaz-cli parses_ec_ivf_build_timing_rows`
  - passed.
- `./target/debug/ecaz --log-file reviews/task-71/004-phase-timing/artifacts/install-after-phase-timing-safe.log dev install ecaz-pg-test --pg 18`
  - passed; backend artifact assertion passed.
  - Installed dylib SHA:
    `867afc0bbe3c12bc4e619a7c77c767198b0706b1d055ce5e2a38f9a1165613ce`.
- `./target/debug/ecaz --host /Users/peter/.pgrx --port 28818 --log-file reviews/task-71/004-phase-timing/artifacts/clean-before-phase-timing-probe.log dev test ivf-parallel-build-clean --include-probe --probe-workers 8`
  - passed; dropped 17 prefixes.
- Current safe real10k probe, w1:
  - Command artifact: `artifacts/probe-phase-timing-w1.log`.
  - Loader artifact: `artifacts/probe-load-real10k-w1-after-loader-timing.log`.
  - Build time: `517.65ms`.
  - Timing row:
    `requested_workers=1 workers_launched=1 heap_ingest_us=91437 train_model_us=275720 stage_build_plan_us=142949 flush_build_plan_us=3363`.
- Current safe real10k probe, w8:
  - Command artifact: `artifacts/probe-phase-timing-w8.log`.
  - Loader artifact: `artifacts/probe-load-real10k-w8-after-loader-timing.log`.
  - Build time: `404.66ms`.
  - Timing row:
    `requested_workers=8 workers_launched=7 heap_ingest_us=35043 train_model_us=258088 stage_build_plan_us=106057 flush_build_plan_us=2212`.

## Rejected Experiment

I also tested a rayon-based pq_fastscan posting-encoding variant and backed it
out before `bad1027cf`.

Artifacts are retained under
`artifacts/rejected-rayon-encode/`.

- real10k improved modestly:
  - w1 `432.68ms`, w8 `349.29ms`.
- real100k failed badly under the target w8 path:
  - w1 `2.31s`, with `heap_ingest_us=876780 train_model_us=501504 stage_build_plan_us=834268`.
  - w8 `9.22s`, with `heap_ingest_us=1455206 train_model_us=3280731 stage_build_plan_us=4251560`.
- Conclusion: rayon posting encode risks oversubscription/phase contention under
  PostgreSQL parallel build workers and is not a viable Task 71 completion path
  as tested.

## Review Focus

- Whether the expanded phase timing surface is acceptable for ongoing Task 71
  measurement.
- Whether the probe helper should use `DROP FUNCTION IF EXISTS` before
  recreating the timing function, given the return shape may evolve during
  development.
- Whether the retained current evidence is enough to justify focusing the next
  implementation slice on leader-side training/staging rather than heap ingest.
