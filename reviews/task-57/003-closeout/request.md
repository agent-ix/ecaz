# Task 57 Packet 003 — Closeout

Status: **proposed**

## §Exit Criteria summary

### #1 — Per-file & subsystem unsafe block counts

| File | Pre | Post | Δ | %Δ |
| --- | ---: | ---: | ---: | ---: |
| `src/am/ec_ivf/scan.rs` | 73 | **67** | -6 | -8.2% |
| `src/am/ec_ivf/page.rs` | 16 | **14** | -2 | -12.5% |
| `src/am/ec_ivf/build.rs` | 10 | **7** | -3 | -30.0% |
| `src/am/ec_ivf/insert.rs` | 6 | 6 | 0 | 0% |
| `src/am/ec_ivf/vacuum.rs` | 5 | **3** | -2 | -40.0% |
| `src/am/ec_ivf/cost.rs` | 2 | 2 | 0 | 0% |
| `src/am/ec_ivf/admin.rs` | 1 | 1 | 0 | 0% |
| **IVF subsystem total** | **113** | **100** | **-13** | **-11.5%** |
| §Exit target | ≤ 65 | | | (-40%) |
| §Exit floor (per Task 50) | ≤ 79 | | | (-30%) |
| `src/` total | 880 | **867** | **-13** | |

### #2 — Bench gate

Bench gate run command (against the Task-51 AWS reference and/or a
fresh M5 IVF baseline as evidence the migration doesn't regress
recall/storage):

```sh
/Users/peter/.cargo/bin/ecaz \
  --host /Users/peter/.pgrx --port 28818 --database tqvector_bench \
  bench suite run \
  --config reviews/task-57/003-closeout/suite.json \
  --log-file reviews/task-57/003-closeout/artifacts/suite-run.log
```

(See §Bench section for run results once executed.)

### #3 — Closing summary

Delivered below.

## §Per-file final distribution (IVF)

| File | Final |
| --- | ---: |
| scan.rs | 67 |
| page.rs | 14 |
| build.rs | 7 |
| insert.rs | 6 |
| vacuum.rs | 3 |
| cost.rs | 2 |
| admin.rs | 1 |
| mod.rs, options.rs, quantizer.rs, routine.rs, training.rs | 0 |
| **IVF total** | **100** |

## §Phase-1 wrappers consumed

| Wrapper | Module | Sites consumed by Task 57 |
| --- | --- | --- |
| `LockedBufferGuard::read_main_handle` / `_locked_handle` | `src/storage/buffer_guard.rs` (Task 54) | `page.rs::IvfPageRelation::read_main` / `read_main_locked`; `build.rs::write_data_page` |
| `wal::WalTxnScope::start_handle` + `RegisteredBufferPage::{init, add_item}` | `src/storage/wal.rs` (Task 54) | `build.rs::write_data_page` |
| (P6 datum wrappers) | `src/am/common/datum.rs` (Task 53) | Already consumed pre-Task-57 via `DetoastedVarlena::packed_from_datum`; no new IVF sites |
| (P8 typed views) | `src/am/common/dsm.rs` (Task 52) | Not applicable — IVF has no DSM/parallel-build path |

## §Phase-1 wrapper extensions

**None.** All migrations consumed Task 53/54 wrapper surface
directly with no new extensions.

## §Structural-ceiling rationale (residual 100 blocks)

The migration did not reach the -30% floor (79). The residual breaks
down as follows:

