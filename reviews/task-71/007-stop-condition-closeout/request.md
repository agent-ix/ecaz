# Task 71 Review Request: Stop-Condition Closeout

## Scope

This packet closes Task 71 by invoking its local M5 Stop Condition for further
Option A optimization. The branch has implemented IVF parallel build with
`amcanbuildparallel = true` and `amcanparallel = false`, validated worker
launch and deterministic output, and measured the full worker curve. The
measured result is useful but bounded: Option A reduces heap-ingest time, while
leader-owned training and staging keep full `CREATE INDEX` speedup below the
task's multi-x exit criterion.

This closeout also updates:

- `plan/tasks/71-ivf-parallel-build.md` status to completed under the Stop
  Condition.
- `reviews/task-71/001-phase1-design/request.md` with the measured Phase 3
  addendum requested by reviewer feedback.

No code changes are in this packet.

## Evidence Summary

Phase 1 selected Option A: per-worker heap tuple collection with leader-side
training, assignment, staging, and flush. The measured addendum now answers the
initial Amdahl uncertainty. On the packet 003 final matrix, w1 heap-ingest
share was ~18.1%, ~28.1%, ~34.3%, and ~33.4% for
real10k/25k/50k/100k. Even with ideal infinite-worker heap ingest, the full
build ceiling is only ~1.22x, ~1.39x, ~1.52x, and ~1.50x unless additional
leader work moves to a broader Option B/C shape.

Phase 2 landed end-to-end IVF parallel build:

- `ec_ivf.amcanbuildparallel = true`
- `ec_ivf.amcanparallel = false`
- PostgreSQL parallel build callbacks wired for PG18
- worker tuple fan-in through the IVF parallel build coordinator
- leader merge sorted by heap TID before the existing deterministic build path
- serial fallback when no workers launch
- structural serial-vs-parallel pg_test coverage in packet 002

Phase 3 measured the required M5 worker curve with isolated one-index-per-table
surfaces. Packet 003 final matrix results:

| scale | worker launch shape | full build w1 -> best | best speedup | recall@10 | index size |
|---|---|---:|---:|---:|---:|
| real10k | `1/1 2/2 4/4 8/7` | `0.464140s -> 0.411170s` | ~1.13x | `1.0000` | `2726298` |
| real25k | `1/1 2/2 4/4 8/7` | `0.721680s -> 0.612060s` | ~1.18x | `0.9990` | `5557453` |
| real50k | `1/1 2/2 4/4 8/7` | `1.160000s -> 0.922410s` | ~1.26x | `1.0000` | `10171187` |
| real100k | `1/1 2/2 4/4 8/7` | `2.630000s -> 2.030000s` | ~1.30x | `0.9820` | `20342374` |

The recall values match the Task 31 comparator baselines used by this task:
real10k `1.0000`, real25k `0.9990`, real50k `1.0000`, real100k `0.9820`.
Index sizes are byte-identical across worker counts for each scale. Memory HWM
was not sampled in this matrix and is recorded as `not_measured` per the task
allowance.

Packet 003 also explains the worker-counter anomaly. The suite-level
`pg_stat_get_db_parallel_workers_launched` delta remained `0` in this local
environment, so the accepted worker evidence is the per-build
`ec_ivf_build_timing` row emitted by the loader. Those rows recorded the
`requested_workers/workers_launched` shapes above and are the authoritative
evidence that this is parallel build, not parallel scan.

Packets 004 and 005 split the leader timing enough to decide whether more
Option A decomposition would change the result. For real10k w8, packet 005
recorded:

- `heap_ingest_us=35347`
- `train_model_us=276084`
- `stage_build_plan_us=144585`
- `stage_postings_us=92930`

Packet 006 then removed the build-time one-TID posting allocation and improved
the one-cell real10k w8 `stage_postings_us` from `92930` to `62898`. The same
probe still recorded `train_model_us=263644`, so the full build remains far
from multi-x. Further local Option A work is not expected to satisfy the Task
71 multi-x criterion.

## Stop Condition

Task 71 says to stop if per-worker overhead is comparable to the heap-scan
speedup at M5 corpus sizes and to defer to a Graviton/cloud-class fixture
before committing more implementation cost. The final measurements meet that
condition:

