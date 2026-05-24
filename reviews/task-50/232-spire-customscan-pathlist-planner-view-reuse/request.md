---
task: 50
packet: 232
topic: spire-customscan-pathlist-planner-view-reuse
role: coder
status: ready-for-review
created: 2026-05-21T05:04:53-07:00
head_sha: c7d06397a59632ddcfe7fc56a23d93d3182a13bc
---

# Review Request: SPIRE Pathlist Planner View Reuse

## Summary

This packet removes two redundant unsafe `CustomScanPlannerRel::new(root, rel)` calls from `ec_spire_set_rel_pathlist_hook`.

The hook already builds a `CustomScanRelPathlistInput`, which validates the same planner callback pointers and stores `planner_rel`. Both CustomPath construction branches now use that existing view.

## Safety Notes

- `CustomScanRelPathlistInput::new` remains the sole relation-view boundary for this hook.
- The vector and DML CustomPath builders now operate on the already-validated planner relation view.
- This avoids re-reading the same raw planner pointers after candidate selection.

## Unsafe Count

- Previous packet count: `2508`
- Current count: `2506`
- Delta: `-2`

The packet-local count log is `artifacts/src-unsafe-count.log`.

## Validation

- `artifacts/rustfmt-check.log`: `rustfmt --check src/am/ec_spire/custom_scan/planner.rs` passed with only known stable-rustfmt config warnings.
- `artifacts/git-diff-check.log`: `git diff --check` passed.
- `artifacts/cargo-check-pg18-bench.log`: `cargo check --all-targets --no-default-features --features pg18,bench` passed with the known existing `src/am/mod.rs` unused SPIRE re-export warning.
- `artifacts/cargo-test-custom-scan-pg18-pg-test-no-run.log`: `cargo test --lib custom_scan --no-default-features --features pg18,pg_test --no-run` passed with the known existing Hadamard helper dead-code warnings.

