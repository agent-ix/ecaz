---
agent: claude
role: reviewer
model: claude-fable-5
date: 2026-08-04
seq: 01
---

# Task 206 packet 003 — full-scale decision: REQUEST CHANGES (not a closeout)

The 10k and 50k diagnostics are honest about what they are, the numbers cited
in `request.md` match `run-50k-retry/results.jsonl`, and the release
provenance rows are present. But this packet is the *decision* lane, and none
of the decision content exists yet:

1. **P1 — no defaults recommendation.** The task's Goal is "produce a
   defaults recommendation." The packet ends at raw diagnostic numbers with
   no recommendation, no Pareto discussion, and no disposition of the current
   BW=4/H=100 default.

2. **P1 — the full matrix was run on the wrong point.** Phase 4 is "Full
   matrix on the winner." Packet 002's Pareto candidates are BW64/H8
   (0.9584 / 187.7 ms) and BW128/H8 (0.9700 / 209.5 ms). This packet's 10k
   and 50k rows are BW32/H8 — a shape packet 002 already showed is dominated
   at 100k (0.8361, vs single-index control 0.8224 — i.e. at 100k the BW32/H8
   physical arm barely beats the single-index baseline on recall while paying
   5× its latency). A BW32/H8 10k/50k lane is a fixture-health diagnostic,
   not the winner matrix. The 10k/50k/100k closeout needs to run the
   recommended default.

3. **P1 — latency evidence is not decision-grade: 2 warmups and 5 timed
   queries** (50k summary, same shape at 10k). Five samples cannot support a
   p50 comparison that will justify a default change. The 100k sweep used 50
   iterations; the closeout matrix must at least match that.

4. **P2 — NFR-021 admissibility is again unstated, and the 50k numbers put
   it in play.** Physical generation storage 1,242,734,592 bytes vs the
   single-index control's 444,186,624 bytes is a 2.80× cluster amplification
   at 50k. Whatever the verdict is under NFR-021's growth rows, the packet
   must state it rather than omit it — this is the constraint most likely to
   halt the 206–209 chain, and silence reads as "not checked."

5. **P2 — phase 3 (NEG-01 / k_head requalification at the winning width) has
   not been run anywhere in the bucket.** It is a named task phase and a
   closeout prerequisite; see also packet 002 feedback item 4.

Suggested path: pick the recommended point from packet 002 (state the
tradeoff between BW64/H8 and BW128/H8 explicitly), run the standard
10k/50k/100k recall+latency+storage matrix on that point with proper
iteration counts plus the owner-traversal control, add the k_head A/B at that
width, state the NFR-021 verdict, and write the recommendation. Note the
sweep's existing 100k rows can serve as the 100k cell only if the closeout
point and seed configuration match them exactly.
