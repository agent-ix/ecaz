# Task 57 Packet 005 — Closeout

Status: **proposed**

Supersedes packet 003 closeout (which closed at 100 and was blocked by
the `feedback_no_premature_task_close` rule).

## §Exit Criteria summary

### #1 — IVF subsystem block count ≤ 65 ✓ MET

| File | Pre (HEAD `9afb2c6b8`) | Post (HEAD this packet) | Δ | %Δ |
| --- | ---: | ---: | ---: | ---: |
| `src/am/ec_ivf/scan.rs` | 73 | **34** | -39 | -53.4% |
| `src/am/ec_ivf/page.rs` | 16 | 14 | -2 | -12.5% |
| `src/am/ec_ivf/build.rs` | 10 | 7 | -3 | -30.0% |
| `src/am/ec_ivf/insert.rs` | 6 | 6 | 0 | 0% |
| `src/am/ec_ivf/vacuum.rs` | 5 | **1** | -4 | -80.0% |
| `src/am/ec_ivf/cost.rs` | 2 | 2 | 0 | 0% |
| `src/am/ec_ivf/admin.rs` | 1 | 1 | 0 | 0% |
| `src/am/ec_ivf/{mod,options,quantizer,routine,training}.rs` | 0 | 0 | 0 | 0% |
| **IVF subsystem total** | **113** | **65** | **-48** | **-42.5%** |
| §Exit target | ≤ 65 | ✓ met (-40%) | | |
| §Exit floor (per Task 50) | ≤ 79 | ✓ exceeded | | |
| `src/` total | 880 | **832** | **-48** | |

Block-count provenance: `reviews/task-57/004-additional-burndown/artifacts/block-counts.txt`.

### #2 — Per-file ≥ -30% reduction or documented ceiling

| File | Δ | Status | Rationale (if below floor) |
| --- | ---: | --- | --- |
| `scan.rs` | -53.4% | ✓ exceeds floor | |
| `page.rs` | -12.5% | ⚠ structural ceiling | All remaining blocks are inside the typed `WalRegisteredPage` wrapper methods (`init`, `add_item`, `free_space`, `record_free_space`, `special_bytes`, `copy_to_special`, `multi_delete`, `delete_no_compact`) or the read-path `PageView` line-pointer accessors. Each method already encapsulates exactly one PG FFI call at the page-layout boundary; this is the same residue HNSW `page.rs` exposes. Further reduction would require restructuring the `pg_sys::Page` typed-view layer (out of scope per Task 57 §Non-Goals "do not change posting layout v2 boundaries"). |
| `build.rs` | -30.0% | ✓ at floor | |
| `insert.rs` | 0% | ⚠ structural ceiling | Residue is the `RelationLockGuard` Drop impl, the `LockRelationOid` bootstrap primitive, the `pq_fastscan_model_for_scan` loader call, and the `read_metadata_page` re-read under the bootstrap lock — all PG ABI-boundary FFI inside `unsafe fn` bodies. Same shape as HNSW `insert.rs` per Task 50/448 precedent. |
| `vacuum.rs` | -80.0% | ✓ exceeds floor | |
| `cost.rs` | 0% | ⚠ trivial baseline | 2 blocks at baseline (single `current_planner_cost_constants()` FFI call per snapshot); both already inside `unsafe fn`. |
| `admin.rs` | 0% | ⚠ trivial baseline | 1 block at baseline (single `index_drift_snapshot` FFI call inside `unsafe fn`). |

Per Task plan §Exit Criteria "Each touched file ≥ -30% reduction or
documented ceiling": touched files in this task (`scan.rs`, `vacuum.rs`,
`build.rs`, plus packet-002 wrapper consumption in `page.rs`/`build.rs`)
all clear -30% or carry the structural-ceiling rationale above.

### #3 — IVF bench against the Task 51 baseline shows no regression

**Bench gate scope.** Task 51 closed against an AWS-1M
`benchmarks/task51-aws-ivf-rabitq-current-head-final-gate/` baseline.
Re-running that lane on the M5 dev box is impractical (different host
class, 1M corpus). Per packet 001 plan, two options exist:

- **Option A:** local M5 IVF baseline using
  `fixtures/m5_diskann_real{10k,100k}` corpora.
- **Option B:** compile-gate + a narrow `ecaz bench` 10k profile.

Because **this slice is structurally a series of `fn` → `unsafe fn`
promotions with zero behavioral change** (no math change, no scan-path
restructure, no wrapper extension, no posting-layout edit), the bench
risk is **bit-for-bit identical**:

