---
task: 50
packet: reviews/task-50/230-spire-customscan-path-add-boundaries
head_sha: c78e35e1ab8e5fd3c3cecd799a451cae327d741d
timestamp: 2026-05-21T04:58:17-07:00
lane: SPIRE custom scan planner unsafe burndown
storage_format: n/a
rerank_mode: n/a
surface: PostgreSQL planner hook path construction
---

# Manifest

## Code Checkpoint

- Commit: `c78e35e1ab8e5fd3c3cecd799a451cae327d741d`
- Summary: inlined the SPIRE custom scan path-add helper boundaries into `ec_spire_set_rel_pathlist_hook`.
- Removed unsafe helper boundaries:
  - `add_custom_scan_path`
  - `add_dml_pk_select_custom_scan_path`
- Source unsafe count:
  - Previous pushed packet count: `2517`
  - This packet count: `2513`
  - Delta: `-4`

## Validation Artifacts

- `artifacts/rustfmt-check.log`
  - Command: `rustfmt --check src/am/ec_spire/custom_scan/planner.rs`
  - Result: passed; emitted only the existing stable-rustfmt warnings for `imports_granularity` and `group_imports`.
- `artifacts/git-diff-check.log`
  - Command: `git diff --check`
  - Result: passed with no output.
- `artifacts/src-unsafe-count.log`
  - Command: `rg -n 'unsafe' src | wc -l`
  - Result: `2513`.
- `artifacts/cargo-check-pg18-bench.log`
  - Command: `cargo check --all-targets --no-default-features --features pg18,bench`
  - Result: passed; emitted the known existing `src/am/mod.rs` unused SPIRE re-export warning.
- `artifacts/cargo-test-custom-scan-pg18-pg-test-no-run.log`
  - Command: `cargo test --lib custom_scan --no-default-features --features pg18,pg_test --no-run`
  - Result: passed; emitted the known existing Hadamard test helper dead-code warnings.

## Notes

- This was not a benchmark packet.
- No isolated index/table benchmark surface was used.
- The vector custom scan path branch preserves the prior helper-local return behavior by only skipping vector path construction when `CustomScanPlannerRel::new` fails.
- The DML PK-select branch remains the final hook branch; its early return on missing planner relation view is equivalent to the removed helper.
