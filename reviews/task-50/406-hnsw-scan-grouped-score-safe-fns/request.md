# Task 50/406: HNSW scan.rs — lift grouped-score family to safe fn

## Why this slice

Per the user direction to push for bigger structural lifts after the
small-pattern rotation reached diminishing returns. `scan.rs` is the
densest HNSW file. The `score_grouped_*` family of internal helpers
all take `opaque: *mut TqScanOpaque` and immediately derive a
`scan_opaque_ref`/`scan_opaque_mut` borrow internally. They are
`unsafe fn`s only because of that derivation; the bodies have either
zero or already-bounded internal `unsafe { ... }` blocks.

Converting the family to take `&TqScanOpaque` / `&mut TqScanOpaque`
directly:

- moves the soundness obligation outward into the callers (which are
  themselves `unsafe fn`s, so the obligation chain remains explicit),
- lets every caller drop its `unsafe { score_grouped_*(opaque, ...) }`
  wrapper, since the callee is now a safe function call,
- avoids anti-pattern B: the new signatures take `&` / `&mut` references
  bound by the caller's frame, not safe-fn-returning-`&'a T` from a raw
  pointer.

The "soundness obligation chain stays explicit" point matches the
reviewer's preferred shape in
`reviews/task-50/401-hnsw-shared-header-ref/feedback/2026-05-22-01-reviewer.md`:
"unsafe fn" callers continue to acknowledge the obligation via their own
unsafe-fn signatures; only the leaf helpers stop being unsafe-fn.

## Scope

Five functions in `src/am/ec_hnsw/scan.rs` converted from `unsafe fn`
taking `*mut TqScanOpaque` to safe `fn` taking the appropriate reference:

1. `score_grouped_search_code_from_scan_state` → safe, takes `&TqScanOpaque`.
2. `score_grouped_candidate_context_binary` → safe, takes `&TqScanOpaque`.
3. `score_grouped_heap_source_from_scan_state` → safe (was already `&mut TqScanOpaque`).
4. `score_grouped_candidate_heap_rerank` → safe, takes `&mut TqScanOpaque`.
5. `exact_score_grouped_candidate_context` → safe, takes `&mut TqScanOpaque`.
6. `score_grouped_candidate_context_exact` → safe, takes `&mut TqScanOpaque`.

Caller-side rewrites (still inside `unsafe fn` bodies, so no per-call
`unsafe { ... }` wrapper is needed once the callee is safe):

- 9 caller sites updated to call the safe functions, passing
  `scan_opaque_ref(opaque)` / `scan_opaque_mut(opaque)` to derive the
  reference from the surrounding `*mut TqScanOpaque` parameter.
- One dispatcher (`score_grouped_candidate_context`) restructured to
  compute both layer predicates (`exact_layer`, `binary_score`) up
  front so the immutable `opaque_ref` borrow can end before the
  later `scan_opaque_mut(opaque)` call. The restructure is
  semantically identical (both predicates are read-only side-effect
  free).

## Unsafe block counts

| File | Before | After | Δ |
| --- | ---: | ---: | ---: |
| `src/am/ec_hnsw/scan.rs` | 136 | 127 | -9 |
| **HNSW subsystem subtotal** | **525** | **516** | **-9** |

Cumulative rotation delta:

| Stage | HNSW total |
| --- | ---: |
| Pre-399 | 549 |
| After 399 | 541 |
| After 400 | 540 |
| After 401 | 535 |
| After 402 | 528 |
| After 403 (anti-pattern B fix) | 529 |
| After 404 | 526 |
| After 405 | 525 |
| After 406 | 516 |

Net rotation delta: **-33 in HNSW**.

## Soundness rationale

Each converted leaf function previously had `unsafe fn` signature only to
defer the raw-pointer borrow to the caller. The conversion moves the
borrow to the call site:

```rust
// Before:
unsafe fn score_grouped_search_code_from_scan_state(opaque: *mut TqScanOpaque, ...) -> f32 {
    let opaque = scan_opaque_ref(opaque);  // unsafe fn call (inside unsafe fn body)
    ...
}

// After:
fn score_grouped_search_code_from_scan_state(opaque: &TqScanOpaque, ...) -> f32 {
    // opaque already a borrow
    ...
}
```

Callers (which remain `unsafe fn` because they take `*mut TqScanOpaque`
themselves) acquire the borrow inline:

```rust
// Before:
let score = unsafe { score_grouped_search_code_from_scan_state(opaque, search_code) };

// After (in unsafe fn body — scan_opaque_ref is unsafe fn so no `unsafe { }` needed):
let score = score_grouped_search_code_from_scan_state(scan_opaque_ref(opaque), search_code);
```

This pattern does not introduce anti-pattern B. The new safe-fn
signatures take references, not raw pointers — the borrow checker
enforces lifetime bounds. `scan_opaque_ref` / `scan_opaque_mut` remain
in the codebase as pre-existing `unsafe fn`s that anchor the borrow at
each call site.

## Validation

Artifacts under
`reviews/task-50/406-hnsw-scan-grouped-score-safe-fns/artifacts/`:

- `manifest.md` — head SHA, files touched, validation mapping.
- `per-file-after.log` — post-change HNSW per-file block counts.
- `diff.patch` — exact diff applied (223 lines).
- `cargo-check-pg18.log` — `cargo check --no-default-features --features
  pg18` (lib smoke). Clean.

## Performance gate

Scan hot path. No semantic change: every score computation reads the
same byte the previous deref read; every record_* call mutates the
same field; every `score_and_cache_scan_element` invocation runs with
the same arguments. The conversion is pure type-signature shape — no
allocation, no inlining hint change, no extra indirection.

Bench evidence deferred per `feedback_coder_push_smoke_checks`
(2026-05-21).

## Out of scope

- `score_grouped_candidate_context_approx` — still `unsafe fn`. Its
  three caller contexts each hold an immutable `opaque_ref` borrow when
  the call would happen, so converting requires the same dispatcher
  restructure pattern used here (lift the predicate up so the borrow
  ends). Queued as next slice.
- `exact_score_cached_graph_element`, `cached_graph_element`, and
  the other `unsafe fn`s in scan.rs that take both `index_relation`
  and `*mut TqScanOpaque` — bigger lift, queued.
