# Task 40 Follow-up Closeout Artifact Manifest

- Head SHA: `53ea3d8d46ba60ee803a4429788c53db4bbd64a4`
- Task bucket: `reviews/task-40/`
- Packet path: `reviews/task-40/003-task-40-followups/`
- Timestamp: `2026-05-18T19:34:03-07:00`
- Isolated one-index-per-table vs shared-table surface: not applicable. These
  are model-checking, simulation, governance, and compile-validation lanes, not
  live PostgreSQL table/index measurement runs.

## Artifacts

### `shuttle-real.log`

- Lane: `shuttle-real`
- Command: `bash scripts/hardening.sh shuttle-real`
- Key result: `test result: ok. 2 passed; 0 failed`.
- Coverage note: epoch-publish visibility now uses Shuttle `RwLock`, with the
  scanner observing through read locks while the writer owns the publish span
  through a write lock.

### `sim-spire-remote.log`

- Lane: `sim-spire-remote`
- Command: `bash scripts/hardening.sh sim-spire-remote`
- Key result: `test result: ok. 5 passed; 0 failed`.

### `sim-spire-remote-deep-smoke.log`

- Lane: `sim-spire-remote-deep` plumbing smoke.
- Command: `SIM_SPIRE_REMOTE_DEEP_SEEDS=2 make sim-spire-remote-deep`
- Key result: command invokes `SIM_SPIRE_SEEDS=2 bash scripts/hardening.sh
  sim-spire-remote`; `test result: ok. 5 passed; 0 failed`.
- Note: this is a plumbing smoke only. The deferred post-Task-35 deep pass is
  expected to use the default `SIM_SPIRE_REMOTE_DEEP_SEEDS=1000`.

### `loom-real.log`

- Lane: `loom-real`
- Command: `bash scripts/hardening.sh loom-real`
- Key result: `test result: ok. 6 passed; 0 failed`.
- Coverage note: validates the lifted worker-slot and HNSW concurrent DSM
  insert-state protocols after the PG-side CAS bridge was corrected.

### `concurrent-dsm-no-run.log`

- Lane: production compile validation.
- Command: `cargo test --lib concurrent_dsm --no-run`
- Key result: lib test executable produced.
- Notes: only pre-existing Hadamard helper `dead_code` warnings were emitted.

### `hardening-validate.log`

- Lane: governance validation.
- Command: `bash scripts/hardening_validate.sh`
- Key result: command exited with `COMMAND_EXIT_CODE="0"`.

### `shuttle-injected-bug.log`

- Lane: negative-control injected bug for `shuttle-real`.
- Temporary break: flipped the candidate merge score comparator so the higher
  score won a duplicate-dedupe merge.
- Key result: `candidate_merge_is_order_invariant_under_concurrent_receive`
  failed with selected input `[0]` instead of `[1]`; Shuttle printed a failing
  schedule and seed.
- Reverted before the code commit.

### `sim-spire-injected-bug.log`

- Lane: negative-control injected bug for `sim-spire-remote`.
- Temporary break: disabled served-epoch mismatch rejection in the transport
  simulation model.
- Key result: `turmoil_strict_rejects_stale_served_epoch_response` failed at
  seed `0`; the stale response was accepted as one ready dispatch instead of
  zero.
- Reverted before the code commit.
