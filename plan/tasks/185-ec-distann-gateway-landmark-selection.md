# Task 185: ec_distann Gateway Landmark Selection

Status: **in progress — Phase 1 gateway attribution preregistered**
(2026-08-07; entry gate added 2026-07-29). Priority: P1 bounded-recall
follow-up, independent of Task 184.

> **Entry gate and boundary (Task 203 audit, correction 2026-07-29-02).**
>
> This task remains **valid and is not superseded**. Its lever is correct: for the
> promoted `training_landmarks_exact` policy the candidate pool is already the
> entire corpus (`head_sample.rs:462-467` scores every node's code), so the
> selection *objective* — not the pool — is what controls which seeds are
> returned. The unexplained fact this task exists to attack still stands: three
> distinct 4,096-row objectives produced identical top-32 seeds.
>
> **Gate satisfied: Task 206 has reported.** This task's diversity-aware
> returned-seed arm penalizes landmarks that share a traversal basin, and that
> cannot pay off at the current BW=4, where the beam pops four candidates per
> round. Running it now would reproduce `NEG-01`'s structure — a seed policy
> measured at a width that cannot exploit it — and would burn the candidate.
>
> **Boundary against Task 207**, so the two head lanes do not collide:
> - **185 owns the selection objective** — which landmarks are chosen, and which
>   of them are returned as seeds.
> - **207 owns the pool, the search path, and sharding** — per-partition union
>   construction (§3), restoring the persisted Vamana graph instead of the
>   4,096-point exact scan, and distributing the head across the roster (§2.2).
>
> With that split they are independent and may run in sequence without
> re-baselining each other. They must not run concurrently.
>
> Evidence: `reviews/task-203/001-decision-reaudit/` Defect 3 and its correction.

Program ledger: `plan/design/ec-distann-recall-latency-roadmap.md`. This task
owns `HEAD-13` through `HEAD-20`, `HEAD-32`, and `HEAD-33`. It does not own
larger or hierarchical heads.

## Why

Task 182's trained cap-4,096 head improved 100k recall from 0.9275 to 0.9625,
while the same-generation O(N) owner oracle reached 0.9970 with the same graph,
BW4/H100, and RaBitQ traversal. Task 183 then built two distinct alternative
4,096-row heads, but exact scoring returned the same ordered top-32 seeds and
the same 0.9625 recall. Their objectives changed unused landmark-tail members,
not the seeds that control traversal.

The next fixed-cap question is therefore not whether another policy represents
owner-nearest seeds. It is whether landmarks and returned seeds can be selected
for marginal success under the actual bounded traversal: gateway nodes that
lead to ground-truth neighborhoods, diversity across traversal basins, and hard
training-query coverage.

## Goal

Measure at cap 4,096 whether a deterministic, disjoint-training gateway
objective improves held-out end-to-end recall over Task 182's frequency policy.
Advance at most one fixed-cap candidate to a separate production task.

This task is measurement-first. Benchmark-only builders, basin diagnostics,
and selection policies are allowed. Production defaults, formats, graph,
BW/H, neighbor codec, and materialization remain unchanged.

## Frozen controls

- Task 182 `training_landmarks_exact`, cap 4,096, exact scoring, 32 seeds;
- graph degree 32, BW4/H100, RaBitQ traversal, exact final ranking;
- the Task 182 corpus and held-out evaluation identities;
- a separately attested training slice and a disjoint policy-selection
  validation slice; and
- the same three-owner exact/disjoint physical topology.

The owner oracle remains an O(N) diagnostic and cannot be selected.

## Phase 1: gateway and basin attribution

For each training query, measure which bounded candidate seeds lead the normal
traversal to each ground-truth result. Emit compact aggregates for:

1. per-seed traversal success and marginal truth coverage;
2. redundancy between seeds that enter the same reachable basin;
3. hard-query clusters and zero-success seed sets;
4. seed-to-result and seed-to-expanded-region overlap;
5. stability across deterministic training/validation splits; and
6. memory, construction time, and bounded work.

No evaluation-query outcome may influence policy selection.

## Phase 2: fixed-cap screen

Pre-register and compare only:

1. Task 182 frequency/owner-seed coverage control;
2. one gateway set-cover policy ranking landmarks by marginal bounded-traversal
   truth coverage (`HEAD-17`--`HEAD-20`, `HEAD-33`); and
3. one diversity-aware returned-seed policy that penalizes shared traversal
   basins (`HEAD-13`--`HEAD-15`, `HEAD-32`).

All arms persist exactly 4,096 landmarks, exact-score all 4,096, return at most
32 seeds, and keep graph/traversal/query inputs byte-identical. If the builder
and seed selector both change, measure them as separate A/B cells before a
combined cell so attribution is preserved.

Select by held-out recall first, then overlapping-CI warm p50, cached bytes,
and construction time. Training or validation metrics never break an
evaluation tie.

## Full-scale confirmation

Only one useful 100k candidate proceeds to 10k/50k/100k. Use at least 200
held-out queries / 2,000 distinct top-10 trials and 50 warm latency samples
after 10 warmups at concurrency 1. Record recall/CI, latency distribution,
storage/cache/build cost, diagnostics, topology, remote engagement, query
separation, and unanimous release provenance through checked-in
`ecaz bench suite` configs.

## Decision

Advance only a deterministic fixed-cap candidate that improves deficient-scale
recall without regressing another measured scale or hiding a material latency,
storage, cache, or construction cost. Otherwise close with STOP and pass the
measured limitation to conditional Task 186.

The candidate remains benchmark-only until a separately numbered production
task accepts its format, lifecycle, and full production-path evidence.

## Required review packets

1. `reviews/task-185/001-program-roadmap-and-scope/`: roadmap, task split, and
   imported candidate review;
2. `reviews/task-185/002-gateway-attribution/`: disjoint input contract,
   gateway/basin diagnostics, and frozen policies;
3. `reviews/task-185/003-fixed-cap-screen/`: isolated cap-4,096 A/B evidence;
4. `reviews/task-185/004-full-scale-decision/`: conditional 10k/50k/100k and
   advance/stop decision.

## Non-goals

- Larger head caps, compressed heads, or hierarchical routing (Task 186).
- Graph construction/search-budget changes (Task 188).
- Neighbor codec changes (Task 189).
- O(N) owner scans, evaluation training, or post-hoc policy choice.
- Remote payload/materialization work (Task 184).

## References

- Tasks 181--183 and their accepted closeout packets.
- `plan/design/ec-distann-recall-latency-roadmap.md`.
- FR-080, NFR-007, and NFR-017 through NFR-020.
