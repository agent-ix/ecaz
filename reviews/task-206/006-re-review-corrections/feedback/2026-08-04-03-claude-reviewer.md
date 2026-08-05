---
agent: claude
role: reviewer
model: claude-fable-5
date: 2026-08-04
seq: 03
---

# Task 206 packet 006 — re-review corrections: ACCEPT; Task 206 is closeout-ready with one follow-up

All three blocking items from the previous round are resolved:

- **k_head A/B is now live and proven live.** The feature-build lane records
  effective `head_seed_count=128` vs `200` with distinct per-arm seed-id
  digests at all three scales — exactly the activation evidence that was
  missing. Result: recall is flat (10k 0.9883/0.9870, 50k 0.9600/0.9600,
  100k 0.9584/0.9587), so NEG-01 is genuinely requalified at BW=64: seeds
  beyond 128 do not buy recall. The inert release comparison is withdrawn
  and the packet-005 supersede notes say why. Feature-lane latency correctly
  quarantined as diagnostic.
- **Default wording fixed** — shipped default stated as BW4/H100, BW64/H8 as
  the measured recommendation for a separate productionization task.
- **Telemetry is truthful and works on the physical path.** The emit now
  runs on both physical return paths with the effective seed count;
  unmeasured fields print `absent`/`unmeasured` instead of fake zeros; and a
  real captured round shows transport_wait=1.24 ms, straggler=1.02 ms,
  512 B request / 112 KB response at BW=64. Note what that single round
  already establishes: at ~8 rounds that is roughly 10–20 ms of transport
  against a ~190 ms physical p50 — the latency gap to single-index is
  compute/serialization on the owners, not network wait. That attribution
  sentence deserves to be in the packet, because it redirects future latency
  work (BatANN/batching territory, not transport).

One item stays open as a follow-up, not a closeout blocker:

- **P2 — structured per-round capture still does not reach results.jsonl.**
  The packet's `run/results.jsonl` contains zero `scan_round` rows; the
  durable evidence is one hand-copied `round=0` line for one arm at one
  scale in `seed-and-notice-evidence.log`, and the per-node logs that
  contained the notices were (reasonably) not committed. The
  `physical_benchmark_scan_round` parser exists but evidently never fires in
  the suite lane. Fix the capture path (or have the fixture write a compact
  per-arm round log into artifacts) so the next run of this lane produces
  per-round rows for all arms without hand-copying. File it; do not re-run
  the matrix for it.

With that filed, I consider Task 206 complete on its own terms: ceiling and
axes landed, the 100k grid measured with attribution-honest evidence, NEG-01
requalified at the winning width, the 10k/50k/100k winner matrix on the
uninstrumented release build with NFR-021 conforming, and a correctly-scoped
defaults recommendation (BW64/H8) that leaves the default change to the
operator-approved productionization task. Merge/closeout is the operator's
call; from the review side this bucket is done.
