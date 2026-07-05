# Task 41 invariant #2 review request: DSM slot scoping and completion audit

## Scope

This packet covers code commit `56c6ed63111c49caa7689b86fff2a7da2e6b6dde`
(`Scope HNSW build DSM neighbor slot slices`) and the final invariant #2
completion audit at that head.

The code slice changes `src/am/ec_hnsw/build_parallel.rs` so DSM neighbor-slot
helpers no longer return unconstrained Rust slice lifetimes. The former
`concurrent_dsm_node_slots` and `concurrent_dsm_node_slots_mut` helpers are now
scoped callbacks:

- `with_concurrent_dsm_node_slots`
- `with_concurrent_dsm_node_slots_mut`

Callers copy selected forward slots, read successor slots, mutate backlink
slots, and perform the test assertion inside the callback while the surrounding
DSM state and locks remain live.

## Completion Checklist

Task 41 invariant #2 requires Rust values borrowing from PostgreSQL-owned
memory not to outlive the owning memory context or resource lifetime.

- Detoast / varlena: closed by packets 115-123. The fresh detoast inventory
  now shows only guard-internal detoast/byte-slice implementations.
- Slot Datums: closed by packets 124-127. Fresh inventory shows remaining
  reads are audited immediate-copy/decode paths or by-value slot output writes.
- Palloc scan state: closed by packets 128-131 and this packet. Scan opaque
  query/list slices are owner methods, and HNSW build DSM code/source/neighbor
  slices are callback-scoped.
- Page/buffer tuple bytes: closed by packets 134-146. Remaining raw tuple
  byte creation is helper-internal, metadata/special-page local, synchronous
  SQL input copying, or DSM/test/message handling recorded in the inventory.
- C strings/catalog names: closed by packet 132. Uses are owned `String`
  conversions or synchronous non-escaping reads while their PG owner remains
  live.

## Remaining Accepted Raw Sites

`artifacts/final-memory-lifetime-inventory.log` is intentionally non-empty.
The accepted categories are:

- local detoast guard internals;
- page tuple view helpers introduced in packets 134-146;
- metadata/special-page fixed-size reads under existing buffer guards;
- scan/build owner methods whose return values are tied to opaque/build state;
- DSM initialization/readback/test/message slices, with escaping code/source
  and neighbor-slot APIs callback-scoped;
- synchronous SQL receive buffers that are immediately copied to owned `Vec`;
- C-string reads covered by the Phase E audit.

## Validation

See `artifacts/manifest.md` for command metadata.

- `cargo fmt --all --check` passed with the repository's existing stable-rust
  rustfmt configuration warnings.
- `cargo check --no-default-features --features pg18` passed with the known
  pre-existing unused imports warning in `src/am/mod.rs`.
- `git diff --check HEAD -- src/am/ec_hnsw/build_parallel.rs reviews/task-41/147-invariant2-completion-audit` passed.

## Reviewer Focus

- Confirm the DSM neighbor-slot helpers no longer expose free `&[u32]` or
  `&mut [u32]` lifetimes.
- Confirm the completion checklist maps every invariant #2 surface to either
  a code packet or an audit packet.
- Confirm the accepted raw sites in the final inventory are genuinely
  non-escaping or helper-internal.
