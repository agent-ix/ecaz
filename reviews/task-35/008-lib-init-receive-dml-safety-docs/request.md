# Review Request: lib.rs Init Receive and DML Safety Docs

Head: `43c62b62b30f0f8e6c612cac6d0234c45f4d6fc9`

Scope:
- `src/lib.rs`
- `scripts/unsafe_comment_baseline.txt`
- `reviews/task-35/008-lib-init-receive-dml-safety-docs/request.md`
- `reviews/task-35/008-lib-init-receive-dml-safety-docs/artifacts/*`

What changed:
- Documented the remaining `src/lib.rs` unsafe sites with nearby, specific
  `// SAFETY:` comments.
- Covered extension initialization hooks, PostgreSQL `StringInfo` receive
  buffers, typmod detoast/array decoding, DML frontdoor query analysis, and
  immediate plan-tree/catalog inspection.
- Removed `src/lib.rs` from `scripts/unsafe_comment_baseline.txt`.

Baseline result:
- Start: 3,510 entries across 107 files.
- End: 3,476 entries across 106 files.
- Net reduction: 34 baseline entries.
- `src/lib.rs` start/end: 34 entries to 0 entries.

Review focus:
- Confirm the comments state real invariants rather than restating the unsafe
  operation.
- Confirm `StringInfo` comments cover non-null, length/cursor bounds, message
  cursor consumption, and returned byte-slice lifetimes.
- Confirm DML query comments correctly limit raw `Query` pointers to immediate
  inspection in the current PostgreSQL memory context.

Validation:
- `make unsafe-baseline-report` before baseline update
  - artifact: `artifacts/unsafe-baseline-before.log`
- `bash scripts/check_unsafe_comments.sh` before baseline update
  - artifact: `artifacts/audit-before.log`
  - result: passed with no output.
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
- No Rust or PostgreSQL behavior changed; this packet adds safety
  documentation only.
