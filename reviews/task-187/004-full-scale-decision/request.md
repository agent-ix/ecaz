---
task: 187
packet: 004-full-scale-decision
role: coder
status: review_requested
date: 2026-07-21
seq: 1
---

# Full-scale decision

Task 187 closes on the STOP branch. No production code or quantizer/index
behavior changed, so no new 10k/50k/100k candidate matrix is warranted. The
accepted Task 191 packet 003 remains the full-scale production lazy10 evidence;
this task adds the fresh 100k traversal attribution and records that remote
owner transport is the next investigation target.

Next task: carry forward the complete nine-way Phase 1 contract: coordinator
frontier/owner partitioning; connection/session/prepared-state; request encode
and bytes; owner directory/graph reads and decode; owner approximate/exact
scoring; response encode and bytes; transport wait and per-owner straggler
distribution; coordinator receive/decode/frontier insertion; and hop count,
batch widths, nodes requested/returned, cache hits, and repeated reads. Then
evaluate exactly one bounded transport optimization with the standard
10k/50k/100k A/B suite.
