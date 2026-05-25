# Task 30 / Packet 967 Artifact Manifest

- head SHA: `f671da96423a6863a723262a8f7da3c78ed786d5`
- task bucket: `reviews/task-30/967-spire-phase13e-leaf-assignment-export`
- timestamp: `2026-05-25T17:29:39Z`
- lane: SPIRE Phase 13e AWS production gap closure
- fixture: coordinator leaf base assignment export
- storage format: SPIRE leaf V1/V2 base assignment rows
- rerank mode: not applicable
- isolated one-index-per-table: not applicable
- shared-table surfaces: not applicable

## Artifacts

### `cargo-check-ecaz-lib.log`

- command: `cargo check -p ecaz --lib`
- result: pass
- key lines: `Finished dev profile`

### `bash-n-register.log`

- command: `bash -n scripts/spire-aws/register.sh`
- result: pass
- key lines: command exited successfully with no syntax diagnostics

### `cargo-fmt-check.log`

- command: `cargo fmt --all -- --check`
- result: pass
- key lines: command exited successfully; log contains existing stable-rustfmt warnings for ignored nightly-only import options

### `git-diff-check.log`

- command: `git diff --check HEAD`
- result: pass
- key lines: command exited successfully with no whitespace diagnostics
