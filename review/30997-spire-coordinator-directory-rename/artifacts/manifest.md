# Artifacts: 30997 SPIRE Coordinator Directory Rename

- Head SHA before code commit: `ff56b5400997e3342ecb9ae95edd6507b8025674`
- Packet/topic: `30997-spire-coordinator-directory-rename`
- Lane / fixture / storage format / rerank mode: Phase 12b.5 mechanical Rust source layout rename; no storage/rerank behavior touched
- Isolated one-index-per-table or shared-table surfaces: not applicable; source-layout-only change

## Validation

### `cargo-check-pg18.log`

- Command: `script -q -e -c "cargo check --no-default-features --features pg18" review/30997-spire-coordinator-directory-rename/artifacts/cargo-check-pg18.log`
- Timestamp: 2026-05-13 12:39 PDT
- Key result lines:
  - `Finished dev profile`
  - pre-existing unused-import warning in `src/am/mod.rs`

### `cargo-fmt-check.log`

- Command: `script -q -e -c "cargo fmt --check" review/30997-spire-coordinator-directory-rename/artifacts/cargo-fmt-check.log`
- Timestamp: 2026-05-13 12:39 PDT
- Key result lines:
  - command exited `0`
  - rustfmt emitted the repository's existing stable-channel warnings for unstable `imports_granularity` / `group_imports` options

### Rename sanity artifacts

- `git-grep-root-module-paths.log`
  - Command: `script -q -e -c "git grep -n root:: -- src/am/ec_spire" ...`
  - Result: exit `1`, no matches. There were no Rust `root::` module paths to update because the old directory was include-only.
- `src-stale-root-path-search.log`
  - Command: `script -q -e -c "rg -n 'include!\\(\\\"root/|src/am/ec_spire/root|ec_spire/root/' src/am/ec_spire" ...`
  - Result: exit `1`, no stale source include/path matches.
- `coordinator-file-count.log`
  - Command: `script -q -e -c "find src/am/ec_spire/coordinator -maxdepth 3 -type f | sort | wc -l" ...`
  - Result: `31`
- `stale-root-path-search.log`
  - Broader task/source search retained for context; remaining matches are historical tracker wording in `plan/tasks/task30-phase12b-spire-cleanup.md`, not source includes.
