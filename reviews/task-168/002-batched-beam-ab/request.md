# Review request: Task 168 Phase 2 — width-W batched-beam + default W=4

- Branch: `task-168-diskann-batched-beam`
- Commits under review: `1adbc7784` (batched-beam implementation, GUC
  default 1) and the follow-up default-flip commit (`ECDISKANN_DEFAULT_BEAM_WIDTH`
  1 → 4) landed with this packet.
- Evidence: `artifacts/manifest.md` — 22-step A/B suite (W∈{1,8} × 3 scales
  + 50k width-pick) plus a 6-step W=4 fill-in, all on the release backend
  over the packet-001 rabitq fixture.

## Summary

- The greedy loop now pops up to `ec_diskann.beam_width` admissible frontier
  entries per round and scores the deduplicated union of their fresh
  neighbors with one `score_batch` call. Width 1 reproduces the legacy loop
  pop-for-pop (`sc_011c` asserts result equality for W∈{2,4,8,64} on the
  chain fixture). Query scans only; insert planning and vacuum edge repair
  stay width-1. This function is the ec_distann FR-081 hop-round primitive
  Task 162 will consume.
- **A/B verdict: W=4 default.** Wins every 50k sweep point (6–9%) and every
  100k sweep point (3–18%; 14.6 → 12.3 ms at L=800), recall never below the
  W=1 reference at any cell (100k low-L recall *improves*, +0.85 pp at
  L=64/W=8). Cost: +0.18 ms at 10k L=64. W=8/16/32 over-expand at 50k.
- Mechanism confirmed: ≥32-wide flushes 0.2% → 98.7% of hops at 100k
  L=800. Remaining frontier residual share (~71%) is allocation work —
  Phase 4's target, unchanged in rank.

## Asks

1. Approve the W=4 default (`SET ec_diskann.beam_width = 1` restores the
   legacy loop; range 1–64).
2. Confirm the 10k L=64 +0.18 ms trade is acceptable against the 50/100k
   wins (release anchors live at 100k).
3. Sanity-check the beam admission semantics (bound tightens intra-round as
   each popped emittable candidate inserts — `scan.rs`
   `greedy_descent_beam_with`).
