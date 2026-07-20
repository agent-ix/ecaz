---
task: 179
packet: 065-search-shape-suite
role: coder
status: review-requested
head: a7fa64895
date: 2026-07-13
---

# Review request: suite-driven DistANN search shape

Please review CLI commit `a7fa64895`, which makes packet-060's requested
fixed-product BW/H A/B and outside-roster scale run first-class
`ecaz bench suite` configurations.

The `distann-local-multinode` step now accepts validated `beam_width` and
`hop_rounds` fields, passes them as session GUCs to both recall and latency
workers, and records them on every normalized physical benchmark result. The
storage row now exposes aggregate `control_index_bytes`. Outside-roster
engagement also counts every owner as remote instead of subtracting the
coordinator unconditionally.

This packet validates the runner surface only. The measurement packet will use
it for the 10k/50k/100k matrices; no benchmark claim is made here.
