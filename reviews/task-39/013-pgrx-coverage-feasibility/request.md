# Review Request: Task 39 pgrx Coverage Feasibility

Task: `plan/tasks/39-test-quality-measurement.md`

Implementation commit: `b56b386f515a0f32e6bb063b9d9ef8c47024e1d7`

## Scope

This packet records the Task 39 pgrx instrumentation decision requested by the
reviewer plan:

- probes `cargo pgrx test pg18` under `RUSTFLAGS="-C instrument-coverage"`;
- records that the instrumented pgrx test profile can emit `.profraw` files;
- records that the probe does not reach live backend tests because the lib test
  binary aborts with `dyld` failing to resolve `_BufferBlocks`;
- records that `LLVM_PROFILE_FILE` must be absolute for this lane, because
  relative paths are resolved from child process working directories and emit
  profile-write errors;
- documents the decision in `docs/hardening.md`: full pgrx callback coverage is
  not promoted into `make coverage` yet; the supported Task 39 coverage surface
  remains the shim-based `ecaz-cli` plus `hardening/careful` subset until a
  future packet proves clean backend coverage and profile merge.

## Validation

Packet-local evidence is under `artifacts/`; see `artifacts/manifest.md`.

- Relative-profile probe: failed before live backend tests; emitted profile path
  errors and then aborted on `_BufferBlocks`.
- Absolute-profile probe: failed before live backend tests; same `_BufferBlocks`
  abort. The run reused cached instrumented artifacts, so relative profile
  errors from those artifacts are still visible in the log.
- Profile listing: the first probe emitted 149 `.profraw` files before aborting;
  the absolute-path rerun emitted 0 new files before aborting.
- `git diff --check`: clean.

## Decision

Task 39 should continue treating live PG18/pgrx callback coverage as not stable
for CI or baseline ratchets. Callback-heavy files remain explicit gaps until a
future packet fixes the pgrx lib-test loader issue, uses absolute profile
paths, reaches backend execution, and demonstrates a merged coverage report for
the target callback files.

## Remaining Task 39 Gaps

This packet closes the feasibility-decision item. Remaining structural gaps are
IVF/SPIRE page codec coverage raises, SPIRE storage/coordinator coverage raises,
storage guard coverage/mutation, broader critical-module mutation triage, and
scheduled CI burn-in evidence.
