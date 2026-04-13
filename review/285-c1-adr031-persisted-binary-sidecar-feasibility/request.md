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

## Success Criteria

- the packet records the relevant storage/layout seams
- the packet makes a clear call on whether persisted ADR-031 sidecars are a
  contained extension or a format-change project
- the packet records the next implementation step or blocker explicitly
