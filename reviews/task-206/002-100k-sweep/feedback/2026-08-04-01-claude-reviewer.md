---
agent: claude
role: reviewer
model: claude-fable-5
date: 2026-08-04
seq: 01
---

# Task 206 packet 002 — 100k sweep: REQUEST CHANGES

The mechanics of the run are good: provenance rows in
`run-100k-retry/results.jsonl` confirm `extension_build_profile=release`,
unanimous across 3 nodes, at SHA `59aeb6c58`; topology/serving gates passed
with zero non-owned rows and zero orphans; the cited recall/latency/storage
numbers all match `results.jsonl` exactly (I re-derived the table); per-node
storage rows are present per Task 204; run dir was under
`$ECAZ_CLUSTER_ROOT` and removed after capture. The recall result is real and
important — BW128/H8 at 0.9700 vs the single-index control at 0.8224 is the
first distributed-path confirmation of the Task 162 regime finding.

Blocking findings:

1. **P1 — the task's required per-round observability is entirely absent.**
   Task 206 phase 2: "Report hop rounds, transport wait, straggler spread,
   expanded nodes, and request/response bytes per round, not just
   end-to-end." `results.jsonl` contains only end-to-end recall and p50/p95
   rows; no per-round metric of any kind exists in the packet. This is not
   bookkeeping: every physical arm sits at 114–210 ms p50 against a 35.8 ms
   single-index control, versus Task 162's 20.3–28.3 ms projection for
   BW=32/H=8. A 4–6× gap to projection with zero attribution (transport wait
   vs straggler spread vs compute) means the sweep cannot support a defaults
   recommendation on the latency axis, which is the P0 axis of this task.
   Either wire the per-round counters through the suite step or record why
   they cannot be captured, then re-emit at least one arm with attribution.

2. **P1 — NFR-022 owner-traversal control arm is missing.** The benchmark
   gate in the task file says "owner-traversal arm as control (NFR-022)". The
   only control in the config and results is the `single` single-index arm.
   Without the owner arm the sweep has no upper-bound reference on the same
   fixture, and the physical-vs-oracle recall gap at each BW/H point — the
   quantity Task 207 depends on — is unmeasured.

3. **P1 — NFR-021 admissibility was not recorded at pre-registration.** The
   task file requires it; `grep -rn NFR-021 reviews/task-206 reviews/task-207`
   returns nothing. Physical generation storage is invariant at 2,496,659,456
   bytes across arms — fine — but no admissibility verdict is stated anywhere
   in the packet.

Non-blocking:

4. **P2 — `head_seed_count=200` is baked into all nine arms**, so the BW32/H8
   point is not a re-test of Task 162's configuration (which ran the default
   seed policy), and phase 3's NEG-01 requalification (k_head A/B at the
   winning width) is pre-empted rather than performed — seed width never
   varies anywhere in the packet. Keep 200 as the sweep default if you like,
   but the NEG-01 requalification still needs its own A/B before closeout.
5. **P3 — the manifest records only the debug-binary audit/dry-run commands.**
   The actual `run` command line (and binary) that produced the retry is not
   in `manifest.md`; the release provenance is only recoverable from
   `results.jsonl`. Record the run command verbatim.
6. **P3 — state the fixture-contention caveat.** Three owner nodes share one
   host; at BW=128 all three expand concurrently per round. Say explicitly
   that absolute p50s are local-fixture numbers, so the Pareto ordering, not
   the absolute values, is the decision input.
