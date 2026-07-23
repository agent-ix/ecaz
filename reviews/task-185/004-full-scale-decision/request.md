---
task: 185
packet: 004-full-scale-decision
role: coder
date: 2026-07-23
head: c83ea6ea8426df0ae5ddc4e8dec55f68db801a94
status: review_requested
---

# Review request: Task 185 STOP decision

## Decision

**STOP.** Do not advance or productionize either fixed-cap candidate.

Packet 003's pre-registered 100k screen found:

- gateway set-cover selected the same 4,096-node membership as the frequency
  control and tied its 0.9625 evaluation recall;
- basin diversification tied recall while increasing warm mean latency from
  about 20 ms to 66--67 ms; and
- no storage or other Pareto improvement that could make either candidate
  useful.

Task 185 says only one useful 100k candidate proceeds to 10k/50k/100k. There
is none, so the full-scale branch is intentionally not entered. This is a
conditional skip, not a benchmark-evidence waiver or an environment deferral.

## Handoff

Task 186's entry gate is satisfied by a residual fixed-cap seed limitation:
changing the fixed-cap objective and query-conditioned seed diversity did not
move held-out recall, while the same-generation owner oracle remains far
higher. Task 186 should retain the Task 182 frequency policy and first run the
transparent cap-8,192 exact-scoring capacity control. Cap 16,384 remains
conditional on a useful monotonic 8,192 signal; compressed or hierarchical
routing must remain separately attributed and explicitly bounded.

No Task 185 benchmark-only policy becomes a production default. The diagnostic
surfaces remain available only under their benchmark feature for durable
negative-evidence reproduction.

## Review questions

1. Is STOP the only decision consistent with the pre-registered selection
   rule?
2. Is skipping the conditional full-scale branch correct given the flat and
   materially slower candidates?
3. Is the Task 186 handoff narrow enough to prevent redesigning the fixed-cap
   objective inside the capacity experiment?
