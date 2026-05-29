# Task 48/005: CI matrix workflows (closes §Exit Criteria #1)

## Scope

Closes Task 48 §Exit Criteria gate 1:

> 1. CI matrix covers at least: aarch64-darwin, x86_64-linux-gnu,
>    aarch64-linux-gnu, pg17, pg18.

Adds six new GitHub Actions workflow files under
`.github/workflows/`, plus a Mondays 02:00 UTC soak run and a
nightly resource-exhaustion run that depend on the harnesses from
packets 001 and 004.

Validation head: post-commit landing this slice (commit immediately
after `9ecfa83ea` for the Task 48/004 code).

## What changed

| File | Lane | Cadence | Targets |
|---|---|---|---|
| `build-matrix-x86_64-linux-gnu.yml` | check + clippy + test | per-PR | pg17, pg18 |
| `build-matrix-aarch64-darwin.yml` | check + clippy + compile-only | per-PR | pg18 (macOS `_BufferBlocks` blocker) |
| `build-matrix-aarch64-linux-gnu.yml` | check + clippy (per-PR), full test (nightly) | per-PR + nightly | pg18 (Graviton) |
| `build-matrix-nightly-toolchain.yml` | miri-expanded + miri-tree + miri-many-seeds + fuzz-all-short | nightly | nightly Rust toolchain |
| `resource-exhaustion-nightly.yml` | `make resource-exhaustion` against pre-configured cluster | nightly | pg18 |
| `soak-weekly.yml` | `make soak DURATION=24h` + JSON artifact upload | weekly (Mon 02:00 UTC) | pg18 |

Combined with the existing `ci.yml` (production primary lanes), the
matrix covers every triple + cadence row in
`docs/build-matrix.md`.

## Cadence implementation

| Cadence | Workflows | Failure policy |
|---|---|---|
| per-PR | `ci.yml`, `build-matrix-x86_64-linux-gnu.yml`, `build-matrix-aarch64-darwin.yml`, `build-matrix-aarch64-linux-gnu.yml` (compile job) | blocking |
| nightly | `build-matrix-aarch64-linux-gnu.yml` (full job), `build-matrix-nightly-toolchain.yml`, `resource-exhaustion-nightly.yml` | blocks merge after first failure |
| weekly | `soak-weekly.yml` | informational (24h artifact upload) |

## Reviewer focus

- Per-PR workflows are concise: install Rust + PG headers,
  `cargo check`, `cargo clippy`, `cargo test`. No bespoke logic.
  Cache-key per matrix axis so the workspace's huge `target/` is
  reused.
- macOS workflow runs only `cargo check` + `cargo clippy` + a
  `cargo build -p ecaz-cli` per the `_BufferBlocks` dyld blocker
  policy in `feedback_dyld_buffer_blocks_known` /
  `docs/build-matrix.md`. Runtime tests on macOS are deliberately
  out of scope until that blocker is resolved.
- The Graviton (`aarch64-unknown-linux-gnu`) workflow splits
  per-PR (compile) from nightly (full test) so PR latency stays
  reasonable on the slower ARM runner.
- `resource-exhaustion-nightly.yml` does its own cluster
  pre-configuration (low `max_locks_per_transaction`,
  `max_connections`, `shared_buffers`) so the restart-only-GUC
  scenarios in packet 004 actually exercise their limit cases.
- `soak-weekly.yml` has a 1500-minute timeout (25h, plus buffer)
  matching the 24h `DURATION` default. The JSON artifact upload
  is preserved across runs via `actions/upload-artifact@v4` with
  a run-id key.

## Limitations

- ARM macOS runners (`macos-14`) and ARM Linux runners
  (`ubuntu-24.04-arm`) are first-party GitHub-hosted runners.
  Spend per minute is higher than x86_64 runners; CI governance
  (Task 49) may downgrade the ARM per-PR lane to nightly-only if
  spend exceeds budget. The workflow files are written so that
  flipping `on:` from `pull_request` to `schedule` is a one-line
  change.
- The `disk-full` scenario in `resource-exhaustion-nightly.yml`
  surfaces as `PrereqUnmet` because the ENOSPC injector lives in
  `make fault-full` (Task 38). A future slice can collapse the
  two by invoking the fault-injection crate directly from
  `ecaz dev resource-test` — out of scope here.

## Task 48 §Exit Criteria progress after this slice

| # | §Exit Criterion | Status |
|---|---|---|
| 1 | CI matrix covers aarch64-darwin + x86_64-linux + aarch64-linux + pg17 + pg18 | **✓ done (this)** |
| 2 | `make soak DURATION=24h` weekly | ✓ done (003) + this (weekly cadence) |
| 3 | `make resource-exhaustion` nightly | ✓ done (004) + this (nightly cadence) |
| 4 | `docs/build-matrix.md` documents matrix, cadence, policy | ✓ done (002) |

Task 48 ≈ **100% complete** — all 4 §Exit gates closed.
