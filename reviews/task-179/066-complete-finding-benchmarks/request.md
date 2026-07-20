---
task: 179
packet: 066-complete-finding-benchmarks
role: coder
status: review-requested
head: 45f9f0f980d9548083aa965659df4de7089b6e18
date: 2026-07-14
---

# Review request: complete-finding benchmark evidence

Please review the four `ecaz bench suite` arms and the derived comparison in
`comparison.md` as the measurement response to packet 060's P2-6/P2-10 and
residual outside-roster evidence findings.

Requested decisions:

1. Does baseline `0b2d4fbab` versus candidate `45f9f0f98` show recall-neutral
   owner fixed-cost improvement at 10k/50k/100k, with latency and storage
   reported rather than inferred?
2. Does the fixed-product BW16/H25 arm support retaining BW4/H100 as the
   default while recording the wider/shorter traversal as a recall/latency
   tradeoff?
3. Does the outside-roster arm prove that all three remote owners participate
   at 10k/50k/100k with clean topology, recall, latency, and storage evidence?
4. Are the explicit 1m and heap-versus-TOAST dispositions accurate and
   sufficiently bounded?

All final suite manifests report 3 completed, 0 failed, 0 missing, and 0
stale steps; all final audits and configured thresholds pass. The first
BW16/H25 attempt's disk-full failure and clean resume are retained rather than
hidden. Raw PostgreSQL node logs and regenerable run directories were pruned
under the repository evidence rules.
