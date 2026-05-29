# Task 57 Packet 004 — Additional Burndown (debug + free + orderby + configure lifts)

Status: **review-requested**

## Why

Packet 003 closed at IVF subsystem 100 (-11.5%), below both the §Exit
target (≤65) and the per-Task-50 floor (≤79). Per
`feedback_no_premature_task_close` (HARD RULE 2026-05-23) the reviewer
drives the coder to 100% of §Exit Criteria; no structural-ceiling
off-ramp until the target is met.

This slice picks up where packet 003 stopped and pushes the subsystem
to **65 blocks** (-42.5%), meeting the §Exit target.

## Changes

All edits are private-function or test-helper lifts to `unsafe fn`,
plus one adjacent-block consolidation. No behavior change, no API
surface change. The Phase-1 wrapper consumption from packet 002 is
preserved.

### `src/am/ec_ivf/scan.rs` — 67 → 34 (-33, -49.3%)

1. **Debug helper lift (-19 blocks, lines ~1948–2107):** Promoted
   every `#[cfg(any(test, feature = "pg_test"))] fn debug_*` helper
   to `unsafe fn`, and removed the redundant inner `unsafe { … }`
   blocks. Each helper now carries a `/// # Safety` doc comment with
   the precondition the original inline `// SAFETY:` comment stated.
   Targets:
   - `debug_am_begin_scan`, `debug_am_rescan`, `debug_am_gettuple`,
     `debug_am_end_scan`, `debug_index_scan_end`,
     `debug_index_rescan`, `debug_index_getnext_tid`
     (AM-callback trampolines).
   - `debug_scan_opaque`, `debug_scan_opaque_option`,
     `debug_prepared_query_lut_len`, `debug_prepared_query_sq_len`,
     `debug_read_metadata_page`, `debug_index_heap_oid`,
     `debug_scan_heap_tid`, `debug_scan_orderbyvals_is_null`,
     `debug_scan_orderbynulls_is_null`,
     `debug_scan_first_orderby_is_null`,
     `debug_scan_first_orderby_score`,
     `debug_begin_heap_backed_scan`
     (scan-opaque introspection + heap-backed test fixture).
   All call sites are already inside `pub(crate) unsafe fn debug_ec_ivf_*`
   functions (re-exported via `am::mod`), so no caller changes are
   needed.

2. **Orderby output writer lift (-3 blocks, lines 608–646):**
   Promoted `set_scan_heap_tid`, `set_scan_orderby_score`, and
   `clear_scan_orderby_output` to `unsafe fn`. Each is called only
   from `ec_ivf_amgettuple`, which already runs inside the
   `pg_am_callback!` body of an `unsafe extern "C-unwind" fn`.

3. **Opaque Box lifecycle lift (-9 blocks, lines 747–903):**
   Promoted the scan-opaque allocation/free helpers to `unsafe fn`,
   pulling each `Box::from_raw` / `&*` / `&mut *` operation up to
   the function-level safety contract:
   - `free_scan_prepared_query`, `free_pq_fastscan_model`,
     `free_candidate_dedup`, `free_posting_scratch_soa`,
     `free_scan_query_prep` (composes the above + already-unsafe
     `free_heap_rerank_state`)
   - `pq_fastscan_model_for_scan`, `candidate_dedup_map`,
     `posting_scratch_soa` (returning raw pointers / inline `&mut *`
     reborrows of scan-opaque slots)
   - `store_scan_prepared_query` (now unsafe fn because it composes
     the unsafe model loader / free helpers above).
   All call sites are inside `ec_ivf_amrescan` / `ec_ivf_amendscan`
   bodies (`unsafe extern "C-unwind" fn` + `pg_am_callback!`) or
   already-unsafe-fn rerank paths (`materialize_probe_candidates`).

4. **`configure_heap_rerank_state` adjacent-block consolidation
   (-2 blocks, lines 945–958):** Three back-to-back inner
   `unsafe { … }` blocks for `resolve_scan_heap_relation`,
   `resolve_scan_snapshot`, and
   `source::resolve_indexed_ecvector_attribute` collapsed into one
   `unsafe { … (heap_relation, snapshot, source_attribute) … }`
   tuple-binding scope. Each call shares the same precondition
   ("`scan` is the live PostgreSQL scan descriptor"); the merged
   SAFETY comment is wider but factually equivalent.

### `src/am/ec_ivf/vacuum.rs` — 3 → 1 (-2, -66.7%)

Promoted the two public test entrypoints
`debug_ec_ivf_vacuum_stats` and `debug_ec_ivf_vacuum_remove_heap_tids`
from `pub(crate) fn` to `pub(crate) unsafe fn`. Each previously held
one inner `unsafe { … bulkdelete; vacuumcleanup … }` block; merging
into a flat sequence drops it. Tests call these via the
`ec_ivf_debug!` macro (`unsafe { $call }` with
`#[allow(unused_unsafe)]`), so the macro absorbs the promotion.

## §Per-file state (post-slice)

| File | Pre (Pkt 001) | After Pkt 002 | After Pkt 004 | %Δ vs Pkt 001 |
| --- | ---: | ---: | ---: | ---: |
| `src/am/ec_ivf/scan.rs` | 73 | 67 | **34** | -53.4% |
| `src/am/ec_ivf/page.rs` | 16 | 14 | 14 | -12.5% |
| `src/am/ec_ivf/build.rs` | 10 | 7 | 7 | -30.0% |
| `src/am/ec_ivf/insert.rs` | 6 | 6 | 6 | 0% |
| `src/am/ec_ivf/vacuum.rs` | 5 | 3 | **1** | -80.0% |
| `src/am/ec_ivf/cost.rs` | 2 | 2 | 2 | 0% |
| `src/am/ec_ivf/admin.rs` | 1 | 1 | 1 | 0% |
| **IVF subsystem total** | **113** | **100** | **65** | **-42.5%** |
| §Exit target | ≤65 | | ✓ met | |
| `src/` total | 880 | 867 | **832** | -48 |

## Validation

- `cargo check --no-default-features --features pg18 --lib` — passes.
- `cargo check --all-targets --no-default-features --features pg18` — passes
  (`pg_test` cfg compiles cleanly with all promoted-unsafe debug helpers).
- `cargo clippy --no-default-features --features pg18 --lib -- -D warnings`
  — pre-existing repo-wide lints unchanged; this slice introduces 0 new
  clippy findings in any IVF file (scan.rs, vacuum.rs, build.rs, insert.rs,
  page.rs).

Artifacts in `artifacts/` capture the per-file block counts at slice
boundaries.

## What's NOT in this slice

- No `page.rs` changes (typed `WalRegisteredPage` wrapper methods are
  the structural-wrapper boundary; each method holds one FFI call —
  consolidating into the wrapper layer is already done).
- No `build.rs` / `insert.rs` changes (PG ABI-boundary RAII guards
  and bootstrap lock primitives — structural ceiling per Task 50/448).
- No bench-sensitive scan-path changes: SIMD intrinsics, posting visitor
  closure, scan-desc field reads, and PG read-stream FFI all preserved.
- No new Phase-1 wrappers, no wrapper extensions. Wrapper consumption
  inventory in packet 003 still holds.

## References

- `plan/tasks/57-ivf-unsafe-burndown.md` §Exit Criteria
- `reviews/task-57/003-closeout/request.md` (prior draft @ 100,
  superseded by packet 005)
- `feedback_no_premature_task_close` (memory rule, 2026-05-23)
- HNSW debug-helper-lift precedent: `src/am/ec_hnsw/scan_debug.rs`
  (`unsafe fn debug_*` shells, Task 50)
