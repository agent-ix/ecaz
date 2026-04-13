# Review Request: C1 ADR-031 Persisted Sidecar A/B Compare

## Context

Packet `285` landed the first persisted ADR-031 sidecar slice:

- optional trailing binary sidecars on element tuples
- bulk-build write support on the supported no-QJL `4-bit` lane
- scan reads prefer persisted sidecars and derive only as fallback

That slice is already green and pushed, and the first cold read on the rebuilt
real `50k` index came back at `5.537ms` for a single `m=8`, `ef_search=40`
query.

## Problem

We now know persisted ADR-031 sidecars work. We still do **not** know whether
they are worth carrying.

The missing evidence is an A/B measurement against the same codebase with
persisted sidecars deliberately ignored at runtime, so the scan falls back to
binary-word derivation on cache miss.

Without that same-build comparison, the cold read from packet `285` is just an
absolute number, not a clear value judgment.

## Planned Investigation

Add the smallest safe comparison seam that:

- leaves persisted sidecars on disk
- forces scan-time binary-word derivation instead of using them
- is easy to switch on and off for a local benchmark

Then run cold real-`50k`, `m=8`, `ef_search=40` reads on both modes and record
the delta.

## Success Criteria

- the packet records the exact A/B switch used
- the packet records cold measurements for persisted-sidecar `auto` vs
  `derive-only`
- the packet makes a clear keep/drop call for persisted ADR-031 sidecars
