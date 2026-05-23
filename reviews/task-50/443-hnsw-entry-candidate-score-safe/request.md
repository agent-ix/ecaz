# Task 50/443: HNSW insert + vacuum entry-candidate + `score_graph_element` safe-fn lifts

## Why this slice

Cross-file leaf cascade. After slice 422 made
`graph::load_exact_graph_element` safe and slice 437 made
`metric.score_new_tuple_against_element` safe, the insert-side
entry-candidate loader's body became safe-op-only. Similarly, the
vacuum-side `score_graph_element` dispatcher had no internal
unsafe blocks (both arms route to safe helpers), and lifting it
makes the vacuum entry-candidate loader's body safe too.

## Scope

Three `unsafe fn` → safe `fn` lifts:

`src/am/ec_hnsw/insert.rs`:

- `load_insert_entry_candidate` — body composes safe ops only.

`src/am/ec_hnsw/vacuum.rs`:

- `VacuumSearchMetric::score_graph_element` — dispatches to
  `score_vacuum_code_element` (safe) or `score_graph_element_pair`
  (already safe fn). No internal unsafe blocks.
- `load_vacuum_entry_candidate` — the single internal
  `unsafe { ... }` block (which wrapped both
  `load_exact_graph_element` and the metric scoring call) is
  dropped entirely. Body is now safe.

Caller-side `unsafe { ... }` wraps stripped (five):

- `discover_insert_forward_neighbor_slots` call to
  `load_insert_entry_candidate`.
- `search_repair_candidates_for_layer` call to
  `load_vacuum_entry_candidate`.
- `search_repair_candidates_for_layer` existing-layer seed loop
  (combined `load_exact_graph_element` + `score_graph_element`
  block).
- `top_up_repair_replacements_from_linear_scan` candidate scoring
  block.

## Unsafe block counts

| File | Before | After | Δ |
| --- | ---: | ---: | ---: |
| `src/am/ec_hnsw/insert.rs` | 26 | 25 | -1 |
| `src/am/ec_hnsw/vacuum.rs` | 27 | 23 | -4 |
| **HNSW subsystem subtotal** | **339** | **334** | **-5** |

Cumulative rotation delta:

| Stage | HNSW total |
| --- | ---: |
| Pre-399 | 549 |
| After 442 | 339 |
| After 443 | 334 |

**Net rotation delta: -215 in HNSW (-39.2%).**

## Soundness rationale

All three lifted functions have zero internal `unsafe { ... }`
blocks after the cascade. The lifts are pure signature.

No anti-pattern B.

## Validation

Artifacts under `reviews/task-50/443-hnsw-entry-candidate-score-safe/artifacts/`:

- `per-file-after.log`
- `diff.patch`
- `cargo-check-pg18.log` — clean.

## Performance gate

aminsert entry-candidate path + amvacuumcleanup repair-search
path; signature-only change. Bench evidence gathered out-of-band
per `feedback_coder_push_smoke_checks`.

## Rotation milestone

**-215 (-39.2%)** on HNSW: 549 → 334. The -30% Exit Criteria
target now has a **9.2-point cushion**.
