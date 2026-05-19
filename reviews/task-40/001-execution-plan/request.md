# Task 40 Execution Plan: Real Concurrency Model Checking

Task: `plan/tasks/40-concurrency-model-checking-real.md`
Branch: `task-40`
Head at plan creation: `74554335980988d9f3beb6825911b937c14178a4`
Code checkpoint under review: `a6a428b081bdb1fa9f00d51d0ceb18165a108204`
Date: 2026-05-18

## Research Summary

Task 40 replaces the Task 34 synthetic Loom/Shuttle placeholders with model
checks that exercise real ECAZ concurrency protocols. Current governance in
`docs/hardening-governance.md` explicitly retired synthetic
`hardening/loom` and `hardening/shuttle`; those lanes can return only when they
import real `src/` code and have reviewer-visible signal evidence.

The current checkout has no `hardening/loom` or `hardening/shuttle` crates.
Existing pure hardening lanes path-lift real modules into standalone crates
(`hardening/careful`, `hardening/kani`) to keep PostgreSQL callback symbols out
of model-check binaries. Task 40 should use the same pattern.

## Loom Coverage Inventory

High-value Loom targets:

1. `src/am/common/parallel.rs` worker-slot protocol.
   - Protocol: initialize slots, claim with CAS, publish runtime snapshot,
     release with stale-epoch rejection, reset on rescan.
   - Invariants: exclusive slot ownership, coordinator claim count equals live
     claimed slots, stale releases/publishes cannot mutate a new epoch, released
     slots expose idle runtime state.
   - Plan: lift slot state into a pgrx-free helper and use it from production
     `parallel.rs` plus `hardening/loom`.
2. `src/am/ec_hnsw/build_parallel.rs` concurrent DSM node insert state.
   - Protocol: `UNINSERTED -> INSERTING -> READY`, neighbor slot publication,
     backlink mutation under per-node locks.
   - Invariants: one inserter per node, readers never traverse an inserting
     node as ready, ready publication happens after neighbor slots are written,
     duplicate partition ownership cannot double-insert.
   - Plan: lift a small node-state helper first; larger graph/search locking
     gets Shuttle coverage after the state helper is in place.
3. `src/am/ec_hnsw/build_parallel.rs` shared build completion accounting.
   - Protocol: workers record tuple counts and signal `workersdonecv`.
   - Invariants: participant completion count is monotonic and exactly one
     completion is recorded per participant.
   - Plan: lower priority than worker slots and DSM node state because the
     correctness impact is accounting/coordination rather than index visibility.
4. `src/am/common/explain.rs`, `src/am/common/stats.rs`, and SPIRE custom-scan
   debug atomics.
   - Protocol: one-time registration flags and monotonic counters.
   - Invariants: counters are monotonic; registration is idempotent.
   - Plan: document as out of first Task 40 code slice unless a reviewer asks
     for broad low-value counter coverage.

Not suitable for Loom as-is:

- PostgreSQL `LWLock`, `ConditionVariable`, `shm_mq`, DSM allocation, and
  pgrx callbacks. Model only lifted Rust protocols around these PG primitives.
- Tokio/libpq remote transport. This belongs in deterministic simulation
  (`madsim`/`turmoil`) or a pure state-machine model, not Loom.

## Shuttle / Simulation Inventory

Shuttle targets:

1. SPIRE candidate receive and compact merge in
   `src/am/ec_spire/coordinator/remote_candidates/production_transport.rs` and
   tests under `remote_candidates/tests/production_executor_state.rs`.
   Invariants: candidate-merge order invariance, duplicate vector collapse,
   strict/degraded failure categorization, cancellation discards retained
   batches.
2. SPIRE epoch publish and replacement scheduler in `src/am/ec_spire/update/`.
   Invariants: active epoch monotonicity, publish-lock recheck rejects drift,
   scanners observe old or new manifests but never partial replacement drafts.
3. HNSW concurrent DSM graph assembly beyond per-node claim state.
   Invariants: graph insertion reaches all partition-owned nodes, ready nodes
   have valid neighbor slots, backlinks never expose out-of-range node IDs.

Madsim/turmoil targets:

1. SPIRE remote candidate transport is Tokio/libpq based
   (`remote_candidates/dispatch.rs`, `tls.rs`, `production_transport.rs`), so a
   deterministic network simulation lane is justified.
