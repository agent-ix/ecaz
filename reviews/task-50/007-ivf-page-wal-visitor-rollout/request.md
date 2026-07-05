# Task 50 Packet 007: IVF Page WAL Visitor Rollout

## Code Under Review

- Commit: `c6718871c314db5f5a7af5b844ffbc58ecc313a7`
- Task: `plan/tasks/50-unsafe-structural-reduction.md`
- Slice: 2, IVF/RaBitQ page tuple visitor rollout.

## Scope

This packet completes the first IVF page-reader rollout started in Packet 006.
It adds:

- `PageTupleWriter` for WAL-registered tuple slot validation and exact-size
  in-place rewrites;
- `WalRegisteredPage` for page initialization, item insertion, special-area
  metadata access, free-space accounting, and posting delete helpers;
- safe private read/visit helpers where `LockedBufferGuard`, `PageTupleReader`,
  or `PageTupleWriter` now encode the local page/tuple invariant.

The WAL transaction lifecycle is unchanged: callers still start the generic WAL
transaction, register the buffer, perform the page mutation, and finish/drop in
the same control-flow positions. The new helpers only centralize page pointer,
line-pointer, tuple-slot, and exact-size-copy checks.

## Unsafe Block Count

```text
file | task-start | packet-before | after | task delta | task percent | top-15 target status
src/am/ec_ivf/page.rs | 134 | 122 | 90 | -44 | -32.8% | complete for this top-15 file
```

Packet-local delta: `122 -> 90` (`-32`, `-26.2%`).

## Risk Register

Relevant row: `IVF page tuple visitor`.

- Failure mode: tuple bounds, item-id interpretation, error ordering, or WAL
  mutation scope drift.
- Mitigation: keep the existing raw tuple-byte helper as the only byte-slice
  exposure point; keep WAL start/register/finish positions in the original
  functions; split read-only and WAL-registered helpers.
- Verification: compile check, touched-file formatting, direct block-count
  delta, focused page-test attempt, and exploratory page-codec microbench run.

## Validation

- PASS: `cargo check --all-targets --no-default-features --features pg18,bench`
  - Log: `artifacts/cargo-check-pg18-bench.log`
- PASS: touched-file `rustfmt --check src/am/ec_ivf/page.rs`
  - Log: `artifacts/rustfmt-touched-check.log`
- PASS: `git diff --check`
  - Log: `artifacts/git-diff-check.log`
- FAIL, host runtime limitation:
  `cargo test --lib --no-default-features --features pg18,bench am::ec_ivf::page:: -- --nocapture`
  - Log: `artifacts/cargo-test-ivf-page-pg18.log`
  - The test binary builds, then fails to execute outside PostgreSQL with
    `undefined symbol: LockBuffer`.
- FAIL, pre-existing repo drift: `cargo fmt --all --check`
  - Log: `artifacts/cargo-fmt-check.log`
  - Existing formatting diffs remain in `hardening/careful/src/spire_diagnostics_helpers.rs`
    and `src/quant/simd.rs`.
- FAIL, pre-existing repo-wide clippy backlog:
  `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings`
  - Log: `artifacts/cargo-clippy-pg18.log`
  - The final clippy log has no diagnostics for `src/am/ec_ivf/page.rs`.
- PASS command, exploratory only:
  `cargo bench --features bench --bench page_codec`
  - Log: `artifacts/criterion-page-codec-after.log`
  - Note: this bench exercises `bench_api::DataPage`, not `src/am/ec_ivf/page.rs`
    directly. Criterion reports mixed changes against its local stored baseline,
    so this artifact is retained for visibility but is not a clean IVF/RaBitQ
    before/after gate.

Benchmark baseline reference: `benchmarks/task-50-local-baseline/`.
