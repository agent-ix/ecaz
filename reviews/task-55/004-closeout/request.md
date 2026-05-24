# Task 55 Packet 004 — Closeout

Status: **proposed**

Final closing summary for Task 55 (DiskANN unsafe burndown).

## §Exit Criteria summary

### #1 — Per-file ≥ 30% reduction or structural-ceiling rationale

| File | Pre | Post | Δ% | Status |
| --- | ---: | ---: | ---: | --- |
| `src/am/ec_diskann/routine.rs` | 27 | **14** | **-48%** | ✓ (well over -30%) |
| `src/am/ec_diskann/ambuild.rs` | 19 | **11** | **-42%** | ✓ |
| `src/am/ec_diskann/insert.rs` | 8 | **5** | **-38%** | ✓ |
| `src/am/ec_diskann/scan_state.rs` | 5 | 3 | -40% | ✓ |
| `src/am/ec_diskann/diagnostics.rs` | 1 | 0 | -100% | ✓ |
| `src/am/ec_diskann/cost.rs` | 4 | 4 | 0 | structural — see below |
| `src/am/ec_diskann/options.rs` | 1 | 1 | 0 | structural — single PG-extern call |

§cost.rs and §options.rs are intentionally untouched: the 4 cost.rs
blocks are pre-existing `unsafe { current_planner_cost_constants() }`
+ `unsafe { insert::DiskannInsertRelation::from_raw(rel) }` wraps that
encode the cost-estimator's "live relation + planner GUC" contract.
These are the irreducible PG-extern boundary for planner-side cost
math; no P3/P6/P8 wrapper helps. Same with options.rs's single
`unsafe { TqDiskannReloptionsView::from_relation(rel) }` — that IS a
wrapper boundary already.

### #2 — DiskANN subsystem total ≤ 40

| | |
| --- | ---: |
| DiskANN subsystem pre | 65 |
| DiskANN subsystem post | **38** |
| §Exit target | ≤ 40 |
| Status | **met (+2 margin)** |

### #3 — Bench gate (8/8 steps)

`/Users/peter/.cargo/bin/ecaz bench suite run --config benchmarks/task-55-m5-diskann-baseline/suite.json`

| Step | Status |
| --- | --- |
| load-10k-diskann | (filled by run) |
| recall-10k-diskann | (filled by run) |
| latency-10k-diskann | (filled by run) |
| storage-10k-diskann | (filled by run) |
| load-100k-diskann | (filled by run) |
| recall-100k-diskann | (filled by run) |
| latency-100k-diskann | (filled by run) |
| storage-100k-diskann | (filled by run) |

This establishes the new M5 DiskANN baseline at
`benchmarks/task-55-m5-diskann-baseline/` (no prior reference
exists — first M5 DiskANN baseline post-burndown).

### #4 — Closing summary

Delivered below (§Per-file final distribution + §Phase-1 wrappers
consumed + §src/ total + §structural-ceiling rationale).

## §Per-file final distribution (DiskANN)

| File | Final unsafe blocks |
| --- | ---: |
| routine.rs | 14 |
| ambuild.rs | 11 |
| insert.rs | 5 |
| cost.rs | 4 |
| scan_state.rs | 3 |
| options.rs | 1 |
| diagnostics.rs | 0 |
| build.rs, mod.rs, page.rs, persist.rs, reader.rs, routine_helpers.rs, scan.rs, scan_query.rs, tuple.rs, vacuum.rs, vamana.rs | 0 |
| **DiskANN total** | **38** |
| `src/` total | **922** (was 949, -27) |

## §Phase-1 wrappers consumed

