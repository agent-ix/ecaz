# Review Request: Unify Ecaz-CLI Dev And Test Tooling

Current head: `64258fc`

Scope:
- `AGENTS.md`
- `crates/ecaz-cli/`
- `docs/`
- `plan/`
- `scripts/`
- `spec/`

Problem:
- `main` still had two parallel operator paths for local setup/testing:
  - the newer `ecaz-cli` corpus / bench / compare / stress surface
  - older wrapper scripts for pgrx tests, PG18 preload validation, scratch
    cluster control, and local install helpers
- That split kept live docs and checkpoint text pointing at deleted-by-policy
  wrapper scripts instead of one supported surface.
- The generic CLI groundwork from the other agent also needed to land onto
  `main` so the Task 18 and DiskANN lanes could both build on the same
  connection / setup / testing contract instead of drifting.

What changed:
- Added a first-class `ecaz dev` subtree:
  - `ecaz dev test pgrx --pg <17|18>`
  - `ecaz dev test pg18-preload-pgstat`
  - `ecaz dev install ecaz-pg-test`
  - `ecaz dev install pgvector`
  - `ecaz dev scratch restart`
  - `ecaz dev scratch sql`
  - `ecaz dev scratch refresh-debug-helpers`
- Added global connection flags to the CLI: `--host`, `--port`, `--user`,
  `--password`, alongside the existing `--database`.
- Moved the scratch debug-helper SQL asset under `crates/ecaz-cli/sql/`.
- Deleted the superseded wrapper scripts:
  - `scripts/run_pgrx_pg17_test.sh`
  - `scripts/run_pg18_preload_pgstat_test.sh`
  - `scripts/install_adr030_pg17_pg_test.sh`
  - `scripts/install_pgvector_pg17_scratch.sh`
  - `scripts/restart_adr030_scratch.sh`
  - `scripts/refresh_adr030_scratch_debug_helpers.sh`
  - `scripts/pg17_scratch_psql.sh`
  - `scripts/resolve_scratch_socket_dir.sh`
- Updated live docs/spec/task/checkpoint text to use direct cargo lanes or the
  new `ecaz dev` commands instead of the deleted wrappers.

Validation:
- Passed:
  - `cargo test -p ecaz-cli`
  - `cargo run -p ecaz-cli -- --help`
  - `cargo run -p ecaz-cli -- dev --help`
  - `cargo run -p ecaz-cli -- dev test --help`
  - `cargo test`
  - `cargo run -p ecaz-cli -- dev test pgrx --pg 17`
  - `cargo run -p ecaz-cli -- dev test pg18-preload-pgstat`
  - `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`

Review focus:
- Whether the new `ecaz dev` surface is the right long-term home for local
  setup/testing helpers that do not belong in `make`
- Whether any live repo text still points at removed wrapper scripts
- Whether the global connection flags and shared pgrx install discovery are
  generic enough for both the Task 18 and DiskANN lanes to build on without
  introducing a third operator path
