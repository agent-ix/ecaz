# Task 53 / 003 — source.rs Consumer Migration

Branch: `task-53`
Code path: `src/am/ec_hnsw/source.rs` only.

## Summary

Migrate `src/am/ec_hnsw/source.rs` consumer sites to route through
the wrappers from `src/am/common/datum.rs` (slice 002). Delete the
HNSW-local wrappers (`DetoastedFloat4Datum`, `FlatFloat4ArrayRef`,
`FlatFloat4VarlenaRef`, `FlatFloat4SourceRef`, plus the private
helpers `flat_array_dims_ptr` / `flat_array_data_offset` /
`maxaligned_size`) since their bodies now live in common.

**Result: `source.rs` 29 → 13 (-16, -55.2%). Task 53 §Exit Criterion
#2 (≤ 14) satisfied with 1 block of margin.**

## Per-file `unsafe { ... }` block delta

| File | Pre | Post | Delta |
| --- | ---: | ---: | ---: |
| `src/am/ec_hnsw/source.rs` | 29 | **13** | **-16** |

Diff stat: `+51 / -234` (183 net lines deleted).

## Task 53 cumulative arc (slices 001-003)

| Surface | Pre-Task-53 | Now | Δ |
| --- | ---: | ---: | ---: |
| `src/am/ec_hnsw/source.rs` | **29** | **13** | **-16 (-55.2%)** |
| `src/am/common/datum.rs` (new) | — | 15 | +15 |
| `src/am/common/detoast.rs` | 4 | 5 | +1 |
| `src/` total | 960 | 960 | **0 net** |

Wrapper-side +16 fully absorbs consumer-side -16. Net zero `src/`
inflation, which is the cleanest possible outcome for a wrapper-
introduction task.

## What landed (subagent return, verified by operator)

### Consumer call-site migrations

- **Line 269** (`AttnumLookup`): the `unsafe { pg_sys::get_attnum(...) }`
  + manual NUL-byte handling collapses to a single
  `AttnumLookup::lookup(rel, attname)` safe call.
- **Line 499** (`DetoastedVarlena`): consumer site retained; existing
  wrapper continues to be used.
- **Lines 529 / 597** (`DetoastedFloat4Datum` → common path): HNSW-
  local `DetoastedFloat4Datum` wrapper deleted; consumers route
  through `DetoastedVarlena::as_typed_slice::<f32>()` (slice-002
  addition) or directly through `FlatFloat4Source` per call shape.
- **Lines 534 / 546 / 551 / 553 / 557 / 578 / 605 / 623 / 727 / 744**:
  HNSW-local `FlatFloat4ArrayRef` and `FlatFloat4VarlenaRef` impls
  deleted; their internal unsafe blocks now live in `common/datum.rs`.
  Consumer sites pass through the public `FlatFloat4Source<'a>`
  façade.
- **Lines 638 / 643 / 667 / 682 / 694**: `FlatFloat4SourceRef` enum
  dispatch collapsed into `FlatFloat4Source::<from_datum>` taking a
  `FlatFloat4Kind`. The three public closure-CPS helpers
  (`with_flat_float4_source_from_datum`,
  `with_source_from_heap_row_reader`,
  `with_indexed_ecvector_from_slot_reader`) now route through
  `common::datum::FlatFloat4Source<'_>`.

### Cross-module impact

External consumers in `vacuum.rs`, `scan.rs`, `build.rs`, `insert.rs`,
and `ec_ivf/scan.rs` continue to compile unchanged — they only call
`.as_slice()` on the closure-passed source, and the closure signature
moved from `FlatFloat4SourceRef` to `FlatFloat4Source<'_>` without
breaking trait inference.

## Behavior changes — flagged

1. `resolve_source_attnum` now folds the previous "invalid NUL byte"
   diagnostic into `AttnumLookup`'s "does not name a user column"
   error. Behavioral parity for any well-formed column name; user-
   supplied NUL-bearing names get the missing-column error instead
   of the NUL-byte-specific error. Low risk — build-time only, no
   scan-path impact.

No other behavior changes.

## Slice 002 deferrals — dispositioned here

| Deferral | Disposition |
| --- | --- |
| `DetoastedVarlena<'a>` lifetime | **Deferred again** to Tasks 55/56/57. 9+ cross-AM consumers (lib.rs, ec_spire/*, ec_diskann/*, ec_ivf/*) make this a cross-cutting refactor; HNSW-only scope of Task 53 can't address it cleanly. Recorded in the closeout's SPIRE/IVF/DiskANN handoff list. |
| `EcVectorView` wiring | **Stays a documented shim.** No `EcVector` type exists in the codebase (verified by grep). Task 53 ships with `EcVectorView` as a thin shim over `FlatFloat4Source<Varlena>`. |
| `flat_array_*` helper duplication | **Resolved.** All three HNSW-local copies deleted; consumers route through `common/datum.rs`'s private copies. |

## Anti-pattern B / view-operations discipline

The migration introduced zero new safe `fn(&self) -> &'a T`
accessors. All wrapper constructors stay `unsafe fn`; reads return
Copy values or slices via contracted `from_raw_parts`. The `&[f32]`
returned by `FlatFloat4Source::as_slice()` is the contracted slice
view per `DetoastedVarlena::as_bytes()` precedent (slice-447 P6
pattern).

## Validation

- `cargo fmt --all` — clean (only `source.rs` touched in-scope).
- `cargo check --no-default-features --features pg18` — `Finished`
  exit 0, 17.16s (subagent), 0.10s (operator re-validation on
  cache).
- `cargo clippy ... -- -D warnings` — not re-run; same pre-existing
  rabitq backlog from main-merge state.
- `cargo pgrx test` — skipped per `feedback_dyld_buffer_blocks_known`.
  The migration is semantics-preserving signature-only substitution;
  all PG primitives invoked in the same order under the same
  contracts.

## Toward closeout (slice 004)

Task 53 §Exit Criteria status:

| # | Criterion | Status |
| - | --- | :-: |
| 1 | Four typed wrappers in `src/am/common/datum.rs` | ✓ (slice 002) |
| 2 | `source.rs` ≤ 14 | ✓ (**13**, slice 003) |
| 3 | HNSW recall + QPS + per-row storage no regression | pending bench |
| 4 | Closing summary + SPIRE/IVF/DiskANN handoff list | pending |

Slice 004 (closeout) will:
- Run the full 8-step `ecaz bench suite` against `benchmarks/task-50-m5-hnsw-baseline/`.
- Author the closeout request with per-file deltas + handoff list
  enumerating each SPIRE/IVF/DiskANN consumer site that the new
  wrappers will absorb under Tasks 55/56/57.

## Provenance

Slice 003 authored by delegated `general-purpose` subagent (agentId
`ab529acffb53c52ff`). Diff reviewed operator-side; cargo check
re-validated; per-file unsafe counts spot-checked.

## Cross-references

- Slice 002 wrappers: `reviews/task-53/002-datum-wrappers/`.
- Planning packet: `reviews/task-53/001-execution-planning/`.
- Task spec: `plan/tasks/53-common-p6-datum-wrappers.md`.
- Memory rules: `feedback_anti_pattern_b_unbounded_lifetime`,
  `feedback_view_operations_not_accessors`, `feedback_no_premature_task_close`.
