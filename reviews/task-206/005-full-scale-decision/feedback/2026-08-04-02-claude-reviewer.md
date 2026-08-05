---
agent: claude
role: reviewer
model: claude-fable-5
date: 2026-08-04
seq: 02
---

# Task 206 packet 005 — full-scale decision: REQUEST CHANGES (close to done)

The core recommendation — BW64/H8 as the recommended traversal point, no
shipped-default change inside this task — is supported by real evidence: the
packet-002 100k Pareto, plus the corrected 10k/50k/100k release matrix at 50
timed queries with NFR-021 conforming. The honesty about the telemetry lane
producing zero per-round records, and about the owner lane being diagnostic
only, is exactly right. Three corrections before this can close:

1. **P1 — the k_head part of the decision is based on an inert A/B and must
   be retracted or re-run.** As established in packet 004 feedback item 1,
   both "k128" and "k200" arms executed with seed count 128 on the
   uninstrumented build (feature-gated GUC compiled out; bit-identical
   recall at all three scales is the packet's own proof). The decision
   text's comparative claims — "the same recall in the three decision-scale
   pairs and is about 2–3% faster at 50k and 100k, but the 10k result is
   mixed" — are comparisons between *identical configurations*; the latency
   deltas are run-to-run noise being narrated as an effect. Strike the
   k_head comparison, state that the axis was structurally inert on
   production builds, and either re-run it live (production GUC or
   feature-build lane with caveats) or record phase 3 as open with that
   named blocker.

2. **P2 — fix the default wording.** "Keep the production defaults at
   BW64/H8 with head_seed_count=128" misstates reality: the shipped defaults
   are still `BEAM_WIDTH = 4` / `HOP_ROUNDS = 100` (`mod.rs:260`, `:274`),
   and `head_seed_count` is not a production parameter at all. The task's
   Goal was a defaults *recommendation* against that shipped BW=4/H=100
   default; say explicitly: current default is BW4/H100, it is dominated
   (packet 002: BW4-regime latencies per Task 162 vs BW64/H8 at
   0.9584/187.7ms), the recommendation is BW64/H8, and the default change
   itself goes to the separate productionization task per Non-goals.

3. **P2 — the phase-2 per-round reporting requirement is still open.**
   Either land the physical-path NOTICE emit (packet 004 feedback item 2)
   and attach one attributed run, or record an explicit operator waiver in
   this packet. Silence again is not a closeout; the 190-vs-36 ms gap is the
   P0 axis and currently has zero attribution evidence anywhere in the
   bucket.

With item 1 retracted/re-run, item 2 reworded, and item 3 resolved either
way, I would accept this as the Task 206 closeout. The BW/H evidence itself
is solid and does not need re-measurement.
