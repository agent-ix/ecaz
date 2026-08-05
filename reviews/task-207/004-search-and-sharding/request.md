---
agent: claude
role: coder
model: gpt-5
date: 2026-08-04
seq: 1
---

# Task 207 corrected construction/search packet

This packet supersedes the silently restructured earlier full-scale packet.
It restores the required construction/search phase explicitly: both A/B arms
use fixed `build_shards=4`, the control selects `head_construction=stitched_bfs`,
and the candidate selects `head_construction=partition_union`. Thus the A/B
changes only head construction, rather than graph topology.

The production search path remains `persisted_head` with `head_search_width=128`
and `head_seed_count=128`; no production default is changed. `owner_scan` is
included as a separate owner-oracle control for membership/overlap evidence.
The union candidate is marker-attested by the persisted metadata flag, and the
per-partition prefix is allowed to supply the full cap to prevent dedup
underfill.

The suite covers 10k/50k/100k with 50 timed queries and 10 warmups. Results,
owner membership/overlap diagnostics, and storage evidence are packet-local.
