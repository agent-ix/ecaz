# Task 40 Model-Checking Completion

Review requested for the completed Task 40 model-checking campaign.

## Scope

- Reintroduced real Loom coverage under `hardening/loom/` and productionized
  the worker-slot state into `src/am/common/parallel_slot.rs`.
- Extended Loom coverage to HNSW concurrent DSM node insert state through
  `src/am/ec_hnsw/concurrent_dsm_state.rs`.
- Reintroduced Shuttle under `hardening/shuttle/` over path-lifted SPIRE
  candidate-merge and epoch-publish helpers.
- Added deterministic SPIRE remote simulation under `hardening/sim-spire/`
  using Turmoil UDP delivery over
  `src/am/ec_spire/coordinator/remote_candidates/remote_transport_sim_model.rs`.
- Updated hardening governance/docs and Makefile/script lane inventory for
  `loom-real`, `shuttle-real`, and `sim-spire-remote`.

## Execution-Plan Coverage

- Parallel worker slots: exclusive claim, four-worker/two-slot pressure,
  publish/release ordering, stale-epoch rejection, and live claim accounting
  are covered by `loom-real`.
- HNSW concurrent DSM insert state: exclusive `UNINSERTED -> INSERTING ->
  READY` publication and reader visibility after neighbor-slot publication are
  covered by `loom-real`.
- SPIRE candidate receive/merge: concurrent receive order invariance and
  duplicate-dedupe winner selection are covered by `shuttle-real`.
- SPIRE epoch publish/replacement scheduling: scanner visibility never exposes
  an in-progress replacement as active, covered by `shuttle-real`.
- SPIRE deterministic remote transport: async delivery, partition/degraded
  skip, stale served-epoch rejection, extracted adapter behavior, and stable
  governance names are covered by `sim-spire-remote`.
- Governance: `hardening_validate.sh` accepts the returned real lanes and
  continues to reject synthetic-only hardening crates.

## Evidence

See `artifacts/manifest.md` for commands, SHA, and key result lines. The final
artifact set at head `a587790950c7f06d51768f1dea23421c62bbeb13` is:

- `artifacts/loom-real.log`: `6 passed; 0 failed`.
- `artifacts/shuttle-real.log`: `2 passed; 0 failed`.
- `artifacts/sim-spire-remote.log`: `5 passed; 0 failed`.
- `artifacts/production-merge-no-run.log`: production lib test binary compiled.
- `artifacts/hardening-validate.log`: command exited `0`.

Tests that require live PostgreSQL execution were not run for this packet; the
touched behavior is covered by pgrx-free model-checking lanes plus production
compile validation.
