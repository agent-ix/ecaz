# Review Request: C1 ADR-031 Persisted Binary Sidecar Feasibility

## Context

Packet `281` landed the cached ADR-031 runtime path on `main`.

Packets `282` and `283` then established:

- the cached ADR-031 path clears `NFR-001` on the normative real `50k` lane at
  `m=8`, `ef_search=40`
- the same live graph path does not introduce a new runtime recall loss versus
  exact quantized results at that target seam

That makes cached ADR-031 a real keep. The next implementation question is
whether the binary-sign codes should stay scan-local and derived on cache miss,
or whether they should be persisted in the index tuples as a sidecar.

## Problem

Persisting the ADR-031 binary codes could remove query-time derivation cost on
cache miss and simplify the hot path, but it would also add durable storage
overhead and may require tuple-layout or index-version changes.

Before implementing anything, we need a concrete answer to:

- where the binary sidecar would live in the current tuple/page layout
- whether it can fit as a backwards-compatible extension
- which subsystems would need to write and read it

## Planned Investigation

Inspect the current seams for:

- element tuple encoding/decoding
- page-local tuple storage layout
- build-time element tuple emission
- any existing version or optional-payload support that could host a `192B`
  binary sidecar

If the answer is "feasible in the current format", the next step should be a
small implementation slice on write/read plumbing.

If the answer is "this is really an index-v2 change", record that explicitly
instead of pretending it is a cheap patch.

## Storage Readout

Relevant seams:

- [src/am/page.rs](/home/peter/dev/tqvector/src/am/page.rs) defines the element
  tuple payload directly in `TqElementTuple` / `TqElementTupleRef`
- [src/am/build.rs](/home/peter/dev/tqvector/src/am/build.rs) writes element
  tuples during bulk build
- [src/am/insert.rs](/home/peter/dev/tqvector/src/am/insert.rs) writes element
  tuples during incremental insert and computes page-fit / max-insert-level from
  `TqElementTuple::encoded_len(code_len)`
- [src/am/graph.rs](/home/peter/dev/tqvector/src/am/graph.rs) and
  [src/am/scan.rs](/home/peter/dev/tqvector/src/am/scan.rs) are the main read
  consumers

Important facts:

- the metadata page currently has no explicit format/version field
- element tuples are fixed-length for a given `code_len`
- the current element payload is:
  - tag / level / deleted
  - inline heap tids
  - heap-tid count
  - gamma
  - neighbor tuple tid
  - packed code bytes
- build and insert both assume element tuple length comes from
  `TqElementTuple::encoded_len(code_len)`

Feasibility conclusion:

- persisted ADR-031 sidecars do **not** look like an automatic index-v2 change
- the cleanest shape is an **optional trailing payload** after `code`
- old tuples can keep the current payload length
- new tuples can append persisted binary-sign bytes
- decoders can distinguish old vs new tuples from the tuple length and expose an
  optional borrowed binary slice

Why this is viable:

- `update_raw_tuple(...)` already enforces same-length rewrites, so existing old
  tuples would stay old instead of being silently reshaped in place
- inserts into an older index could append new-format tuples without breaking
  old-format tuple reads, as long as decode handles both lengths
- scan can use persisted binary words when present and keep the current
  derivation fallback when absent

Page-fit impact:

- current 1536-dim, 4-bit no-QJL element payload is `74B + 768B = 842B`
- a persisted binary sidecar adds `192B`, bringing the element payload to
  `1034B`
- with `m=8`, the level-0 neighbor payload is `99B`, so a colocated
  element+neighbor pair still fits comfortably on an `8KB` page
- the higher-level insert cap would shrink slightly, but only because the
  element tuple gets larger; this does not look like a catastrophic layout
  break

## Readout

Persisted ADR-031 sidecars look like a **contained extension**, not an
automatic index-v2 project.

The next implementation slice should be:

1. extend `TqElementTuple` / `TqElementTupleRef` with an optional trailing
   binary sidecar
2. teach build and insert to write it for the no-QJL 4-bit lane
3. teach graph/scan reads to use the persisted sidecar when present and derive
   only as fallback

## Success Criteria

- the packet records the relevant storage/layout seams
- the packet makes a clear call on whether persisted ADR-031 sidecars are a
  contained extension or a format-change project
- the packet records the next implementation step or blocker explicitly
