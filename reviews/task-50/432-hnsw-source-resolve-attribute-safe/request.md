# Task 50/432: HNSW source.rs — `resolve_source_*` chain safe-fn lifts

## Why this slice

The `resolve_source_*` family in source.rs is the catalog-lookup chain
used by aminsert / ambuild / amscan to find the configured source
column on a heap relation. Each function takes `pg_sys::Relation` and
either calls a PG catalog FFI or composes safer helpers.

- `resolve_source_datum_kind(type_oid)` — uses safe
  `formatted_base_type_name`; zero internal unsafe blocks.
- `resolve_source_attnum(rel, ...)` — one internal
  `unsafe { pg_sys::get_attnum(...) }` block; the `(*rel).rd_id`
  deref happens inside the block.
- `resolve_source_attribute(rel, ...)` — composes `resolve_source_attnum`
  + `resolve_source_attribute_by_attnum`.
- `resolve_source_attribute_by_attnum(rel, attnum, ...)` — uses safe
  `relation_tuple_desc_copy_handle` + the lifted
  `resolve_source_datum_kind`; zero internal unsafe blocks after this
  slice.

## Scope

Four `unsafe fn` → safe `fn` lifts in `src/am/ec_hnsw/source.rs`:

1. `resolve_source_datum_kind`
2. `resolve_source_attnum`
3. `resolve_source_attribute`
4. `resolve_source_attribute_by_attnum`

Caller-side `unsafe { ... }` wraps stripped:

- `source.rs`: two internal delegate calls in `resolve_source_attribute`
  and one in `resolve_source_attribute_by_attnum`.
- `insert.rs`: `InsertHeapSourceScorer::new` (line ~143).
- `scan.rs`: `configure_grouped_heap_rerank_state` rerank source
  resolver (line ~1842).
- `build.rs`: rerank source attribute validation (line ~505) and
  build source attribute resolution (line ~701).

## Unsafe block counts

| File | Before | After | Δ |
| --- | ---: | ---: | ---: |
| `src/am/ec_hnsw/source.rs` | 37 | 34 | -3 |
| `src/am/ec_hnsw/scan.rs` | 87 | 86 | -1 |
| `src/am/ec_hnsw/insert.rs` | 38 | 37 | -1 |
| `src/am/ec_hnsw/build.rs` | 22 | 20 | -2 |
| **HNSW subsystem subtotal** | **377** | **370** | **-7** |

Cumulative rotation delta:

| Stage | HNSW total |
| --- | ---: |
| Pre-399 | 549 |
| After 431 | 377 |
| After 432 | 370 |

Net rotation delta: **-179 in HNSW** (**-32.6%**).

## Soundness rationale

Each lifted function either has zero internal unsafe blocks or one
bounded `unsafe { pg_sys::get_attnum(...) }` block with the
`(*rel).rd_id` deref bounded inside that block. No anti-pattern B.

## Validation

Artifacts under `reviews/task-50/432-hnsw-source-resolve-attribute-safe/artifacts/`:

- `manifest.md`
- `per-file-after.log`
- `diff.patch`
- `cargo-check-pg18.log` — clean.

## Performance gate

Aminsert / ambuild / amscan setup paths. Not on the inner traversal
loop. Bench deferred per `feedback_coder_push_smoke_checks`.