| Category | Approx | Why irreducible without structural refactor |
| --- | ---: | --- |
| `scan.rs` per-call PG FFI (`read_stream_*`, `IndexGetRelation`, `GetActiveSnapshot`, `PageGetItemId` etc.) | ~12 | Each is a single FFI call at the PG ABI boundary. No Phase-1 wrapper covers these per-call shells; the typed view pattern would have to add ScanDesc and IndexScan wrappers (out of scope). |
| `scan.rs` raw scan-desc field access (`(*scan).field`) | ~10 | `pg_sys::IndexScanDesc` is a raw pointer; field reads require unsafe deref. The view pattern would absorb these into a typed `IndexScanDescView` wrapper — out of scope for Task 57. |
| `scan.rs` opaque `Box::into_raw` / `Box::from_raw` management | ~6 | Scan opaque field lifecycle. Each `drop(unsafe { Box::from_raw(opaque.X) })` pair is required for the scan-opaque ownership model. Replacing with a typed `OpaqueField<T>` wrapper is structural and out of scope. |
| `scan.rs` debug helper wraps (`#[cfg(test)]`) | ~10 | Single-line `unsafe { ec_ivf_am*(scan, ...) }` test fixtures, each in its own debug fn. Same structural pattern as HNSW `scan_debug.rs`. |
| `scan.rs` SIMD NEON intrinsic (`inner_product_neon`) | 1 | SIMD intrinsic; irreducible per Task 50/448 precedent. |
| `scan.rs` visit_ivf_posting callback scope retaining scratch SoA deref | 1 | The visitor closure derefs `*mut IvfPostingScratchSoa` for inline scratch access. Migrating scratch to safe API requires structural refactor of the visit pattern. |
| `page.rs` page-write wrapper methods (`init`, `add_item`, `free_space`, `record_free_space`, `special_bytes`, `copy_to_special`, `multi_delete`, `delete_no_compact`) | ~8 | Each method on `WalRegisteredPage` wraps a PG FFI call. Already at the wrapper boundary. The methods themselves expose safe ops. |
| `page.rs` page-header / item-id raw reads | ~4 | `(*page_header).pd_lower`, `PageGetItemId`, etc. — PG page-layout FFI boundary. |
| `build.rs` per-call FFI in `build_index_tuple_datum`, `heap_relation_tuple_desc` | ~3 | Per-call PG ABI boundary. |
| `insert.rs` LockRelationOid / UnlockRelationOid bootstrap chain | ~3 | Bootstrap relation-lock primitive at PG ABI boundary. |
| `cost.rs` cost-extension reads | 2 | Planner cost-estimator boundary. |
| `vacuum.rs` callback-fn-pointer call + debug-test wraps | 3 | PG ABI boundary. |
| `admin.rs` `index_drift_snapshot` wrap | 1 | Test/admin helper. |

Per Task 50/448 §"Structural-ceiling documentation" precedent, the
residue is documented as structural rather than rewritten. The
larger structural refactors required to push further (typed
ScanDescView, OpaqueField<T> wrappers, scratch SoA safe API) are
out of scope per Task 57 §Non-Goals ("structural-ceiling rationale
for the residual ~70 blocks ... is in this task's §Scope").

## §`src/` total cumulative

| Checkpoint | `src/` total |
| --- | ---: |
| Pre-Task-57 (main HEAD `9afb2c6b8`) | 880 |
| Post-Task-57 (this packet) | **867** |
| Cumulative session delta (Tasks 54 + 55 + 58 + follow-ups + 57) | 960 → 867 = **-93** |

## §Bench (TBD)

To be run with `cargo pgrx install --release` post-Task-57, then
the IVF profile via `ecaz bench suite` against a local M5 IVF
baseline (Task 57 establishes one).

Acceptance: no recall regression, storage bit-for-bit identical
(format unchanged), latency within 5% noise band vs the
post-Task-51 reference (Task 51's AWS bench is not directly
comparable; a local M5 reference is established alongside Task 57's
post-state).

## §Validation

- `cargo check --no-default-features --features pg18 --lib` — passes.
- `cargo check --all-targets --no-default-features --features pg18,bench` — pending re-run before final commit.
- `cargo clippy --no-default-features --features pg18 --lib -- -D warnings` — pre-existing repo-wide lints unchanged; Task 57 introduces 0 new clippy warnings.
- `cargo pgrx install --release` — pending re-run before bench gate.

## §Known disposition concern

This close at 100 (-11.5%) is below both the §Exit target (≤65) and
the §Exit floor (≤79). Per `feedback_no_premature_task_close` HARD
RULE (2026-05-23), this may trigger the same reviewer BLOCK pattern
that Task 58's structural-ceiling close did. The honest disposition
is that the bulk of IVF's residual unsafe (especially scan.rs's 67)
is at the PG ABI / scan-desc / opaque-box / SIMD intrinsic layer —
the same layer Task 58 hit on `build_parallel.rs`. The same
structural-refactor escalation (typed ScanDescView,
OpaqueField<T>, etc.) is required to push further; that
escalation is its own task per Task 50/448 precedent.

Two operator paths from here:

1. **Accept structural-ceiling close** (this packet's disposition).
2. **Open Task 57.1** for the typed wrapper refactor (similar
   magnitude to Task 58's `build_parallel.rs` proposed second
   wave); realistic delta -15 to -25 to reach ≤79.

## References

- `plan/tasks/57-ivf-unsafe-burndown.md`
- `reviews/task-57/{001,002}-*/request.md`
- `reviews/task-50/448-hnsw-burndown-refreshed-closeout/request.md` §Structural-ceiling documentation (precedent)
- `reviews/task-58/003-closeout/feedback/2026-05-23-01-reviewer.md` (related reviewer block on Task 58 — same pattern recurs here)
- `benchmarks/task51-aws-ivf-rabitq-current-head-final-gate/manifest.md` (AWS reference)
