# Review Request: Remove Ecaz-CLI Superseded Scripts

Current head: `dbd02fa`

Scope:
- `crates/ecaz-cli/`
- `docs/`
- `plan/`
- `scripts/`
- `spec/`

Problem:
- `ecaz-cli` landed on `main` as the supported corpus / benchmark / compare /
  stress entry surface, but the repo still carried the older shell and Python
  wrappers for the same jobs.
- That left two competing operator paths in-tree and kept live docs/spec/task
  text pointing at deleted-by-policy tooling instead of the supported CLI.
- The stale script-side tests also only exercised the deprecated wrappers, not
  the shipped `ecaz-cli` commands.

What changed:
- Removed the deprecated script entry points now covered by `ecaz-cli`:
  - corpus generation / preparation / load / recall harness wrappers
  - latency / storage / overhead benchmark wrappers
  - pgvector comparison wrappers
  - vacuum stress wrapper
- Removed the script-only tests for those deleted entry points.
- Updated live docs, specs, and task text to point at `ecaz` commands instead of
  the deleted scripts.
- Updated `ecaz-cli` help/comments that still referred to the removed script
  names.
- Committed formatter output encountered in the touched Rust files so the tree
  stays clean after the tooling transition.

Still intentionally retained under `scripts/`:
- `run_pgrx_pg17_test.sh`
- `run_pg18_preload_pgstat_test.sh`
- scratch-cluster helpers and their focused tests
- repo maintenance helpers like `check_unsafe_comments.sh`

Validation:
- Passed:
  - `cargo test -p ecaz-cli`
  - `cargo test`
  - `bash scripts/run_pgrx_pg17_test.sh`
  - `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`

Review focus:
- Whether any live repo surface still points at the deleted deprecated scripts
- Whether the retained `scripts/` helpers are the right boundary after the
  `ecaz-cli` transition, rather than additional CLI-covered leftovers
- Whether the doc/spec/task updates reflect the supported operator workflow