2. First simulation should model dispatch lifecycle and remote result receive
   as a pure transport trait before trying to simulate PostgreSQL/libpq itself.
   Scenarios: packet reorder/drop/duplicate, partition/heal, timeout/cancel,
   tail latency, and stale served-epoch response.

## Fine-Grained Execution Plan

1. Reintroduce only a real Loom lane.
   - Add `hardening/loom/` as a standalone crate.
   - Update `scripts/hardening_validate.sh` so `hardening/loom` is allowed only
     when it path-imports real `src/` code.
   - Add `make loom-real` and a `scripts/hardening.sh loom-real` lane.
2. Lift the parallel worker-slot protocol.
   - Add a pgrx-free module under `src/am/common/`.
   - Keep production `EcParallelWorkerSlot` layout and PG-facing callback code
     in `parallel.rs`.
   - Route claim/release/runtime snapshot mutation through the lifted helper.
3. Model-check the lifted worker-slot helper.
   - Cover two workers racing for one slot.
   - Cover four workers claiming/releasing two slots.
   - Cover publish versus release ordering.
   - Cover rescan/stale-epoch rejection.
   - Assert claim count and slot state after every modeled interleaving.
4. Extend Loom to HNSW concurrent DSM node state.
   - Lift `UNINSERTED/INSERTING/READY` transition logic.
   - Model two participants racing to insert the same node.
   - Model reader visibility around ready publication.
   - Keep full graph search and backlink mutation for Shuttle.
5. Reintroduce Shuttle only after the Loom slice is green.
   - Add `hardening/shuttle/` as a real path-import crate.
   - Start with SPIRE candidate receive/merge state because it already has a
     pure testable surface.
   - Then add SPIRE epoch publish/replacement scheduler models.
6. Add deterministic SPIRE remote simulation after Shuttle.
   - Extract a pure remote transport trait/state adapter.
   - Add either `crates/ecaz-sim-spire` or `hardening/sim-spire`.
   - Use madsim/turmoil only for the async/network lifecycle, not for PG FFI.
7. Documentation and governance.
   - Update `docs/hardening.md` with the lift-then-model pattern.
   - Update `docs/hardening-governance.md` inventory and tier placement.
   - Ensure `make hardening-validate` rejects synthetic-only returns.
8. Validation packets.
   - For each slice, store logs under the owning packet `artifacts/`.
   - Use `make loom-real` for first evidence.
   - Do not promote to PR/nightly until the injected-bug check produces a
     reproducible counterexample.

## First Code Slice

The first implementation slice will complete steps 1-3: a real `loom-real`
lane over the lifted parallel worker-slot protocol. This gives Task 40 a real
model-checking foothold on a production concurrency protocol with limited blast
radius and a clear invariant set.

## First Code Slice Result

Implemented in this checkpoint:

- Added `src/am/common/parallel_slot.rs`, a pgrx-free helper for the real
  parallel worker-slot state machine.
- Routed production `src/am/common/parallel.rs` claim, publish, release, and
  snapshot reads through the lifted helper.
- Added a real `hardening/loom` crate that path-imports
  `src/am/common/parallel_slot.rs`.
- Added `make loom-real` / `scripts/hardening.sh loom-real`.
- Updated hardening governance docs and validation so returned model-checking
  lanes must import real `src/` code.

The first Loom run exposed a publish/release interleaving where a publisher
could write a non-idle runtime snapshot after release reset but before the slot
became free. The fix adds explicit transient states:

- `CLAIMED -> PUBLISHING -> CLAIMED`
- `CLAIMED -> RELEASING -> FREE`

Release waits for `PUBLISHING` to finish before resetting and freeing the slot.
The final Loom run covers exclusive claim, live claim count, publish/release
ordering, and stale-epoch rejection.

## Validation

- `bash scripts/hardening.sh loom-real`: pass, 4 Loom tests.
  Log: `reviews/task-40/001-execution-plan/artifacts/loom-real.log`
- `bash scripts/hardening_validate.sh`: pass.
  Log: `reviews/task-40/001-execution-plan/artifacts/hardening-validate.log`
- `cargo test --lib parallel_scan --no-run`: pass, production test binary
  compiles. Log:
  `reviews/task-40/001-execution-plan/artifacts/parallel-scan-no-run.log`

`cargo test --lib parallel_scan` was not used as final evidence because the
standalone lib test binary needs PostgreSQL/pgrx runtime symbols outside the
normal pgrx harness.
