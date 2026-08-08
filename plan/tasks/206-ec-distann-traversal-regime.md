# Task 206: ec_distann Traversal Regime (Wide Beam, Few Rounds)

Status: **complete — review-closed 2026-08-05; recommendation subsequently
REJECTED by the Task 215 release A/B 2026-08-06.** Priority: P0 latency.

Entry gate (satisfied): Task 205's pushdown landed and its A/B dispositioned.
Widening the beam without pushdown reproduces Task 194's result by construction.

Outcome. The corrected conforming matrix measured the wide-beam/few-round
regime and requalified effective seed count; **BW64/H8 with 128 effective seeds
was the measured recommendation** and the shipped default was left at BW4/H100.
Scan-round capture was fixed and review-closed
(`reviews/task-206/007-scan-round-capture/feedback/2026-08-05-01-claude-reviewer.md`),
with transport only ~10--20 ms across eight rounds against a much larger
physical scan time.

**The recommendation did not survive productionization.** Task 215's normal
release A/B measured the same BW64/H8 configuration at 22.6--31.6 ms mean
against this task's accepted 194--231 ms rows. The reconciliation
(`reviews/task-215/003-release-matrix-and-decision/artifacts/reconciliation-206.md`)
attributes the ~8x gap to a work-surface difference — `top_k=200`/L200 here
versus `top_k=10`/effective L64 there — with matching per-scale query hashes and
warm-cache protocol. **Task 206's absolute latency rows must not be reused as a
normal-release forecast.** Task 215 is authoritative for the shipped default.

## Why

Task 162's G0 kill-check — the measurement that unblocked the entire ec_distann
program — concluded (`reviews/task-162/003-g0-killcheck/request.md:10-26`):

> "**Wide beam, few rounds is the only viable multinode shape.** BW=4 needs H=64
> for 0.995 -> projected 78-142 ms (dead). BW=32 reaches 0.994 at H=8. **This
> matches the DistributedANN paper's regime** and should inform the M2 defaults
> (beam_width default 4 is a single-node default; multinode wants >=32)."

Its table: BW=32/H=8 at 0.9940 recall, 12.3 ms compute, 20.3-28.3 ms projected
multinode ("under, 1.3-1.9x headroom"); BW=4/H=64 at 77.6-141.6 ms ("far over").

**The default was never changed.** `mod.rs:253` is still `BEAM_WIDTH = 4`,
`:260` `HOP_ROUNDS = 100`, and every distann suite from Task 179 onward is pinned
there. The `hop_rounds` GUC still reads "provisional until the M0 recall-vs-H
kill-check measurement pins it" (`options.rs:331`); the kill-check ran
2026-07-07.

BW=4 was inherited from `ec_diskann` (`ec_diskann/options.rs:29-32`), whose Task
168 A/B swept beam width to fill 32-wide **local SIMD kernels with no network in
the loop**. A cache line and a network round trip were tuned with one constant.

Structural blockers:

- `ECDISTANN_MAX_BEAM_WIDTH = 64` (`mod.rs:254`) makes the paper's grid
  (BW 96-192; production BW=128) unreachable **even as a session GUC**.
- `top_k` default 10 vs the paper's k=L=200, and `profiles.rs:218-235` treats
  `top_k`, not BW/H, as the quality axis — so BW/H are not sweep axes at all.
- Seed count is `max(BW*2, 32)` (`generation_read.rs:2650`) vs k_head=200.
- **BW >= 32 has never been run on the distributed path.** The only BW=32 rows
  are single-node 50k from the kill-check, before remote code existed.
- The one wide-beam distributed test (Task 179 packet 066, BW16/H25) held
  `BW x H = 400` **fixed** — trading rounds for width at constant work. No
  experiment has raised the budget the way the paper does.

## Goal

Measure the paper's traversal regime on the distributed path with pushdown
present, and produce a defaults recommendation. **This task does not change the
production default on its own** — see Non-goals.

## Phases

1. **Raise the ceiling.** Lift `ECDISTANN_MAX_BEAM_WIDTH` so the paper's grid is
   expressible, and make BW/H first-class sweep axes in the suite rather than
   `top_k` alone. Confirm the NFR-019 `BW x H` bound still holds and is asserted
   at the new values.
2. **Sweep.** BW in {32, 64, 128} x H in {4, 5, 8} at 100k, with `top_k`/`L`
   raised toward 200. Re-test Task 162's BW=32/H=8 point under the real
   multinode fixture. Report hop rounds, transport wait, straggler spread,
   expanded nodes, and request/response bytes per round, not just end-to-end.
3. **Requalify the seed negative.** `NEG-01` (seeds 64/128 "recall-flat") was
   measured only at BW=4, where the beam pops 4 per round and extra seeds are
   structurally unusable. Re-test `k_head` at the winning width.
4. **Full matrix on the winner.** 10k/50k/100k recall + latency + storage.

## Benchmark gate

`ecaz bench suite`, owner-traversal arm as control (NFR-022), NFR-021
admissibility recorded at pre-registration, per-node storage rows present
(Task 204). Report the recall/latency Pareto rather than a single point: the
paper accepts higher single-query latency for throughput and IO efficiency, so a
wide-beam point that improves recall at equal latency is a win even if p50 does
not fall.

## Required review packets

1. `reviews/task-206/001-ceiling-and-axes/` — the ceiling lift and suite axes.
2. `reviews/task-206/002-100k-sweep/` — the grid and its attribution.
3. `reviews/task-206/003-full-scale-decision/` — 10k/50k/100k and the defaults
   recommendation.

## Non-goals

- **Changing the shipped default.** This task recommends; a default change is a
  separate operator-approved productionization task with its own release A/B.
- Head or seed-selection policy changes beyond re-testing `k_head` (Task 207).
- Graph degree or codec changes.
- Reviving the traversal replica.

## References

- `reviews/task-162/003-g0-killcheck/` — the unapplied finding.
- `reviews/task-203/001-decision-reaudit/` Defect 1.
- `DISTRIBUTEDANN` §4 (H=5, BW=128, R=72, k=L=200, k_head=200), Figure 4 grid.
- Voided ledger rows `TRAV-14`, `TRAV-15`; qualified `NEG-01`.
