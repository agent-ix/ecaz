---
task: 179
packet: 037-head-cap-suite-surface
role: coder
status: review-requested
head: efb0aa8cb4c5d9bb6f61b88b03baa671a4f9d10c
date: 2026-07-12
---

# Review request: suite-driven DistANN head-cap sensitivity

Please review commit `efb0aa8cb` and the exact-SHA evidence under
`artifacts/`.

Packet 030's outside reviewer requires a 10k/50k/100k `head_index_cap`
sensitivity sweep driven by `ecaz bench suite`. The existing
`distann-local-multinode` step could not vary that reloption. This checkpoint
adds the missing narrow runner surface before the measurement packet uses it.

The requested decisions are:

1. Does `head_index_cap` flow from SuiteConfig to the dev fixture and into both
   the physical distributed-control index and same-data single-index control?
2. Does validation enforce the extension's frozen `16..=1048576` range?
3. Do structured build, recall, latency, storage, and remote-engagement rows
   carry the cap, preserving per-arm attribution in `results.jsonl`?
4. Does the default remain 4096 for existing configs that omit the field?

Exact-SHA compile and focused tests pass. The checked-in dry-run manifest shows
the expanded command includes `--head-index-cap 256` together with the physical
benchmark arguments.

This packet is only the runner prerequisite. It does not provide or decide the
10k/50k/100k sensitivity evidence, which will land in a subsequent measurement
packet.
