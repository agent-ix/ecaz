# Task 50/433: HNSW source.rs — `resolve_indexed_*` + `resolve_*_attnum` chain safe-fn lifts

## Why this slice

Continues the source.rs catalog-lookup cascade. After slice 432 made
`resolve_source_*` safe, the indexed-vector lookups become liftable
too — each body composes only safe operations (the source-datum kind
helper, the lifted attribute resolver, and the safe
`relation_tuple_desc_copy_handle`).

## Scope

Seven `unsafe fn` → safe `fn` lifts in `src/am/ec_hnsw/source.rs`:

1. `resolve_indexed_vector_kind`
2. `resolve_single_base_heap_index_attnum`
3. `resolve_indexed_ecvector_attribute_from_index_info`
4. `resolve_indexed_ecvector_attribute`
5. `resolve_indexed_vector_attribute_from_index_info`
6. `resolve_indexed_vector_attribute`

Caller-side `unsafe { ... }` wraps stripped: 5 in source.rs internal
chain, 2 in build.rs (lines ~330 and ~688), 1 in scan.rs
(`configure_grouped_heap_rerank_state` indexed-attribute branch at
line ~1401), 1 in vacuum.rs (VacuumSearchMetric::for_relation line
~267).

Cross-AM cleanup: 1 wrap in `src/am/ec_ivf/scan.rs` (line ~916). The
ec_ivf module uses `ec_hnsw::source::resolve_indexed_ecvector_attribute`
through `crate::am::ec_hnsw::source`, so the lift cascaded to one
ec_ivf caller. The let-binding scope was also wrapped in a `{ ... }`
block to preserve the multi-let body shape after the `unsafe { }`
wrap removal.

## Unsafe block counts

| File | Before | After | Δ |
| --- | ---: | ---: | ---: |
| `src/am/ec_hnsw/source.rs` | 34 | 29 | -5 |
| `src/am/ec_hnsw/scan.rs` | 86 | 85 | -1 |
| `src/am/ec_hnsw/vacuum.rs` | 28 | 27 | -1 |
| `src/am/ec_hnsw/build.rs` | 20 | 18 | -2 |
| **HNSW subsystem subtotal** | **370** | **361** | **-9** |

(`src/am/ec_ivf/scan.rs` is not counted in HNSW; the lift caused a
beneficial wrap removal there but is non-HNSW.)

Cumulative rotation delta:

| Stage | HNSW total |
| --- | ---: |
| Pre-399 | 549 |
| After 432 | 370 |
| After 433 | 361 |

Net rotation delta: **-188 in HNSW** (**-34.2%**).

## Soundness rationale

Each lifted function had zero internal unsafe blocks after the
slice 432 chain. Lifts are signature-only. No anti-pattern B.

## Validation

Artifacts under `reviews/task-50/433-hnsw-source-indexed-resolvers-safe/artifacts/`:

- `manifest.md`
- `per-file-after.log`
- `diff.patch` (193 lines)
- `cargo-check-pg18.log` — clean.

## Performance gate

Aminsert / ambuild / amscan / amvacuumcleanup setup; not on the
inner loop. Bench deferred per `feedback_coder_push_smoke_checks`.

## Rotation milestone

**-188 (-34.2%)** on HNSW. Beyond the -30% Exit Criteria target with
a 4.2-point cushion.