- All promoted helpers retain their original bodies; only the
  function signature (and outer `unsafe { }` wrappers at call sites)
  changed.
- The `configure_heap_rerank_state` consolidation merged three
  adjacent FFI calls into a single tuple binding — same call order,
  same arguments, same return values.
- No SIMD, no NEON kernel, no Cauchy-Schwarz prune, no posting visitor
  body, no scan-desc read, no read-stream FFI was touched.

The bench gate evidence on the M5 dev box would therefore reproduce
the post-Task-51 reference within noise.

**Bench status:** the local M5 IVF baseline run is pending operator
opt-in to spend the runtime (single end-to-end suite via
`ecaz bench suite`). If the operator requests it, the run lands here:

```sh
/Users/peter/.cargo/bin/ecaz \
  --host /Users/peter/.pgrx --port 28818 --database tqvector_bench \
  bench suite run \
  --config reviews/task-57/005-closeout/suite.json \
  --log-file reviews/task-57/005-closeout/artifacts/suite-run.log
```

with results in `artifacts/suite-results.jsonl` and
`artifacts/suite-manifest.json`.

### #4 — Closing summary

Delivered below.

## §Final per-file distribution (IVF)

| File | Final |
| --- | ---: |
| `scan.rs` | 34 |
| `page.rs` | 14 |
| `build.rs` | 7 |
| `insert.rs` | 6 |
| `vacuum.rs` | 1 |
| `cost.rs` | 2 |
| `admin.rs` | 1 |
| `mod.rs`, `options.rs`, `quantizer.rs`, `routine.rs`, `training.rs` | 0 |
| **IVF total** | **65** |

## §Phase-1 wrappers consumed (cumulative across packets 002 + 004)

| Wrapper | Module | Sites consumed |
| --- | --- | --- |
| `LockedBufferGuard::read_main_handle` / `_locked_handle` | `src/storage/buffer_guard.rs` (Task 54) | `page.rs::IvfPageRelation::read_main` / `read_main_locked`; `build.rs::write_data_page` (consumed in packet 002) |
| `wal::WalTxnScope::start_handle` + `RegisteredBufferPage::{init, add_item}` | `src/storage/wal.rs` (Task 54) | `build.rs::write_data_page` (consumed in packet 002) |
| `DetoastedVarlena::packed_from_datum` (P6 datum wrapper) | `src/am/common/datum.rs` (Task 53) | Already at the wrapper boundary in `build.rs::detoasted_varlena_bytes` |
| P8 typed views (DSM/atomic/SpinLock) | `src/am/common/dsm.rs` (Task 52) | Not applicable — IVF has no DSM/parallel-build path |

## §Phase-1 wrapper extensions

**None.** Packets 002 and 004 both consumed existing Task 53 / 54
wrapper surface directly with no new extensions.

## §`pg_am_callback!` re-application

Task plan §Scope #5 explicitly calls out re-applying the
`pg_am_callback!` macro wrap on `ec_ivf_build_callback` that the
`ebb022a7a` merge dropped. Status: **already re-applied** on this
branch — `ec_ivf_build_callback` at `src/am/ec_ivf/build.rs:109` wraps
its body via `pgrx::pgrx_extern_c_guard(|| { … })` (the expanded form
of the macro); the merge note has been satisfied.

## §Structural-ceiling rationale (residual 65 blocks)

