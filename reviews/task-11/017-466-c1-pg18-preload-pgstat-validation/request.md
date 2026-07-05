# Review Request: PG18 Preload pgstat Validation

Current head: `d8ea5ad`

Scope:
- `scripts/run_pg18_preload_pgstat_test.sh`
- `docs/contributing.md`
- `plan/plan.md`
- `plan/status.md`
- `plan/tasks/11-planner.md`
- `plan/tasks/19-pg18-completion.md`
- `plan/tasks/README.md`
- `spec/functional/FR-025-custom-statistics.md`
- `spec/functional/FR-027-pgrx-pg18-upgrade.md`
- `spec/tests.md`

Problem:
- The shared PG18 pgstat path was already wired, but the repo only had the
  ordinary non-preloaded validation lane.
- That meant `cargo pgrx test pg18` covered the backend-local fallback path,
  but not the actual preload-time registration contract that makes the shared
  counters live across backends.
- Live task/status/spec text still described preload-aware PG18 pgstat
  validation as remaining follow-up work.

What changed:
- Added `scripts/run_pg18_preload_pgstat_test.sh`.
- The script:
  - uses the installed PG18 pgrx toolchain already present under `~/.pgrx`
  - starts a repo-local PG18 cluster under `target/pg18-preload-pgstat`
  - forces `shared_preload_libraries = 'ecaz'`
  - retries ports upward from the requested base if one is already in use
  - creates the extension plus a small `ec_hnsw` fixture
  - verifies `ec_hnsw_planner_integration_snapshot(...)` clears the PG18
    blocker under preload
  - runs a scan in one backend and verifies from another backend that
    `ecaz_stats()` sees the shared counter delta
- Updated the live task/plan/status/spec surfaces so they no longer claim
  preload-aware PG18 pgstat coverage is still missing.
- Added the corresponding test inventory entry in `spec/tests.md`.

Validation:
- Passed:
  - `cargo test`
  - `bash scripts/run_pgrx_pg17_test.sh`
  - `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`
  - `cargo pgrx test pg18`
  - `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings`
  - `bash scripts/run_pg18_preload_pgstat_test.sh`

Review focus:
- Whether the preload script proves the shared pgstat path rather than merely
  rechecking the backend-local fallback
- Whether the repo-local cluster approach is the right containment boundary for
  this PG18 preload validation lane
- Whether the task/plan/spec text now reflects the post-validation state
  accurately without overstating what remains
- Whether the script’s port-retry and cleanup behavior is safe and predictable
