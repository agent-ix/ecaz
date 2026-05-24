---
task: 50
packet: 240
topic: diskann-scan-descriptor-view
role: coder
status: ready-for-review
created: 2026-05-21T06:58:42-07:00
head_sha: 4aaf5ea0a56f690f14543d8a9a7063d4447743fa
---

# Review Request: DiskANN Scan Descriptor View

## Summary

This packet replaces DiskANN's standalone raw `IndexScanDesc` heap/snapshot resolvers with a typed scan descriptor view.

Changes:

- Added `DiskannScanDescView`, which borrows a live `IndexScanDescData` behind one constructor boundary.
- Moved heap relation resolution and snapshot resolution onto the view.
- Removed `resolve_scan_heap_relation` and `resolve_scan_snapshot`.
- Updated `amrescan` to derive heap relation, snapshot, and index relation reads from the view.

## Safety Notes

- `DiskannScanDescView::from_raw` remains `unsafe fn` because callers must supply a live PostgreSQL scan descriptor.
- Scan-owned `heapRelation`, `xs_snapshot`, and `indexRelation` reads now share one descriptor borrow instead of reopening the raw scan pointer in separate helpers.
- The call site remains inside the PostgreSQL AM callback guard; the redundant nested `unsafe` block was removed because the callback macro already runs the guarded body inside an unsafe context.

## Unsafe Count

- `src/am/ec_diskann/scan_state.rs`: `23 -> 18`
- `src/am/ec_diskann/routine.rs`: `58 -> 58`
- Previous repo count: `2489`
- Current repo count: `2484`
- Delta: `-5`

The packet-local count log is:

- `artifacts/unsafe-counts.log`

## Validation

- `artifacts/rustfmt-check.log`: `rustfmt --check src/am/ec_diskann/scan_state.rs src/am/ec_diskann/routine.rs` passed with only known stable-rustfmt config warnings.
- `artifacts/git-diff-check.log`: `git diff --check HEAD^ HEAD` passed.
- `artifacts/cargo-check-pg18-bench.log`: `cargo check --all-targets --no-default-features --features pg18,bench` passed with the known existing `src/am/mod.rs` unused SPIRE re-export warning.
- `artifacts/cargo-test-lib-ec-diskann-pg18-no-run.log`: `cargo test --lib ec_diskann --no-default-features --features pg18,pg_test --no-run` passed with the known existing Hadamard helper dead-code warnings.
