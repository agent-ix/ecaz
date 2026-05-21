# Artifact Manifest

- head SHA: `a25cc50e3e41caefd9b235d637e41876bff4411c`
- task bucket: `reviews/task-50`
- packet path: `reviews/task-50/254-spire-common-pg-pointer-helpers`
- timestamp: `2026-05-21T15:15:27Z`
- lane: SPIRE planner/custom-scan pointer helper cleanup
- fixture: local compile/static validation
- storage format: not applicable
- rerank mode: not applicable
- table/index isolation: not applicable

## Artifacts

- `artifacts/rustfmt-files.log`
  - Command: `rustfmt --edition 2021 --check src/am/common/pg_ptr.rs src/am/common/mod.rs src/am/ec_spire/custom_scan/mod.rs src/am/ec_spire/dml_frontdoor/mod.rs`
  - Result: passed; stable rustfmt emitted existing unstable-option warnings.
- `artifacts/git-diff-check.log`
  - Command: `git diff --check`
  - Result: passed.
- `artifacts/cargo-check-pg18-bench.log`
  - Command: `cargo check --all-targets --no-default-features --features pg18,bench`
  - Result: passed; existing unused SPIRE re-export warning in `src/am/mod.rs`.
- `artifacts/cargo-test-lib-pg18-no-run.log`
  - Command: `cargo test --lib --no-default-features --features pg18,pg_test --no-run`
  - Result: passed; existing Hadamard test-helper dead-code warnings.
- `artifacts/unsafe-counts.log`
  - Command: counted `unsafe` lines in `src`, the touched SPIRE files, and
    the new common pointer helper before and after the code change.
  - Key lines:
    - `HEAD src unsafe count: 2409`
    - `working src unsafe count: 2405`
    - `HEAD DML unsafe count: 68`
    - `working DML unsafe count: 64`
    - `HEAD custom scan cost unsafe count: 26`
    - `working custom scan cost unsafe count: 22`
    - `working common pg_ptr unsafe count: 4`
