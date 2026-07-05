# Task 160: promote or retire ec_spire.pre_materialization_prune (default-off speedup GUC)

Status: **proposed** (2026-07-04). Owner: unassigned. Priority: P3

## Why

`ec_spire.pre_materialization_prune` (default **false**,
`src/am/ec_spire/options/mod.rs:80`) drops scored candidates below the running
min-IP before row materialization (`pre_materialization_prune` use at
`src/am/ec_spire/candidates.rs:3152`). It is a shipped, gated speedup that has
never been promotion-benched — no packet records an A/B for it. Tasks 77/78
established SPIRE latency is dominated by scored-candidate volume; a prune
that cuts materialization work downstream of scoring is exactly the kind of
lever that family of findings points at, and it is sitting off by default
without evidence either way. Related default-off SPIRE GUCs
(`remote_search_global_pre_heap_merge`, `remote_search_initial_threshold_early_stop`,
`adaptive_nprobe`) should at least be inventoried in the packet so the
"default-off with no evidence" set shrinks deliberately.

## Scope

- A/B `pre_materialization_prune` on/off on the ec_spire RaBitQ lane (and TQ
  if cheap) at 10k/50k/100k via `ecaz bench suite`: recall + latency +
  storage, plus candidate/materialization counters to attribute the effect.
- Decision: flip default on, keep opt-in with documented guidance, or retire
  the GUC (dead-option cleanup) — one of the three, recorded in the packet.
- Coordinate with the active SPIRE task lane (141-146 spire series on the
  other desktop) before flipping any default — this task's evidence must not
  race a concurrent SPIRE default change.

## Out of Scope (hard)

- No new pruning logic; this benches the existing gate only.

## Gate / Exit Criteria

- The on/off matrix at 10k/50k/100k with recall parity analysis and one of
  promote/keep/retire recorded. Closes on the decision.
