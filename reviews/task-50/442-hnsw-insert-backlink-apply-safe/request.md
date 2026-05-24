# Task 50/442: HNSW insert.rs — backlink-mutation apply + plan chain safe-fn lifts

## Why this slice

Slice 437 flipped `plan_backlink_mutation` and
`select_backlink_rewrite_slice` to safe `fn`. With those leaf
planners now safe and `load_exact_graph_element` /
`load_graph_neighbors` already safe (slice 422), the entire
backlink mutation chain composes safe operations and can be lifted.

## Scope

Four `unsafe fn` → safe `fn` lifts in `src/am/ec_hnsw/insert.rs`:

1. `apply_backlink_mutations` — dispatcher over per-page mutation
   runs; body composed of safe ops.
2. `add_backlinks_on_page` — page-mutation primitive scaffolding
   kept as internal narrow `unsafe { ... }` blocks
   (`LockedBufferGuard::read_main`, `wal::GenericXLogTxn::start`,
   `with_writable_page_tuple_bytes`).
3. `plan_backlink_mutations` — body composed of safe ops:
   `load_exact_graph_element`, `load_graph_neighbors`,
   `plan_backlink_mutation`.
4. `add_backlinks_to_forward_neighbors` — body composed of safe
   ops after the chain lifts.

Caller-side `unsafe { ... }` wraps stripped (four):

- `apply_backlink_mutations` internal call to
  `add_backlinks_on_page`.
- `add_backlinks_to_forward_neighbors` internal call to
  `apply_backlink_mutations`.
- `add_backlinks_to_forward_neighbors` internal call to
  `plan_backlink_mutations`.
- `HnswInsertAdapter::add_backlinks_to_forward_neighbors` method
  wrap (caller-side at line ~488).

## Unsafe block counts

| File | Before | After | Δ |
| --- | ---: | ---: | ---: |
| `src/am/ec_hnsw/insert.rs` | 30 | 26 | -4 |
| **HNSW subsystem subtotal** | **343** | **339** | **-4** |

Cumulative rotation delta:

| Stage | HNSW total |
| --- | ---: |
| Pre-399 | 549 |
| After 441 | 343 |
| After 442 | 339 |

**Net rotation delta: -210 in HNSW (-38.3%).**

## Soundness rationale

`add_backlinks_on_page`'s body retains three narrow internal
`unsafe { ... }` blocks: `LockedBufferGuard::read_main` (returns a
guard whose Drop releases the lock), `wal::GenericXLogTxn::start`
(returns a transaction wrapper whose Drop finalizes the WAL
transaction), and the closure passed to
`with_writable_page_tuple_bytes` (page-bytes mutation). All three
are page-mutation primitives the rotation has deliberately left as
narrow `unsafe { ... }` blocks rather than fully encapsulated safe
fns, because each one's safety contract depends on the live
relation/buffer pairing being upheld by the surrounding aminsert
protocol.

Lifting the `unsafe fn` signatures preserves these internal
contracts as `unsafe { ... }` blocks while relieving callers from
re-declaring an `unsafe fn` contract above an already-narrow scope.

## Validation

Artifacts under `reviews/task-50/442-hnsw-insert-backlink-apply-safe/artifacts/`:

- `per-file-after.log`
- `diff.patch`
- `cargo-check-pg18.log` — clean.

## Performance gate

aminsert backlink-application path; signature-only change. Bench
evidence gathered out-of-band per
`feedback_coder_push_smoke_checks`.

## Rotation milestone

**-210 (-38.3%)** on HNSW: 549 → 339. The -30% Exit Criteria
target now has an **8.3-point cushion**.
