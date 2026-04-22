# Review Request: Parallel Scan LWLock Serializer

Current head: `e3e083c`

Scope:
- `src/am/common/parallel.rs`
- `csrc/standalone_pg_backend_stubs.c`
- `plan/tasks/18-parallel-index-scan.md`

Problem:
- The staged parallel-scan coordinator still serialized shared heap mutation
  with a raw atomic lock word.
- That was acceptable for narrow staging, but it is not the right runtime
  contract for real PostgreSQL parallel execution and had already been called
  out in review as a must-fix before `amcanparallel = true`.

What changed:
- `EcParallelCoordinatorHeapState` now embeds a real PostgreSQL `LWLock`
  instead of an `AtomicU32` lock word.
- Parallel-scan descriptor initialization now:
  - allocates a tranche id
  - registers a named tranche (`ecaz_parallel_scan_dsm`)
  - initializes the DSM-local `LWLock`
- Parallel-scan attachment validation now re-registers the stored tranche id
  before using the shared coordinator lock in an attaching backend.
- The old heap-lock helper/guard naming is now aligned with the broader role:
  `acquire_parallel_scan_coordinator_lock(...)` and
  `ParallelScanCoordinatorLockGuard`.
- The standalone unit-test backend now provides the minimal `LWLock*` symbols
  needed for linking.
- `#[cfg(test)]` keeps a local atomic shim over the embedded `LWLock.state`
  field so Rust unit tests do not trip `pgrx`'s cross-thread FFI guard while
  the runtime path still uses real `LWLockAcquire` / `LWLockRelease`.
- Task 18 notes now record that the staged raw lock word is gone and that the
  remaining blocker is the handoff/ownership contract, not the serializer type.

Why this matters:
- It removes one of the major remaining “staged only” shortcuts from the
  shared coordinator path.
- The coordinator serializer now matches PostgreSQL’s real parallel runtime
  expectations closely enough that the next remaining blocker is the actual
  multi-worker output handoff, not the lock primitive itself.

Feedback processed in this slice:
- Packets `509` and `510` both came back accepted.
- Their feedback narrowed the next step toward the serializer swap instead of
  more blocker-snapshot expansion.
- This slice addresses the older cumulative must-fix reviewer note that the
  raw coordinator lock word had to be replaced before real parallel enablement.

Still intentionally deferred:
- the real multi-worker output handoff / ownership transfer contract
- planner-visible parallel execution and `amcanparallel = true`
- `n = 2/4/8` correctness coverage once the real multi-worker path lands

Validation:
- Passed:
  - `cargo test`
  - `cargo run -p ecaz-cli -- dev test pgrx --pg 18`
  - `cargo run -p ecaz-cli -- dev test pg18-preload-pgstat`
  - `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings`

Review focus:
- Whether the DSM-local LWLock tranche initialization / attach-time
  re-registration boundary is the right one for this AM-private descriptor
- Whether the test-only atomic shim is an appropriate containment boundary for
  Rust unit tests without masking real runtime lock semantics
