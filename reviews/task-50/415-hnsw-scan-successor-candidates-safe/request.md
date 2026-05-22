# Task 50/415: HNSW scan.rs — `cached_scan_successor_candidates_for_layer` safe-fn lift

## Why this slice

The harder of the two lifts queued from packets 413/414. The function
was `unsafe fn(*mut TqScanOpaque, ...)` despite having zero internal
`unsafe { ... }` blocks of its own, because its body needed both
long-lived `quantizer: &ProdQuantizer` and `binary_query:
&BinarySignNoQjl4BitQuery` borrows that overlapped many
`scan_opaque_mut(opaque)` reborrows in the binary-prefilter scoring
loop. The current `scan_opaque_ref` / `scan_opaque_mut` anti-pattern B
helpers bypassed the borrow checker; a `&mut TqScanOpaque` parameter
exposes the real overlap.

The fix: tighten the quantizer and binary_query borrows to the
innermost block that actually uses them. Each scoring iteration
re-acquires them via `cached_quantizer_ref(opaque)` and
`binary_sign_query(opaque)` — both are pointer derefs, essentially
free, and the borrow ends at the closing brace of the inner block so
the next mutable call to `record_binary_prefilter_score_elapsed(opaque, ...)`
sees no conflict.

The same restructure applies to the post-loop scoring branches
(`turboquant_binary_live_rerank_enabled`,
`grouped_binary_traversal_score_enabled`,
`grouped_exact_traversal_full_candidate_scoring_for_layer`): each
predicate is computed into a `bool` before the subsequent mutable
call, releasing the immutable borrow.

## Scope

- `cached_scan_successor_candidates_for_layer<KeepFn>` lifted from
  `unsafe fn(_, *mut TqScanOpaque, _, _, KeepFn)` to safe
  `fn(_, &mut TqScanOpaque, _, _, KeepFn)`.
- Body restructured:
  - Hoist `binary_query_present: bool = binary_sign_query(opaque).is_some()`
    early so the test borrow is released before any `&mut` calls.
  - In the binary-prefilter loop, scope quantizer + binary_query
    borrows to the inner `let approx_score = { ... }` block.
  - In the post-loop scoring branches, compute each predicate into
    a `bool` (`live_rerank`, `binary_traversal_enabled`,
    `exact_full`) up front before the `if … else …` selector that
    may mutate.
  - Drop both stale `let opaque_ref = scan_opaque_ref(opaque);`
    bindings; the parameter is the borrow.
- Both caller closures (in `cached_upper_layer_seed_candidate` and
  the layer-0 seed driver in `initialize_scan_entry_candidate`)
  drop their `unsafe { cached_scan_successor_candidates_for_layer(...) }`
  wrappers and pass `opaque` directly.
- The layer-0 inner `keep_neighbor_tid` closure (which needs an
  immutable `&TqScanOpaque` while the outer FnMut closure holds a
  `&mut` borrow) retains a tight `unsafe { &*opaque_ptr }` block
  inside the inner closure, bounded by the FnMut call frame; net +1
  inline block to compensate. The `opaque_ptr` raw pointer is
  derived once at the outer-function scope (`let opaque_ptr = opaque
  as *mut TqScanOpaque`).

## Unsafe block counts

| File | Before | After | Δ |
| --- | ---: | ---: | ---: |
| `src/am/ec_hnsw/scan.rs` | 97 | 96 | -1 |
| **HNSW subsystem subtotal** | **486** | **485** | **-1** |

Two caller wraps removed (-2); one inner `unsafe { &*opaque_ptr }`
added in the layer-0 inner closure (+1). Net -1.

Cumulative rotation delta:

| Stage | HNSW total |
| --- | ---: |
| Pre-399 | 549 |
| After 414 | 486 |
| After 415 | 485 |

Net rotation delta: **-64 in HNSW** (-11.7%).

## Soundness rationale

- The function body now operates entirely through the `&mut
  TqScanOpaque` parameter. All borrow scopes are explicit and
  validated by the borrow checker.
- The new innermost `let approx_score = { ... }` block isolates the
  immutable borrows on `opaque` to the scoring expression; the
  borrow ends at the closing brace before any mutable call.
- The new `unsafe { &*opaque_ptr }` block inside the layer-0
  `keep_neighbor_tid` closure is the standard FnMut-nested-borrow
  pattern: the outer closure holds `&mut TqScanOpaque`, so the
  inner closure (a `FnMut(NodeId) -> bool`) cannot capture another
  borrow of `opaque` directly. Reborrowing the raw `opaque_ptr` per
  inner call is the same pattern existing code uses for nested
  closures in HNSW.

No anti-pattern B introduced: the lifted function's signature takes
a reference, not a raw pointer with an unbounded `'a`.

## Validation

Artifacts under `reviews/task-50/415-hnsw-scan-successor-candidates-safe/artifacts/`:

- `manifest.md`
- `per-file-after.log`
- `diff.patch` (302 lines)
- `cargo-check-pg18.log` — clean.

## Performance gate

Scan hot path. Behavior unchanged: same candidates produced in same
order with same scores. The per-iteration re-acquisition of
`quantizer` / `binary_query` is a pointer deref into already-cached
state. No allocation, no extra indirection. Bench deferred per
`feedback_coder_push_smoke_checks`.

## Out of scope

- `initialize_scan_entry_candidate` and other top-level driver
  functions in scan.rs that still thread `opaque: *mut TqScanOpaque`
  through the AM callback. These wrap the lifted helpers in their
  own `unsafe fn` shells; lifting them depends on the AM-callback
  boundary work in a future slice.
- Removal of `scan_opaque_ref` / `scan_opaque_mut` helpers
  themselves (anti-pattern B). Their remaining call sites are inside
  the still-`unsafe fn` driver shells; queued.
