---
task: 50
packet: 233
topic: spire-index-scan-descriptor-view
role: coder
status: ready-for-review
created: 2026-05-21T05:11:33-07:00
head_sha: 295e4b0a96678b447376d2522724defd343ccc51
---

# Review Request: SPIRE IndexScan Descriptor View

## Summary

This packet adds a concrete `SpireIndexScanView` for SPIRE scan callbacks and heap-rerank candidate preparation.

The callback code now uses the view to access:

- `indexRelation`
- heap relation fallback
- snapshot fallback
- scan opaque state
- recheck/orderby output flags

The old standalone unsafe `resolve_scan_heap_relation` and `resolve_scan_snapshot` helpers were removed. The raw `IndexScanDesc` invariant now sits on the view constructor instead of being repeated at each caller.

## Safety Notes

- `SpireIndexScanView::from_raw` is still unsafe and is only called at SPIRE AM callback boundaries or the existing heap-rerank preparation boundary.
- The view rejects null scan descriptors before exposing safe field accessors.
- Opaque access is scoped through `opaque_mut`; `amendscan` uses `take_opaque_for_end_scan` to null the descriptor before dropping/freeing the opaque state.
- `ambeginscan` still owns initial scan descriptor allocation and opaque installation, so its direct setup unsafe remains outside this slice.

## Unsafe Count

- `src/am/ec_spire/scan/callbacks.rs`: `4 -> 4`
- `src/am/ec_spire/scan/relation.rs`: `24 -> 22`
- Previous repo count: `2506`
- Current repo count: `2504`
- Delta: `-2`

The packet-local count logs are:

- `artifacts/touched-file-unsafe-counts.log`
- `artifacts/src-unsafe-count.log`

## Validation

- `artifacts/rustfmt-check.log`: `rustfmt --check src/am/ec_spire/scan/callbacks.rs src/am/ec_spire/scan/relation.rs` passed with only known stable-rustfmt config warnings.
- `artifacts/git-diff-check.log`: `git diff --check` passed.
- `artifacts/cargo-check-pg18-bench.log`: `cargo check --all-targets --no-default-features --features pg18,bench` passed with the known existing `src/am/mod.rs` unused SPIRE re-export warning.
- `artifacts/cargo-test-ec-spire-pg18-pg-test-no-run.log`: `cargo test --lib ec_spire --no-default-features --features pg18,pg_test --no-run` passed with the known existing Hadamard helper dead-code warnings.

