# Review Request: C1 ADR-031 On/Off A/B

## Context

Packet `288` established two things on the current Tier 1 build:

1. the high-`ef_search` frontier is still fast on the full real `50k` table
2. a fair `queries_50` comparison against the old A4 seam shows lower Recall@10
   than the earlier A4-era slice

That leaves the main diagnostic question unresolved:

- is the high-`ef_search` recall drop actually caused by ADR-031, or
- is it from later non-ADR-031 evolution in the graph/runtime/index state?

## Problem

The current build has a same-build toggle for persisted sidecar usage
(`tqhnsw.force_binary_derivation`), but it does **not** yet have a same-build
way to disable ADR-031 runtime behavior entirely.

Without that seam, we can compare old packets to new packets, but we cannot do
the decisive test:

- same codebase
- same fixture
- same query table
- same `ef_search`
- ADR-031 fully enabled vs fully disabled

## Planned Slice

Add the smallest hidden diagnostic seam that disables ADR-031 runtime behavior
entirely by skipping binary-query preparation during scan setup.

That should turn off both:

- binary-sign prefilter scoring
- ADR-031-driven lazy exact scoring on cache miss

while leaving the rest of the current build untouched.

Then run the current build with ADR-031:

1. enabled
2. disabled

on the same high-`ef_search` recall seams.

## Success Criteria

- the packet records the exact on/off switch used
- the packet records same-build recall results for ADR-031 enabled vs disabled
  on the same real-corpus seam
- the packet makes a clear call on whether ADR-031 is the cause of the
  high-`ef_search` recall drop
