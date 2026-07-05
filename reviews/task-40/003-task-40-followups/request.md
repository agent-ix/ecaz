# Task 40 Follow-up Closeout

Review requested for the closeout packet requested by
`reviews/task-40/002-model-checking-completion/feedback/2026-05-18-02-reviewer.md`.

## Scope

- Strengthened the Shuttle epoch-publish visibility model from a single
  serialized `Mutex<SpireEpochPublishModel>` to Shuttle `RwLock`, with writer
  publication under a write lock and scanner observations through read locks.
- Added sim-spire seed sweep plumbing through `SIM_SPIRE_SEEDS=N`.
- Added `make sim-spire-remote-deep`, defaulting to
  `SIM_SPIRE_REMOTE_DEEP_SEEDS=1000`, while leaving the normal local lane at
  one seed.
- Fixed `PgLockedDsmInsertStateCell` in place: load/store now use PostgreSQL
  atomic helpers and compare-exchange now calls
  `pg_atomic_compare_exchange_u32`, matching the CAS semantics assumed by the
  lifted HNSW concurrent DSM model.
- Captured negative-control injected-bug runs for both `shuttle-real` and
  `sim-spire-remote`, then reverted the temporary breaks.
- Marked `plan/tasks/40-concurrency-model-checking-real.md` as
  `closed pending deep-coverage knob bump`.

## Injected-Bug Evidence

- `shuttle-real`: temporarily flipped the candidate merge score comparator so
  the worse duplicate won. `shuttle-injected-bug.log` shows
  `candidate_merge_is_order_invariant_under_concurrent_receive` failing with
  selected input `[0]` instead of `[1]`, plus Shuttle replay data.
- `sim-spire-remote`: temporarily disabled served-epoch mismatch rejection.
  `sim-spire-injected-bug.log` shows
  `turmoil_strict_rejects_stale_served_epoch_response` failing at seed `0`
  because the stale response was accepted as ready.

## Evidence

See `artifacts/manifest.md` for command lines and key result lines. At head
`53ea3d8d46ba60ee803a4429788c53db4bbd64a4`:

- `artifacts/shuttle-real.log`: `2 passed; 0 failed`.
- `artifacts/sim-spire-remote.log`: `5 passed; 0 failed`.
- `artifacts/sim-spire-remote-deep-smoke.log`: two-seed deep-target plumbing
  smoke passed.
- `artifacts/loom-real.log`: `6 passed; 0 failed`.
- `artifacts/concurrent-dsm-no-run.log`: production lib test binary compiled.
- `artifacts/hardening-validate.log`: command exited `0`.
- `artifacts/shuttle-injected-bug.log`: expected failure captured.
- `artifacts/sim-spire-injected-bug.log`: expected failure captured.

The only intentionally deferred items are the post-Task-35 deep-coverage knob
bumps: increasing Shuttle’s default/random budget and making the full sim-spire
1000-seed pass the normal selected budget.
