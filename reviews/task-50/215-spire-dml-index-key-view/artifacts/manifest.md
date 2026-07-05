# Artifact Manifest: SPIRE DML Index Key View

- head SHA: `e26cb564023b98d1ec8c1c2a6b570cf6e650fe98`
- task bucket: `reviews/task-50`
- packet path: `reviews/task-50/215-spire-dml-index-key-view`
- timestamp: `2026-05-21T10:47:19Z`
- storage format / rerank mode: not applicable; code-only unsafe burn-down slice
- isolated one-index-per-table or shared-table surface: not applicable; no benchmark or corpus run

## Artifacts

### `rustfmt-check.log`

- command: `rustfmt --check src/am/ec_spire/dml_frontdoor/mod.rs`
- result: passed
- key lines: stable-channel warnings for `imports_granularity` and `group_imports`; no formatting diff.

### `git-diff-check.log`

- command: `git diff --check`
- result: passed
- key lines: no output.

### `cargo-check-pg18-bench.log`

- command: `cargo check --all-targets --no-default-features --features pg18,bench`
- result: passed
- key lines: existing unused-import warning in `src/am/mod.rs`; `Finished dev profile`.

### `cargo-test-dml-frontdoor-pg18-pgtest-no-run.log`

- command: `cargo test --lib dml_frontdoor --no-default-features --features pg18,pg_test --no-run`
- result: passed
- key lines: existing Hadamard test-helper dead-code warnings; `Finished test profile`; unit-test executable built.

## Unsafe Counts

- `git grep -n "unsafe" HEAD^ -- src | wc -l`: `2554`
- `git grep -n "unsafe" HEAD -- src | wc -l`: `2552`
- `git grep -n "unsafe" HEAD^ -- src/am/ec_spire/dml_frontdoor/mod.rs | wc -l`: `71`
- `git grep -n "unsafe" HEAD -- src/am/ec_spire/dml_frontdoor/mod.rs | wc -l`: `69`
- `git grep -n "unsafe fn" HEAD^ -- src/am/ec_spire/dml_frontdoor/mod.rs | wc -l`: `20`
- `git grep -n "unsafe fn" HEAD -- src/am/ec_spire/dml_frontdoor/mod.rs | wc -l`: `20`

