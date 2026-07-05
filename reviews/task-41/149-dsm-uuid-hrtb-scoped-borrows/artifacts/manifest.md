# Artifact manifest

- head SHA: `705b2c940e06b40297b2973e74430e2851f3ff5a`
- task bucket: `reviews/task-41`
- packet path: `reviews/task-41/149-dsm-uuid-hrtb-scoped-borrows`
- timestamp: `2026-05-18T04:58:50Z`
- lane: Task 41 invariant #2 DSM / fixed payload scoped borrows
- fixture: code-level validation and scoped-helper inventory
- storage format: HNSW DSM, HNSW/IVF page read helpers, SPIRE UUID payload
- rerank mode: not applicable
- isolated one-index-per-table surface: not applicable

## Artifacts

### `code-commit-stat.log`

- command: `git show --stat --oneline 705b2c94`
- result: recorded code commit `705b2c94` and changed file stats.

### `scoped-helper-inventory.log`

- command: `rg -n "with_concurrent_dsm|with_uuid_payload_bytes|with_page_line_tuple_bytes|with_required_page_tuple_bytes|for<'a> FnOnce|from_raw_parts_mut\(parts\.(neighbor_slots|codes|sources)" src/am/ec_hnsw/build_parallel.rs src/am/ec_hnsw/shared.rs src/am/ec_hnsw/scan_debug.rs src/am/ec_ivf/page.rs src/am/ec_spire/build/drafts.rs`
- result: DSM init and runtime helpers, UUID payload helper, and page read
  helpers now show scoped helper or HRTB signatures; the previous open-coded
  DSM init `from_raw_parts_mut(parts.neighbor_slots|codes|sources)` patterns
  are absent.

### `cargo-fmt-check.log`

- command: `cargo fmt --all --check`
- result: passed
- key lines: stable rustfmt emitted the repository's existing warnings that
  `imports_granularity` and `group_imports` require nightly.

### `cargo-check-pg18.log`

- command: `cargo check --no-default-features --features pg18`
- result: passed
- key lines: `Finished dev profile` after the known pre-existing unused
  imports warning in `src/am/mod.rs`.

### `git-diff-check-code-commit.log`

- command: `git diff --check 705b2c94^ 705b2c94`
- result: passed

### `git-status.log`

- command: `git status --short --branch`
- result: recorded packet worktree state after code commit.
