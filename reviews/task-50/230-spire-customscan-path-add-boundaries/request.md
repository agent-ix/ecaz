---
task: 50
packet: 230
topic: spire-customscan-path-add-boundaries
role: coder
status: ready-for-review
created: 2026-05-21T04:58:17-07:00
head_sha: c78e35e1ab8e5fd3c3cecd799a451cae327d741d
---

# Review Request: SPIRE Custom Scan Path Add Boundaries

## Summary

This packet removes two single-use unsafe helper boundaries from SPIRE custom scan planner path construction:

- `add_custom_scan_path`
- `add_dml_pk_select_custom_scan_path`

The path construction logic now lives directly inside `ec_spire_set_rel_pathlist_hook`, so the remaining unsafe blocks are adjacent to the PostgreSQL planner hook inputs and the `add_path` ownership transfer.

## Safety Notes

- The vector custom scan path branch keeps the removed helper's behavior: if `CustomScanPlannerRel::new(root, rel)` cannot build a relation view, only vector path construction is skipped and the DML candidate branch can still run.
- The DML PK-select branch is still the final branch in the hook, so returning when the planner relation view cannot be built remains equivalent to the removed helper's local return.
- The `CustomPath` and `custom_private` list are still allocated in PostgreSQL planner memory and handed to PostgreSQL through `add_path`.

## Unsafe Count

- Previous pushed packet count: `2517`
- Current count: `2513`
- Delta: `-4`

The packet-local count log is `artifacts/src-unsafe-count.log`.

## Validation

- `artifacts/rustfmt-check.log`: `rustfmt --check src/am/ec_spire/custom_scan/planner.rs` passed with only known stable-rustfmt config warnings.
- `artifacts/git-diff-check.log`: `git diff --check` passed.
- `artifacts/cargo-check-pg18-bench.log`: `cargo check --all-targets --no-default-features --features pg18,bench` passed with the known existing `src/am/mod.rs` unused SPIRE re-export warning.
- `artifacts/cargo-test-custom-scan-pg18-pg-test-no-run.log`: `cargo test --lib custom_scan --no-default-features --features pg18,pg_test --no-run` passed with the known existing Hadamard helper dead-code warnings.

