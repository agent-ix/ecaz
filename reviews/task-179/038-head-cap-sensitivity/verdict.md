# Verdict: retain head_index_cap 4096

Retain the frozen FR-080/ADR-085 default of 4096.

The real three-owner 10k/50k/100k suite shows a monotone quality benefit from a
larger persisted head at the scale that distinguishes the choices. Cap 64 loses
0.030 physical recall at 100k. Cap 256 recovers 0.025 but remains 0.005 below
4096. The 4096 default also restores the final 0.005 at 10k and is tied with 256
at 50k.

The quality comparison is not confounded by incomplete placement: every one of
the nine cells has exact Ready/Published row coverage, no non-owned or orphan
records, and two proven remote owners. All 27 suite thresholds pass.

The tradeoff is bounded and visible. In the same-data local control index,
4096 costs about 33 MB more than cap 64 and about 31.5 MB more than cap 256.
The standard physical-generation storage metric excludes the shared coordinator
head catalog, so this packet does not claim the physical persisted head has zero
storage cost. Default-cap latency also includes one cold head fill; the warm
majority p50 is favorable, but this is not presented as a fully warmed AC-13
latency closeout.

This packet closes FR-080-AC-4 and the head-cap sensitivity portion of packet
030 P2-1 for outside review. It does not close the requested removed-O(N)
seed-scan A/B, Task 179 AC-13 latency, NFR-018, or the packet 033 P1 findings.
