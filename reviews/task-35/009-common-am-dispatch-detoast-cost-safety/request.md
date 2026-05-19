# Review Request: Common AM Dispatch Detoast and Cost Safety

Head: `0e4eae93b20f7e7702d854d9b209b44360057df3`

Scope:
- `src/am/mod.rs`
- `src/am/common/detoast.rs`
- `src/am/common/cost.rs`
- `scripts/unsafe_comment_baseline.txt`
- `reviews/task-35/009-common-am-dispatch-detoast-cost-safety/request.md`
- `reviews/task-35/009-common-am-dispatch-detoast-cost-safety/artifacts/*`

What changed:
- Documented AM dispatch wrappers in `src/am/mod.rs` that forward live
  relation pointers into AM-specific snapshot helpers.
- Documented detoast ownership, borrowed slice lifetime, and pfree invariants
  in `src/am/common/detoast.rs`.
- Documented planner cost globals, PG18 planner callbacks, `amcostestimate`
  output pointers, relation metadata reads, and metadata-page reads in
  `src/am/common/cost.rs`.
- Removed all baseline entries for these three files.

Baseline result:
- Start: 3,476 entries across 106 files.
- End: 3,448 entries across 103 files.
- Net reduction: 28 baseline entries.
- `src/am/mod.rs`, `src/am/common/detoast.rs`, and
  `src/am/common/cost.rs` start/end: 28 entries to 0 entries.

Review focus:
- Confirm wrapper comments in `src/am/mod.rs` preserve the caller contract:
  the raw relation pointer must remain live for the delegated helper call.
- Confirm detoast comments distinguish borrowed varlena pointers from
  palloc-owned detoast copies.
- Confirm planner callback comments cover PG-owned pointers and output slots,
  not just the existence of unsafe blocks.

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
