# Static Remote Leaf Materialization Artifacts

- head SHA: `8b47b9adaca63c4c8ece26ce35e89466af2a00e8`
- task bucket: `reviews/task-30`
- packet path: `reviews/task-30/968-spire-phase13e-static-remote-leaf-materialization`
- timestamp: `2026-05-25T17:41:21Z`
- lane: SPIRE Phase 13e AWS remote placement
- fixture: static SQL/compile validation, no live AWS fixture in this packet
- storage format: SPIRE relation object store, leaf V2 objects
- rerank mode: not applicable
- table isolation: AWS prefix-derived coordinator and remote corpus tables, one index per table

## Artifacts

- `bash-n-register.log`
  - command: `bash -n scripts/spire-aws/register.sh`
  - result: `COMMAND_EXIT_CODE="0"`
- `cargo-check-ecaz-lib.log`
  - command: `cargo check -p ecaz --lib`
  - result: `Finished dev profile [unoptimized + debuginfo]`
  - result: `COMMAND_EXIT_CODE="0"`
- `cargo-fmt-check.log`
  - command: `cargo fmt --all -- --check`
  - result: `COMMAND_EXIT_CODE="0"`
  - note: log includes existing stable-rustfmt warnings for nightly-only import grouping options.
- `git-diff-check.log`
  - command: `git diff --check`
  - result: `COMMAND_EXIT_CODE="0"`

