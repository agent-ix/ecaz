---
task: 50
packet: 231
topic: spire-customscan-expression-list-view-boundary
role: coder
status: ready-for-review
created: 2026-05-21T05:02:02-07:00
head_sha: c9250b493ae623971d2c6207ea1f0675008efe45
---

# Review Request: SPIRE CustomScan Expression List View Boundary

## Summary

This packet removes the generic unsafe PostgreSQL List helpers used by the SPIRE DML CustomScan expression handoff:

- `custom_scan_list_len`
- `custom_scan_list_nth_node`

`CustomScanExprList` now captures the verified expression-list length during construction from a live provider-owned `CustomScan` plan node. Its safe accessors operate only through that concrete view and perform local bounds checks before calling `list_nth`.

## Safety Notes

- `CustomScanExprList::from_custom_scan` remains the boundary that validates the provider-owned `CustomScan` plan and its `custom_exprs` list.
- `expr(offset, label)` now checks the captured list length directly before reading the PostgreSQL List item.
- This avoids turning arbitrary raw `List` pointers into broadly reusable safe helper calls; the invariant stays tied to the DML CustomScan expression-list view.

## Unsafe Count

- Previous packet count: `2513`
- Current count: `2508`
- Delta: `-5`

The packet-local count log is `artifacts/src-unsafe-count.log`.

## Validation

- `artifacts/rustfmt-check.log`: `rustfmt --check src/am/ec_spire/custom_scan/dml.rs src/am/ec_spire/custom_scan/cost_helpers.rs` passed with only known stable-rustfmt config warnings.
- `artifacts/git-diff-check.log`: `git diff --check` passed.
- `artifacts/cargo-check-pg18-bench.log`: `cargo check --all-targets --no-default-features --features pg18,bench` passed with the known existing `src/am/mod.rs` unused SPIRE re-export warning.
- `artifacts/cargo-test-custom-scan-pg18-pg-test-no-run.log`: `cargo test --lib custom_scan --no-default-features --features pg18,pg_test --no-run` passed with the known existing Hadamard helper dead-code warnings.