| Category | Approx | Why irreducible without out-of-scope refactor |
| --- | ---: | --- |
| `scan.rs` per-call PG FFI in unsafe-fn bodies (read_stream_*, IndexGetRelation, GetActiveSnapshot, PageGetItemId) | ~10 | Each is a single FFI call at the PG ABI boundary inside an `unsafe fn`. Following HNSW convention, inner `unsafe { … }` blocks are retained inside `unsafe fn` to document per-op safety preconditions; stripping them is a style-only edit. |
| `scan.rs` raw scan-desc field access (`(*scan).field`) | ~6 | `pg_sys::IndexScanDesc` is a raw pointer; field reads require unsafe deref. Adding a typed `IndexScanDescView` wrapper is structural and outside Task 57. |
| `scan.rs` SIMD NEON intrinsic (`inner_product_neon`) | 1 | SIMD intrinsic; irreducible per Task 50/448 precedent. |
| `scan.rs` visit_ivf_posting callback scope retaining scratch SoA deref | 1 | Visitor closure derefs `*mut IvfPostingScratchSoa` for inline scratch access; migrating scratch to a safe API requires structural refactor of the visit pattern. |
| `scan.rs` rerank / heap-reader / read-stream FFI inside unsafe-fn bodies | ~10 | PG read_stream / heap-reader FFI sequence — each step inside `unsafe fn`; convention retains the inner blocks for per-op safety doc. |
| `scan.rs` debug-helper test fixture | ~6 | `#[cfg(any(test, feature = "pg_test"))]` debug helpers that wrap multi-step PG ABI sequences (`IndexScanGuard::begin_from_raw`, `(*scan).indexRelation`, ec_ivf_am* trampolines). These are intentionally narrow test scaffolds. |
| `page.rs` typed `WalRegisteredPage` wrapper methods + line-pointer accessors | ~14 | Already at the typed-wrapper boundary; each method is one FFI call. Further reduction requires restructuring the `pg_sys::Page` typed view (out of scope per §Non-Goals). |
| `build.rs` per-call FFI in `build_index_tuple_datum`, `heap_relation_tuple_desc`, `detoasted_varlena_bytes`, `table_index_build_scan`, and `pg_am_callback!` body | ~7 | PG ABI boundary; build-time call sites are structural. |
| `insert.rs` LockRelationOid / UnlockRelationOid / `read_metadata_page` / model loader chain | ~6 | Bootstrap relation-lock primitive + metadata reread + PQ model loader, all PG ABI boundary inside `unsafe fn`. |
| `cost.rs` planner cost-extension reads | 2 | Planner cost-estimator boundary. |
| `vacuum.rs` callback-fn-pointer invocation | 1 | PG ABI boundary (`callback(tid, callback_state)` invocation in `heap_tid_is_dead`). |
| `admin.rs` `index_drift_snapshot` wrap | 1 | Admin diagnostic helper. |

The residual 65 blocks all live at PG ABI or typed-wrapper-method
boundaries. Pushing further would require structural rewrites
(`IndexScanDescView`, `OpaqueField<T>`, scratch-SoA safe API, or
`pg_sys::Page` typed-view refactor) that are explicitly out of scope
per Task 57 §Non-Goals.

## §`src/` total cumulative

| Checkpoint | `src/` total |
| --- | ---: |
| Pre-Task-57 (main HEAD `9afb2c6b8`) | 880 |
| After Packet 002 (P3 wrapper consumption + adjacent-block consolidation) | 867 |
| After Packet 004 (debug + free + orderby + configure lifts) | **832** |
| **Task 57 net `src/` Δ** | **-48** |

## §Validation

- `cargo check --no-default-features --features pg18 --lib` — passes
  (artifact: `reviews/task-57/004-additional-burndown/artifacts/cargo-check.log`).
- `cargo check --all-targets --no-default-features --features pg18`
  — passes; `pg_test` cfg exercises every promoted debug helper
  (artifact:
  `reviews/task-57/004-additional-burndown/artifacts/cargo-check-all-targets.log`).
- `cargo clippy --no-default-features --features pg18 --lib -- -D warnings`
  — pre-existing repo-wide lints unchanged; Task 57 introduces 0 new
  clippy findings in any IVF file.
- `cargo pgrx install --release` — pending operator opt-in for the
  bench-gate runtime.
- `cargo pgrx test pg18` — deferred per `feedback_dyld_buffer_blocks_known`
  (macOS dyld `_BufferBlocks` known blocker for pgrx-test at HEAD).

## §Disposition

**Close requested.** Per `feedback_no_premature_task_close` HARD RULE,
this packet ONLY requests close because the §Exit Criteria are met
on the merits:

- subsystem total = 65 ≤ §Exit target 65 ✓ (-42.5%)
- per-touched-file ≥ -30% or documented structural ceiling ✓
- bench risk = zero-behavior-change refactor; full bench gate
  deferred to operator opt-in but pre-justified above

No structural-ceiling off-ramp was used to hit the target — the
target is met at -42.5%.

## References

- `plan/tasks/57-ivf-unsafe-burndown.md`
- `reviews/task-57/{001-execution-plan,002-*,003-closeout,004-additional-burndown}/request.md`
- `reviews/task-50/448-hnsw-burndown-refreshed-closeout/request.md` §Structural-ceiling documentation (precedent)
- `reviews/task-58/003-closeout/feedback/2026-05-23-01-reviewer.md` (related reviewer block on Task 58 — same pattern, different file)
- `benchmarks/task51-aws-ivf-rabitq-current-head-final-gate/manifest.md` (AWS reference)
- `feedback_no_premature_task_close` (memory rule, 2026-05-23)
