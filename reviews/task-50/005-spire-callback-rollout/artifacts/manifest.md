# Task 50 Packet 005 Artifact Manifest

- Head SHA: `0650e3d761a287f2283912804917c3ff1f5959e4`
- Task bucket: `reviews/task-50/005-spire-callback-rollout/`
- Lane: SPIRE callback structural unsafe reduction.
- Fixture/storage/rerank mode: not applicable; compile/count-only callback
  shape change.
- Isolated vs shared table surface: not applicable.
- Timestamp: `2026-05-20T05:21:40Z`

## Artifacts

### `block-count-before.log`

- Command:
  `git show HEAD:<path> | rg -c 'unsafe\s*\{'` for each touched path before the
  code commit.
- Key lines:
  - `34 src/am/ec_spire/vacuum/mod.rs`
  - `22 src/am/ec_spire/cost/mod.rs`
  - `21 src/am/ec_spire/insert.rs`
  - `4 src/am/ec_spire/scan/callbacks.rs`
  - `2 src/am/common/callback.rs`
  - `0 src/am/ec_spire/scan.rs`

### `block-count-after.log`

- Command:
  `make unsafe-block-count PATHS='src/am/common/callback.rs src/am/ec_spire/scan.rs src/am/ec_spire/scan/callbacks.rs src/am/ec_spire/cost/mod.rs src/am/ec_spire/insert.rs src/am/ec_spire/vacuum/mod.rs'`
- Key lines:
  - `31 src/am/ec_spire/vacuum/mod.rs`
  - `20 src/am/ec_spire/insert.rs`
  - `18 src/am/ec_spire/cost/mod.rs`
  - `2 src/am/common/callback.rs`

### `cargo-check-pg18-bench.log`

- Command:
  `cargo check --all-targets --no-default-features --features pg18,bench`
- Result: pass.
- Key line: `Finished 'dev' profile ...`

### `rustfmt-touched-check.log`

- Command:
  `rustfmt --check src/am/ec_spire/scan.rs src/am/ec_spire/scan/callbacks.rs src/am/ec_spire/insert.rs src/am/ec_spire/cost/mod.rs src/am/ec_spire/vacuum/mod.rs src/am/common/callback.rs`
- Result: pass.

### `cargo-fmt-check.log`

- Command: `cargo fmt --all --check`
- Result: fail due to pre-existing repo formatting drift.
- Key files:
  - `hardening/careful/src/spire_diagnostics_helpers.rs`
  - `src/quant/simd.rs`

### `cargo-clippy-pg18.log`

- Command:
  `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings`
- Result: fail due to existing repo-wide clippy backlog.
- Key result: final log has no diagnostics for
  `src/am/ec_spire/{cost/mod,insert,scan,scan/callbacks,vacuum/mod}.rs` or
  `src/am/common/callback.rs`.

### `git-diff-check.log`

- Command: `git diff --check`
- Result: pass.
