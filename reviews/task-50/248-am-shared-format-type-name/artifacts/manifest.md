# Artifact Manifest

- head SHA: `b8e76bd989500d619b7203764940a1f7763ca83c`
- task bucket: `reviews/task-50`
- packet path: `reviews/task-50/248-am-shared-format-type-name`
- timestamp: `2026-05-21T14:43:59Z`
- lane: cross-AM P7 C-string contract cleanup; IVF and SPIRE
- fixture: local compile/static validation
- storage format: not applicable
- rerank mode: not applicable
- table/index isolation: not applicable

## Artifacts

- `artifacts/rustfmt-files.log`
  - Command: `rustfmt --edition 2021 --check src/am/ec_ivf/build.rs src/am/ec_spire/dml_frontdoor/mod.rs`
  - Result: passed; stable rustfmt emitted existing unstable-option warnings.
- `artifacts/git-diff-check.log`
  - Command: `git diff --check`
  - Result: passed.
- `artifacts/cargo-check-pg18-bench.log`
  - Command: `cargo check --all-targets --no-default-features --features pg18,bench`
  - Result: passed; existing unused SPIRE re-export warning in `src/am/mod.rs`.
- `artifacts/cargo-test-ec-ivf-pg18-no-run.log`
  - Command: `cargo test --lib ec_ivf --no-default-features --features pg18,pg_test --no-run`
  - Result: passed; existing Hadamard test-helper dead-code warnings.
- `artifacts/cargo-test-ec-spire-pg18-no-run.log`
  - Command: `cargo test --lib ec_spire --no-default-features --features pg18,pg_test --no-run`
  - Result: passed; existing Hadamard test-helper dead-code warnings.
- `artifacts/unsafe-counts.log`
  - Command: counted `unsafe` lines in `src`, `src/am/ec_ivf/build.rs`, and
    `src/am/ec_spire/dml_frontdoor/mod.rs` before and after the code change.
  - Key lines:
    - `HEAD src unsafe count: 2443`
    - `working src unsafe count: 2441`
    - `HEAD IVF build unsafe count: 19`
    - `working IVF build unsafe count: 18`
    - `HEAD SPIRE DML unsafe count: 69`
    - `working SPIRE DML unsafe count: 68`
