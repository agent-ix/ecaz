# Task 50/414: HNSW scan.rs — `cached_upper_layer_seed_candidate` safe-fn lift

## Why this slice

`cached_upper_layer_seed_candidate` was the smaller of the two
remaining `unsafe fn(*mut TqScanOpaque, ...)` candidates after slice
413. Its body has zero internal unsafe blocks of its own — it only
wraps the still-`unsafe fn cached_scan_successor_candidates_for_layer`
call inside the closure passed to
`graph::greedy_descend_with_successors`. That closure remains
`unsafe { ... }` until the inner function lifts, but the parent's
signature can flip to safe `fn(_, &mut TqScanOpaque, _, _)` now,
removing the lone caller-side wrap.

## Scope

- `cached_upper_layer_seed_candidate` lifted from
  `unsafe fn(_, *mut TqScanOpaque, _, _)` to safe
  `fn(_, &mut TqScanOpaque, _, _)`.
- Closure body keeps the `unsafe { cached_scan_successor_candidates_for_layer(...) }`
  wrap with a re-explained SAFETY comment naming the reborrow
  pattern. `graph::greedy_descend_with_successors` accepts an
  `FnMut(NodeId, u8) -> Vec<...>` closure; the closure captures
  `opaque: &mut TqScanOpaque` and reborrows-as-`*mut` at each call,
  letting it pass into the still-unsafe-fn callee.
- One caller in `initialize_scan_entry_candidate` drops its
  `unsafe { ... }` wrap and passes `opaque: &mut TqScanOpaque`
  directly (no `opaque_ptr` cast needed for this branch).

## Unsafe block counts

| File | Before | After | Δ |
| --- | ---: | ---: | ---: |
| `src/am/ec_hnsw/scan.rs` | 98 | 97 | -1 |
| **HNSW subsystem subtotal** | **487** | **486** | **-1** |

Cumulative rotation delta:

| Stage | HNSW total |
| --- | ---: |
| Pre-399 | 549 |
| After 413 | 487 |
| After 414 | 486 |

Net rotation delta: **-63 in HNSW** (-11.5%).

## Soundness rationale

The closure's `unsafe { ... }` block is bounded to the closure body
and reborrows the captured `&mut TqScanOpaque` as `*mut T` per call.
That's the standard FnMut pattern. No anti-pattern B.

## Validation

Artifacts under `reviews/task-50/414-hnsw-scan-upper-layer-seed-candidate-safe/artifacts/`:

- `manifest.md`
- `per-file-after.log`
- `diff.patch`
- `cargo-check-pg18.log` — clean.

## Performance gate

Scan hot path. Bench deferred per `feedback_coder_push_smoke_checks`.

## Out of scope

- `cached_scan_successor_candidates_for_layer<KeepFn>` — still
  `unsafe fn` because its body has a long-lived
  `cached_quantizer_ref(opaque_ref) -> &ProdQuantizer` borrow that
  overlaps the many `scan_opaque_mut(opaque)` reborrows in the
  scoring loop. Lifting requires either (a) narrowing the quantizer
  borrow per scoring branch, (b) extracting a Copy
  `QuantizerHandle`, or (c) splitting the function into a
  binary-prefilter half and a scoring half. Queued.
- Branch renamed to `task-50-hnsw` at end of this packet's
  push cycle.
