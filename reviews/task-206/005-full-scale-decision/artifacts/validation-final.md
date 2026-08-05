# Final validation

- `PGRX_PG_CONFIG_PATH=/home/peter/.pgrx/18.3/pgrx-install/bin/pg_config cargo check --offline --all-targets --no-default-features --features pg18` — passed.
- `PGRX_PG_CONFIG_PATH=/home/peter/.pgrx/18.3/pgrx-install/bin/pg_config cargo test --offline --lib --no-default-features --features pg18 distann_orchestration_accepts_maximum_beam_width_without_budget_overflow` — passed; 1 test, 0 failed, 2557 filtered out.
- `/home/peter/.cargo-target/release/ecaz bench suite audit --config reviews/task-206/004-corrected-closeout/artifacts/task206-corrected.json` — passed, 3 steps.
- `/home/peter/.cargo-target/release/ecaz bench suite audit --config reviews/task-207/004-search-and-sharding/artifacts/task207-corrected.json` — passed, 6 steps.
- `git diff --check` — passed; worktree clean after the packet commit.

