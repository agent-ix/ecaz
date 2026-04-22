# Review Request: Parallel Scan LWLock Ereport Coverage

Current head: `33ecc67`

Scope:
- `src/am/common/parallel.rs`
- `src/lib.rs`

Problem:
- The staged PG18 serializer had already moved to a real `LWLock`, but the
  branch still lacked an integration-level check that an error raised while
  holding that lock unwinds cleanly and allows reacquire on the same runtime
  lock surface afterward.
- Reviewer packet `511` explicitly called for stronger runtime-path coverage
  here before planner-visible parallel enablement.

What changed:
- Added a backend-local PG18 parallel-scan descriptor fixture under
  `#[cfg(any(test, feature = "pg_test"))]` so tests can exercise the real
  embedded coordinator `LWLock` without widening the production runtime
  surface.
- Added two narrow debug helpers in `src/am/common/parallel.rs`:
  - one that acquires and releases the coordinator `LWLock`
  - one that acquires the same `LWLock` and then raises `ereport(ERROR)`
- Added a PG18 `#[pg_test]` in `src/lib.rs` that:
  - proves the lock is initially acquirable
  - raises and catches the held-lock PostgreSQL error via `PgTryBuilder`
  - proves the same runtime `LWLock` is acquirable again after unwind

Why this matters:
- This is the missing runtime-path coverage for the serializer swap.
- It tests the actual PG18 error/unwind contract instead of only unit-test
  shims or backend-local happy paths.

Feedback processed in this slice:
- This directly addresses the outstanding reviewer concern from packet `511`
  about lacking integration-level lock semantics coverage for the new
  parallel-scan `LWLock` serializer.

Still intentionally deferred:
- final cross-worker ownership transfer instead of deferred local retention
- planner-visible parallel execution and `amcanparallel = true`
- `n=2/4/8` correctness and measurement after the remaining ownership seam
  lands

Validation:
- Passed:
  - `cargo test`
  - `cargo run -p ecaz-cli -- dev test pgrx --pg 18`
  - `cargo run -p ecaz-cli -- dev test pg18-preload-pgstat`
  - `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings`

Review focus:
- Whether the backend-local descriptor fixture is the right containment
  boundary for exercising the runtime `LWLock`
- Whether the `PgTryBuilder` error-path test is a sufficient integration
  proof for held-lock `ereport(ERROR)` unwind and reacquire behavior