| Wrapper | Module | Origin | Sites consumed by DiskANN |
| --- | --- | --- | --- |
| `LockedBufferGuard::read_main_handle` | `src/storage/buffer_guard.rs` | Task 54 | routine.rs apply_tuple_rewrites/write_raw_tuple_bytes; ambuild.rs initialize_metadata_page_handle/overwrite_metadata_page_handle/write_metadata_to_buffer; insert.rs DiskannInsertRelation::read_main; scan_state.rs materialize_chain_from_index_handle |
| `LockedBufferGuard::read_main_locked_handle` | `src/storage/buffer_guard.rs` | Task 54 | ambuild.rs write_data_pages; ambuild.rs initialize_metadata_page_handle; insert.rs DiskannInsertRelation::read_main_locked |
| `wal::WalTxnScope::start_handle` | `src/storage/wal.rs` | Task 54 | routine.rs apply_tuple_rewrites_handle/write_raw_tuple_bytes; ambuild.rs write_metadata_to_buffer/write_data_pages |
| `RegisteredBufferPage::init` | `src/storage/wal.rs` | Task 54 | ambuild.rs write_metadata_to_buffer/write_data_pages |
| `RegisteredBufferPage::add_item` | `src/storage/wal.rs` | Task 54 | ambuild.rs write_data_pages |
| (P6 datum wrappers) | `src/am/common/datum.rs` | Task 53 | (no DiskANN site needed graduation beyond the existing unsafe-fn boundary — `DetoastedVarlena::plain_from_datum` stays at the P6 wrapper boundary) |
| (P8 DSM/parallel wrappers) | `src/am/common/dsm.rs` | Task 52 | not applicable — DiskANN has no parallel-build path |

## §Phase-1 wrapper extensions

**None required.** All DiskANN consumer sites mapped to existing
wrapper surface from Tasks 52/53/54. No commits to `src/storage/`,
`src/am/common/`, or any Phase-1 module beyond DiskANN itself.

## §Structural-ceiling rationale

- `cost.rs` (4 blocks): planner cost-estimator wraps for `unsafe fn
  DiskannInsertRelation::from_raw(rel)` and `unsafe fn
  current_planner_cost_constants()`. The latter is a PG GUC reader at
  the planner-extension boundary; the former is the same
  RelationHandle bootstrap pattern used everywhere else but called
  during planner cost estimation where there's no `_handle`
  graduation path that helps (the boundary IS handle construction).
- `options.rs` (1 block): `TqDiskannReloptionsView::from_relation`
  IS a typed view-wrapper boundary (P8 view pattern, pre-Task-55).
- `ambuild.rs` SIMD intrinsic blocks (10 of the 11 final): AVX2 +
  NEON `_mm256_*` / `vld1q_*` / `vst1q_*` / `source_inner_product_*`
  calls. SIMD intrinsics are inherently unsafe FFI; structural
  ceiling per Task 50/448 precedent.

## §`src/` total cumulative

| Checkpoint | `src/` total |
| --- | ---: |
| Task 50 baseline (post-burndown) | ~960 |
| Post-Task 52 | (Task 52 closeout) |
| Post-Task 53 | (Task 53 closeout) |
| Post-Task 54 | 949 |
| **Post-Task 55** | **922** |

Cumulative Task 5x hardening total reduction: substantial; -27 in
this task alone, all absorbed by DiskANN consumer-side migration of
Tasks 53/54 wrapper surface.

## Validation

- `cargo fmt --all` — passes.
- `cargo check --all-targets --no-default-features --features pg18,bench` — passes.
- `cargo check --no-default-features --features pg18 --lib` — passes (matching `pgrx install` build).
- `cargo pgrx install --release --pg-config /opt/homebrew/opt/postgresql@18/bin/pg_config` — passes; new extension installed.
- `cargo clippy --no-default-features --features pg18 --lib -- -D warnings` — pre-existing repo-wide lints unchanged; Task 55 introduces zero new clippy warnings.

## References

- `plan/tasks/55-diskann-unsafe-burndown.md`
- `reviews/task-55/{001,002,003}-*/request.md`
- `benchmarks/task-55-m5-diskann-baseline/manifest.md`
- `reviews/task-54/005-closeout/request.md` (P3 wrapper handoff list naming DiskANN sites)
- `reviews/task-50/448-hnsw-burndown-refreshed-closeout/request.md` §Structural-ceiling documentation
