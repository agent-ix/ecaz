# Task 50/405: HNSW source.rs — collapse ArrayType field derefs

## Why this slice

`src/am/ec_hnsw/source.rs` had two sequential `unsafe { (*array_ptr).field }`
derefs (`ndim` and `elemtype`) inside the real-array validation block of
the float4 source decode path. Each was its own `unsafe { }` block. The
borrow is bounded by the in-scope `detoasted: DetoastedFloat4Datum` guard,
so we can collapse the two derefs into a single
`let array_header = unsafe { &*array_ptr };` and read fields off the typed
reference safely.

Same frame-bounded inline-borrow shape as packets 403 and 404. The
following `unsafe { pg_sys::array_contains_nulls(array_ptr) }` FFI call
remains unsafe because that is an irreducible FFI boundary.

## Scope

- Single inline borrow at the start of the array-validation block:
  `let array_header = unsafe { &*array_ptr };` (held alive by `detoasted`).
- Field reads now go through `array_header.ndim` and
  `array_header.elemtype` (no `unsafe { ... }` wrapper required).
- `array_contains_nulls(array_ptr)` and `flat_array_data_offset(array_ptr)`
  call sites are unchanged — they remain unsafe FFI / unsafe fn calls.

## Unsafe block counts

| File | Before | After | Δ |
| --- | ---: | ---: | ---: |
| `src/am/ec_hnsw/source.rs` | 38 | 37 | -1 |
| **HNSW subsystem subtotal** | **526** | **525** | **-1** |

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

Net rotation delta: **-24 in HNSW**.

## Validation

Artifacts under
`reviews/task-50/405-hnsw-source-array-header-borrow/artifacts/`:

- `manifest.md` — head SHA, files touched, validation mapping.
- `per-file-after.log` — post-change HNSW per-file block counts.
- `diff.patch` — exact diff applied.
- `cargo-check-pg18.log` — `cargo check --no-default-features --features
  pg18` (lib smoke). Clean.

## Performance gate

Not on a scoring or traversal hot path. Build/insert source-attribute
decode runs per heap tuple but the change is a borrow lift, not a code-path
change — `array_header.ndim` reads the same byte the previous
`(*array_ptr).ndim` read did, no extra indirection, no allocation. Bench
evidence deferred per `feedback_coder_push_smoke_checks`.

## Out of scope

- The downstream `flat_array_data_offset` function still has a single
  `unsafe { (*array_ptr).dataoffset }` deref; that function is itself
  `unsafe fn` and its caller bundling is fine as-is.
- Further structural lifts in source.rs (heap-source scorer reshape,
  attribute kind dispatching) — queued.
