# Task 50/410: HNSW scan.rs — `exact_score_cached_graph_element` + `score_and_cache_scan_element` → safe fn

## Why this slice

Two more siblings in the scan.rs scoring family. After slices 406-409
the dispatcher and grouped-context scorers are safe; the remaining
chain of TurboQuant / TurboQuantHotCold exact scoring still routes
through these two `unsafe fn`s. Both have well-bounded internal
unsafe scopes after the prior lifts:

- `score_and_cache_scan_element` body has one internal
  `unsafe { score_scan_element_result(...) }` block (FFI-shaped
  call into a quantizer score helper). All other operations on
  `opaque: &mut TqScanOpaque` are safe.
- `exact_score_cached_graph_element` body has one internal
  `unsafe { score_cached_graph_element_from_storage(...) }` block
  (other branches are pure safe operations on the borrow). All
  other operations on the borrow are safe.

Converting both to safe `fn(&mut TqScanOpaque, ...)`:

- removes 5 caller-side `unsafe { ... }` wrappers around
  `exact_score_cached_graph_element` across scan.rs,
- removes 1 caller-side `unsafe { ... }` wrapper around
  `score_and_cache_scan_element` in `exact_score_grouped_candidate_context`,
- one final caller site (`refine_grouped_frontier_head_exact`) is
  cleaned up further: its `let opaque_ptr = opaque as *mut TqScanOpaque`
  binding becomes superfluous for the `CandidateScoreDispatch::Exact`
  branch and is dropped (`opaque` is `&mut TqScanOpaque` already in
  scope).

## Scope

- `score_and_cache_scan_element` lifted to safe
  `fn(&mut TqScanOpaque, page::ItemPointer, f32, &[u8])`.
  Internal `unsafe { score_scan_element_result(...) }` block preserved
  with its existing SAFETY comment.
- `exact_score_cached_graph_element` lifted to safe
  `fn(pg_sys::Relation, &mut TqScanOpaque, page::ItemPointer,
  LoadedElementState)`. Two stale `let opaque_ref = scan_opaque_mut(opaque);`
  bindings removed (parameter is already a borrow). Internal
  `unsafe { score_cached_graph_element_from_storage(...) }` block
  preserved with its SAFETY comment.
- Six caller-side `unsafe { ... }` wrappers removed across scan.rs.

## Unsafe block counts

| File | Before | After | Δ |
| --- | ---: | ---: | ---: |
| `src/am/ec_hnsw/scan.rs` | 112 | 106 | -6 |
| **HNSW subsystem subtotal** | **501** | **495** | **-6** |

Cumulative rotation delta:

| Stage | HNSW total |
| --- | ---: |
| Pre-399 | 549 |
| After 409 | 501 |
| After 410 | 495 |

Net rotation delta: **-54 in HNSW** (-9.8%).

## Soundness rationale

Both lifted functions keep the same one internal `unsafe { ... }` block
each. The lift moves the `*mut TqScanOpaque` → `&mut TqScanOpaque`
shape outward; the obligation chain becomes:

```
unsafe fn produce_next_*(*mut TqScanOpaque)  -> takes obligation
   ├─ scan_opaque_mut(opaque) -> &mut TqScanOpaque
   └─ fn exact_score_cached_graph_element(rel, &mut T, ...)  -- safe
         └─ unsafe { score_cached_graph_element_from_storage(...) }
              -- single irreducible boundary block (still unsafe fn)
```

The borrow-check rationale is the same as slice 406-409: `&mut T`
reborrows for the inner call site, no overlapping borrows.

No anti-pattern B: the new signatures take references.

## Validation

Artifacts under `reviews/task-50/410-hnsw-scan-exact-score-and-cache-element-safe/artifacts/`:

- `manifest.md`
- `per-file-after.log`
- `diff.patch` (184 lines)
- `cargo-check-pg18.log` — clean.

## Performance gate

Scan hot path. No semantic change: same scoring branches selected on
same inputs, same caching of computed scores, same record_* timing
calls. Bench deferred per `feedback_coder_push_smoke_checks`.

## Out of scope

- `score_cached_graph_element_from_storage` and
  `score_scan_element_result` — both still `unsafe fn` because they
  call into `unsafe fn graph::load_exact_graph_element` (page/buffer
  FFI) and `quant::*` SIMD intrinsics respectively. The chain ends
  here at the irreducible PG / SIMD boundary.
- `cached_graph_neighbors`, `cached_graph_adjacency`, `prefetch_*`,
  and the rest of the `unsafe fn`s in scan.rs that route through
  similar bounded internal blocks: queued.
