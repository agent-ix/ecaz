# Review Request: lib.rs SQL FFI Index Relation Boundaries

Head: `bd523a12c0577f43cf2e93453196c1e37b4ff210`

Scope:
- `src/lib.rs`
- `scripts/unsafe_comment_baseline.txt`
- `reviews/task-35/006-lib-sql-ffi-index-relation-boundaries/request.md`
- `reviews/task-35/006-lib-sql-ffi-index-relation-boundaries/artifacts/*`

What changed:
- Added `with_live_index_relation!`, a narrow wrapper for SQL functions that
  have already opened and AM-validated an `IndexRelationGuard` before calling
  an unsafe AM diagnostic helper.
- Converted repeated `unsafe { am::...(index_relation.as_ptr(), ...) }`
  call sites in `src/lib.rs` to the wrapper macro.
- Made `relation_oid_exists` safe by moving its catalog-read invariant inside
  the helper, then removed repeated unsafe blocks around existence checks.
- Left lower-level `StringInfo`, typmod detoast, query parsing, and plan-tree
  unsafe sites for follow-up `src/lib.rs` slices because their invariants are
  different from the live index-relation boundary handled here.

Baseline result:
- Start: 3,657 entries across 107 files.
- End: 3,579 entries across 107 files.
- Net reduction: 78 baseline entries.
- `src/lib.rs` start/end: 181 entries to 103 entries.

Review focus:
- Confirm the macro is narrow enough and does not hide unrelated unsafe: it
  only accepts an opened `IndexRelationGuard`, forwards its raw relation
  pointer, and keeps the guard in scope for the AM helper call.
- Confirm the converted SQL wrappers all validate the index relation before
  calling the macro.
- Confirm making `relation_oid_exists` safe is acceptable because the helper
  filters `InvalidOid` and only reads catalog relkind metadata.

Validation:
- `make unsafe-baseline-report` before baseline update
  - artifact: `artifacts/unsafe-baseline-before.log`
- `bash scripts/check_unsafe_comments.sh` before baseline update
  - artifact: `artifacts/audit-before.log`
  - result: failed as expected after line movement/source changes and before
    refreshing the generated baseline.
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
  SQL-callable diagnostic functions.
