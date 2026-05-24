# Task 50/425: HNSW shared.rs snapshot + count helpers safe-fn cascade

## Why this slice

Slice 424 made `with_page_line_tuple_bytes` and `with_writable_page_tuple_bytes`
safe. That removed the last internal `unsafe { ... }` blocks from the
shared.rs higher-level helpers, which were `unsafe fn` only for legacy
reasons:

- `count_live_elements_on_buffer` — used the safe page-tuple visitor only.
- `count_element_tuples` — used `read_metadata_page` (already safe),
  `graph::GraphStorageDescriptor::from_index_relation` (safe from
  slice 419), and `count_live_elements_on_buffer` (now safe).
- `highest_level_live_entry_candidate` — used the safe page-tuple
  visitor.
- `ec_hnsw_noop_vacuum_stats` — used `count_element_tuples`.
- `index_admin_snapshot` / `index_explain_snapshot` /
  `index_cost_snapshot` / `planner_integration_snapshot` — used
  `read_metadata_page` and the now-safe count/snapshot helpers.

Lifting all seven to safe `fn` strips one or two caller-side wraps
from `vacuum.rs`, `scan.rs`, and one inside `shared.rs` itself. The
`src/lib.rs` `with_live_index_relation!` macro gets a
`#[allow(unused_unsafe)]` since callers pass a mix of safe and
unsafe-fn paths through it.

## Scope

Seven `unsafe fn` → safe `fn` flips in `src/am/ec_hnsw/shared.rs`:

1. `count_live_elements_on_buffer`
2. `count_element_tuples`
3. `highest_level_live_entry_candidate`
4. `ec_hnsw_noop_vacuum_stats`
5. `index_admin_snapshot`
6. `index_explain_snapshot`
7. `index_cost_snapshot`
8. `planner_integration_snapshot`

Caller-side `unsafe { ... }` wraps stripped:

- `shared.rs`: 4 (count_live_elements_on_buffer in count_element_tuples,
  count_element_tuples in index_admin_snapshot, index_admin_snapshot
  in index_explain_snapshot, three snapshot helpers in
  planner_integration_snapshot)
- `vacuum.rs`: 2 (`ec_hnsw_noop_vacuum_stats`, `highest_level_live_entry_candidate`)
- `scan.rs`: 1 (`highest_level_live_entry_candidate` fallback in
  `initialize_scan_entry_candidate`)
- `src/lib.rs::with_live_index_relation!` macro: `#[allow(unused_unsafe)]`
  added so the macro stays shape-agnostic for callers that pass safe
  fn paths (`index_admin_snapshot`, `index_cost_snapshot`,
  `planner_integration_snapshot`).

## Unsafe block counts

| File | Before | After | Δ |
| --- | ---: | ---: | ---: |
| `src/am/ec_hnsw/shared.rs` | 27 | 21 | -6 |
| `src/am/ec_hnsw/scan.rs` | 88 | 87 | -1 |
| `src/am/ec_hnsw/vacuum.rs` | 41 | 39 | -2 |
| `src/lib.rs` | (n/a — non-HNSW; macro `#[allow]` only) | | |
| **HNSW subsystem subtotal** | **407** | **398** | **-9** |

Cumulative rotation delta:

| Stage | HNSW total |
| --- | ---: |
| Pre-399 | 549 |
| After 424 | 407 |
| After 425 | 398 |

Net rotation delta: **-151 in HNSW** (-27.5%).

## Soundness rationale

Each lifted function had zero internal `unsafe { ... }` blocks after
slice 424 — the lift is signature-only. The library-wide
`with_live_index_relation!` macro retains the `unsafe { ... }` body
to support callers that pass `unsafe fn` paths (e.g. SPIRE-side
snapshots that still are `unsafe fn`); the `#[allow(unused_unsafe)]`
attribute suppresses the now-noise warning for the safe-fn invocation
arms.

No anti-pattern B: every lifted function returns owned data, not
`&'a T`.

## Validation

Artifacts under `reviews/task-50/425-hnsw-shared-snapshots-safe/artifacts/`:

- `manifest.md`
- `per-file-after.log`
- `diff.patch` (179 lines)
- `cargo-check-pg18.log` — clean, **0 unused_unsafe warnings**.

## Performance gate

Snapshot helpers are diagnostic / cost-callback cold paths.
`count_element_tuples` runs once per vacuum-stats summary. None of
these are on the scoring/traversal hot path. Bench deferred per
`feedback_coder_push_smoke_checks`.

## Rotation milestone

**Past the -27% threshold** on HNSW: 549 → 398, net -151 (-27.5%).
The Task 50 §Exit Criteria target is -30% per processed module;
remaining headroom is ~15 unsafe blocks to cross that mark.

## Out of scope

- `shared.rs::initialize_metadata_page`, `update_metadata_page`,
  `with_locked_metadata_page`, `read_data_page`, `page_item_id`,
  `decode_heap_tid`, `write_metadata_bytes`: each retains one or
  more internal unsafe blocks tied to PG page-init / pfree / FFI
  buffer operations. The page-mutation surface needs its own slice
  family to lift.
- `src/lib.rs`: only the `#[allow(unused_unsafe)]` attribute change
  here; further reductions across the file are out of HNSW scope.
