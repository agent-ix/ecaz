# Artifact manifest

- head SHA: `56c6ed63111c49caa7689b86fff2a7da2e6b6dde`
- task bucket: `reviews/task-41`
- packet path: `reviews/task-41/147-invariant2-completion-audit`
- timestamp: `2026-05-18T04:25:20Z`
- lane: Task 41 invariant #2 completion audit
- fixture: code-level validation and fresh static inventories
- storage format: all AMs covered by Task 41 invariant #2
- rerank mode: not applicable
- isolated one-index-per-table surface: not applicable

## Artifacts

### `code-commit-stat.log`

- command: `git show --stat --oneline HEAD`
- result: recorded code commit `56c6ed63` and changed file stats.

### `final-memory-lifetime-inventory.log`

- command: `rg -n 'from_raw_parts\\(|from_raw_parts_mut\\(|slice::from_raw_parts|slice::from_raw_parts_mut|CStr::from_ptr|pg_detoast_datum|pg_detoast_datum_packed|tts_values|varlena_to_byte_slice' src/am src/lib.rs -g '*.rs'`
- result: recorded final raw memory-lifetime inventory for classification in
  `request.md`.

### `detoast-inventory.log`

- command: `rg -n 'pg_detoast_datum(_packed)?\\(|varlena_to_byte_slice' src/am src/lib.rs -g '*.rs'`
- result: remaining hits are guard internals.

### `slot-datum-inventory.log`

- command: `rg -n 'tts_values|tts_isnull|ExecClearTuple' src/am -g '*.rs'`
- result: remaining hits are audited immediate reads, by-value writes, or slot
  clear calls.

### `task41-packet-index.log`

- command: `find reviews/task-41 -maxdepth 2 -name request.md -printf '%h\\n' | sed 's#reviews/task-41/##' | sort`
- result: recorded packet sequence through packet 146 before this request was
  written.

### `cargo-fmt-check.log`

- command: `cargo fmt --all --check`
- result: passed
- key lines: stable rustfmt emitted the repository's existing warnings that
  `imports_granularity` and `group_imports` require nightly.

### `cargo-check-pg18.log`

- command: `cargo check --no-default-features --features pg18`
- result: passed
- key lines: `Finished dev profile` after one known pre-existing unused imports
  warning in `src/am/mod.rs`.

### `git-diff-check-head-build-parallel-and-packet.log`

- command: `git diff --check HEAD -- src/am/ec_hnsw/build_parallel.rs reviews/task-41/147-invariant2-completion-audit`
- result: passed

### `git-status.log`

- command: `git status --short --branch`
- result: recorded branch ahead count and unrelated dirty comparator/benchmark
  files.
