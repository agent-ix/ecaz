# Task 50 Packet 005: SPIRE Callback Rollout

## Code Under Review

- Commit: `0650e3d761a287f2283912804917c3ff1f5959e4`
- Task: `plan/tasks/50-unsafe-structural-reduction.md`
- Slice: 1c, SPIRE callback rollout.

## Scope

This packet applies the shared Task 50 AM callback guard helper to SPIRE callback
entry points in:

- `src/am/ec_spire/scan/callbacks.rs`
- `src/am/ec_spire/cost/mod.rs`
- `src/am/ec_spire/insert.rs`
- `src/am/ec_spire/vacuum/mod.rs`
- `src/am/ec_spire/scan.rs` only for the callback macro import used by the
  included callback module.

The change removes repeated outer `unsafe { pgrx::pgrx_extern_c_guard(...) }`
wrappers from SPIRE AM callbacks and replaces them with `pg_am_callback!` or
`am_callback`. The callback-duration FFI boundary invariant remains centralized
in `src/am/common/callback.rs`; raw PostgreSQL pointer dereferences inside each
callback body remain visible at the local use sites.

This packet intentionally does not touch SPIRE coordinator snapshots,
read-efficiency internals, DML frontdoor surfaces, or page/object storage
helpers. Those are later Task 50 SPIRE anchor/read-path slices.

## Unsafe Block Count

Before:

```text
  34 src/am/ec_spire/vacuum/mod.rs
  22 src/am/ec_spire/cost/mod.rs
  21 src/am/ec_spire/insert.rs
   4 src/am/ec_spire/scan/callbacks.rs
   2 src/am/common/callback.rs
   0 src/am/ec_spire/scan.rs
```

After:

```text
  31 src/am/ec_spire/vacuum/mod.rs
  20 src/am/ec_spire/insert.rs
  18 src/am/ec_spire/cost/mod.rs
   2 src/am/common/callback.rs
```

The touched SPIRE callback surface drops from 83 to 71 direct
`unsafe { ... }` blocks. `src/am/ec_spire/scan/callbacks.rs` drops from 4 to 0.

## Validation

- PASS: `cargo check --all-targets --no-default-features --features pg18,bench`
  - Log: `artifacts/cargo-check-pg18-bench.log`
- PASS: touched-file `rustfmt --check`
  - Log: `artifacts/rustfmt-touched-check.log`
- PASS: `git diff --check`
  - Log: `artifacts/git-diff-check.log`
- FAIL, pre-existing repo drift: `cargo fmt --all --check`
  - Log: `artifacts/cargo-fmt-check.log`
  - Existing formatting diffs remain in `hardening/careful/src/spire_diagnostics_helpers.rs`
    and `src/quant/simd.rs`.
- FAIL, pre-existing repo-wide clippy backlog:
  `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings`
  - Log: `artifacts/cargo-clippy-pg18.log`
  - After cleaning three simple existing diagnostics in the touched vacuum file,
    the final clippy log has no diagnostics for the touched files in this
    packet.

No runtime benchmark was run. This slice keeps the same
`pgrx_extern_c_guard` boundary shape behind the shared helper and does not
change SPIRE scoring, traversal, cache, placement, or read-efficiency logic.
