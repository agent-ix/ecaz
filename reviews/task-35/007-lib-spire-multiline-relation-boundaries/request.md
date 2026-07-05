# Review Request: lib.rs SPIRE Multiline Relation Boundaries

Head: `e7c6732d4ef25ca7cfc98afae63f1040c2f1b8dc`

Scope:
- `src/lib.rs`
- `scripts/unsafe_comment_baseline.txt`
- `reviews/task-35/007-lib-spire-multiline-relation-boundaries/request.md`
- `reviews/task-35/007-lib-spire-multiline-relation-boundaries/artifacts/*`

What changed:
- Reused the `with_live_index_relation!` wrapper from packet 006 for remaining
  multi-line SQL wrapper calls that pass a validated `IndexRelationGuard` into
  SPIRE AM helper functions.
- Removed the remaining `unsafe { am::...(index_relation.as_ptr(), ...) }`
  blocks in `src/lib.rs` that fit the live index-relation invariant.
- Left non-relation-boundary unsafe sites untouched: `_PG_init`, `StringInfo`
  receive buffers, typmod detoast, query parsing, and DML frontdoor plan-tree
  inspection.

Baseline result:
- Start: 3,579 entries across 107 files.
- End: 3,510 entries across 107 files.
- Net reduction: 69 baseline entries.
- `src/lib.rs` start/end: 103 entries to 34 entries.

Review focus:
- Confirm the converted multi-line SPIRE helper calls all happen while the
  validated `IndexRelationGuard` remains in scope.
- Confirm the wrapper is not being used for unrelated raw pointer invariants.
- Confirm the remaining `src/lib.rs` unsafe sites are correctly outside this
  packet’s live-relation-boundary scope.

Validation:
- `make unsafe-baseline-report` before baseline update
  - artifact: `artifacts/unsafe-baseline-before.log`
- `bash scripts/check_unsafe_comments.sh` before baseline update
  - artifact: `artifacts/audit-before.log`
  - result: passed; the old baseline was stale but still covered all current
    missing unsafe lines before refresh.
- `cargo fmt --all`
  - artifact: `artifacts/fmt.log`
  - result: passed; rustfmt emitted existing stable-toolchain warnings for
    unstable `rustfmt.toml` options.
- `cargo check --all-targets --no-default-features --features pg18,bench`
  - artifact: `artifacts/cargo-check-pg18.log`
  - result: passed with existing warnings from `src/am/common/parallel.rs` and
    `src/am/mod.rs`.
- `bash scripts/check_unsafe_comments.sh --update-baseline`
  - artifact: `artifacts/update-baseline.log`
- `make unsafe-baseline-report` after baseline update
  - artifact: `artifacts/unsafe-baseline-after.log`
- `bash scripts/check_unsafe_comments.sh` after baseline update
  - artifact: `artifacts/audit-after.log`
  - result: passed with no output.
- `git diff --check`
  - artifact: `artifacts/git-diff-check.log`
  - result: passed with no output.

Tests skipped:
- No PostgreSQL runtime behavior was intended to change; validation used
  `cargo check` because this packet changes Rust wrapper structure around many
  SQL-callable SPIRE diagnostic functions.
