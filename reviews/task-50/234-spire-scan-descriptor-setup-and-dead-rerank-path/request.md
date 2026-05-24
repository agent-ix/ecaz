---
task: 50
packet: 234
topic: spire-scan-descriptor-setup-and-dead-rerank-path
role: coder
status: ready-for-review
created: 2026-05-21T05:16:07-07:00
head_sha: d62863752a3de4f1397299fb8fba7c0f5fb3f92f
---

# Review Request: SPIRE Scan Descriptor Setup And Dead Rerank Path

## Summary

This packet extends the SPIRE scan descriptor view from packet 233 and removes a stale heap-rerank preparation path.

Changes:

- `ambeginscan` now uses `SpireIndexScanView::install_opaque` for descriptor setup instead of writing `parallel_scan` and `opaque` directly in the callback.
- Removed the unreferenced legacy `prepare_single_level_relation_snapshot_scan_candidates` path.
- Removed private helpers that only existed for that dead path:
  - `prefetch_heap_rerank_candidate_blocks`
  - `allocate_heap_slot`
  - `exact_heap_source_inner_product`
  - `exact_source_inner_product`

## Safety Notes

- The descriptor setup invariant remains owned by `SpireIndexScanView`; this keeps scan descriptor reads/writes behind one local contract.
- The deleted heap-rerank preparation path was not referenced by production code or tests. Current SPIRE production scan uses `remote_search_production_scan_heap_resolution_am_result_stream`.
- `heap_rerank_prefetch_block_numbers` remains because scan tests still cover it directly.

## Unsafe Count

- `src/am/ec_spire/scan/callbacks.rs`: `4 -> 4`
- `src/am/ec_spire/scan/relation.rs`: `22 -> 14`
- Previous repo count: `2504`
- Current repo count: `2496`
- Delta: `-8`

The packet-local count logs are:

- `artifacts/touched-file-unsafe-counts.log`
- `artifacts/src-unsafe-count.log`

## Validation

- `artifacts/rustfmt-check.log`: `rustfmt --check src/am/ec_spire/scan/callbacks.rs src/am/ec_spire/scan/relation.rs` passed with only known stable-rustfmt config warnings.
- `artifacts/git-diff-check.log`: `git diff --check` passed.
- `artifacts/cargo-check-pg18-bench.log`: `cargo check --all-targets --no-default-features --features pg18,bench` passed with the known existing `src/am/mod.rs` unused SPIRE re-export warning.
- `artifacts/cargo-test-ec-spire-pg18-pg-test-no-run.log`: `cargo test --lib ec_spire --no-default-features --features pg18,pg_test --no-run` passed with the known existing Hadamard helper dead-code warnings.