- heap ingest scales, including real100k `877228us -> 274479us` from w1 to w8
  (~3.19x for the heap phase);
- full build only reaches ~1.30x at real100k w8;
- measured Amdahl math predicts the observed full-build ceiling;
- packet 004/005/006 timing shows leader train/stage dominates remaining wall
  time.

This closeout therefore stops further local Option A optimization and defers
multi-x work to an explicit Option B/C design or a cloud-class fixture.

Do not revert `amcanbuildparallel = true`. The implementation is correct,
worker-launching, deterministic, recall-preserving, clippy-clean, and provides
a real full-build improvement on every measured scale. Reverting it would throw
away validated speedup and would not improve correctness.

## Validation

Packet-local artifacts are under
`reviews/task-71/007-stop-condition-closeout/artifacts/`.

- `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings`
  - passed.
  - Artifact: `artifacts/cargo-clippy-pg18.log`
  - Key line: `Finished dev profile [unoptimized + debuginfo] target(s) in 48.64s`

No DB setup or runtime pg_test was rerun for this documentation closeout. The
latest runtime PG18 evidence remains packet 006, which used the CLI-owned
`ecaz dev install` and `ecaz dev test ivf-parallel-build-probe` surfaces.

## Pre-Merge Follow-Up

Reviewer seq 02 converted three surface follow-ups into pre-merge blockers.
This follow-up resolves them:

- Suite runner `capture_parallel_workers` no longer brackets load steps with
  `pg_stat_get_db_parallel_workers_launched`, because that database-cumulative
  snapshot stayed stale in the Task 71 matrix and produced misleading
  `parallel_workers_delta=0` rows.
- The runner now parses the load step's own
  `[loader] ec_ivf build timing: ... workers_launched=N` line from expected
  load artifacts after a successful opted-in load step.
- For manifest compatibility, `parallel_workers_before` is recorded as `0`,
  while `parallel_workers_after` and `parallel_workers_delta` both record the
  parsed per-build `workers_launched` count.
- `IndexInfoGuard` and `IndexInfoView` moved to
  `src/am/common/index_info.rs`; IVF and HNSW now import the same typed wrapper
  instead of carrying duplicate local versions.
- The two stale untracked packet-003 probe artifacts from the stale-dylib
  triage path were removed because they were redundant with later committed
  evidence and did not contain the authoritative loader timing row.

Validation for this follow-up:

- `cargo test -p ecaz-cli commands::bench::suite::tests::`
  - passed: `40 passed; 0 failed`
- `cargo check --no-default-features --features pg18`
  - passed: `Finished dev profile [unoptimized + debuginfo] target(s) in 23.70s`
- `cargo build -p ecaz-cli`
  - passed, producing the `./target/debug/ecaz` binary used for the one-cell
    suite validation.
- One-cell suite validation:
  `./target/debug/ecaz --host /Users/peter/.pgrx --port 28818 --log-file reviews/task-71/007-stop-condition-closeout/artifacts/suite-one-cell.log bench suite run --config reviews/task-71/003-worker-curve/suite.json --only load-real10k-w2 --artifact-dir reviews/task-71/007-stop-condition-closeout/artifacts/one-cell-suite`
  - passed.
  - Loader timing line:
    `requested_workers=2 workers_launched=2`
  - Generated `suite-manifest.json` now records
    `parallel_workers_before=0`, `parallel_workers_after=2`,
    `parallel_workers_delta=2`.
  - Generated `results.jsonl` has a `parallel_workers` row with
    `before=0`, `after=2`, `delta=2`, matching the loader timing line.
- `cargo pgrx test pg18 test_ec_ivf_parallel_build`
  - passed without approval escalation.
  - Target pg_tests passed:
    `test_ec_ivf_parallel_build_workers_and_counts` and
    `test_ec_ivf_parallel_build_matches_serial_structure`.

## Review Focus

- Whether invoking the Task 71 Stop Condition is the right closeout for the
  measured Option A M5 result.
- Whether the packet 001 addendum answers the earlier design feedback.
- Whether the task status wording is acceptable for a completed
  stop-condition closeout.
