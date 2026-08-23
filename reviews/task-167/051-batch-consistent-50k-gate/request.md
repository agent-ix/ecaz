---
agent: codex
role: coder
model: GPT-5
date: 2026-08-22
seq: 1
---

# Task 167 batch-consistent 50k gate

Status: measurement preregistered; result pending. No Task 167 acceptance or
closeout is claimed.

Packet 047 is the before arm: after 160 isolated robust-prune-all inserts, its
50k heldout physical-vs-fresh deficit was `0.008611`, missing the fixed
`0.007000` band by `0.001611`. Packet 050 reconciled that physical behavior
against both batch Vamana and the shared incremental planner: both append a
backlink while a target has spare degree and only robust-prune at capacity.

This packet changes only that backlink default. It preregisters one isolated
50k fixture at the same operating point, with the same 160 insert sources, 200
heldout queries, 48 inserted-neighborhood queries, exact fp32 truth, and fixed
packet-045 bands. The diagnostic robust-prune-all control is forbidden from
mutating the fixture before quality is measured.

The threshold will not be widened after observing this run. Failure rejects
the candidate and leaves Task 167 open. Passing this 50k branch point is not
closeout; it permits the required isolated 10k/50k/100k recall, latency, and
storage confirmation.

Configuration and provenance are in
[`artifacts/manifest.md`](artifacts/manifest.md).
