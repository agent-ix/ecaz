---
task: 185
packet: 005-suite-arbitrary-head-trace
role: coder
status: open
date: 2026-08-07
seq: 01
head: 22ed70bb9
---

# Task 185 arbitrary-head attribution checkpoint

This packet wires the arbitrary persisted-head attribution endpoint from
packet 004 into the canonical `ecaz bench suite` runner and records the first
corrected full-head 100k diagnostic. It still does not claim a selector, an
A/B result, or a production change.

## Change

Commit `9627d36c2` adds two explicit suite options:

- `gateway_head_candidate_trace` enables the diagnostic;
- `gateway_head_candidate_positions` supplies bounded, 1-based persisted-head
  positions to trace per disjoint training query.

The CLI validates the physical benchmark and training-slice prerequisites,
checks positions are within `1..=4096`, preflights the new extension endpoint,
and writes one packet-directed JSON trace per physical variant. The suite
artifact manifest includes those JSON outputs, so a future run is resumable
and review-visible.

The diagnostic uses the same physical settings as the control and invokes the
feature-only endpoint from packet 004. Commit `22ed70bb9` removes an internal
benchmark-path clamp that had exposed only 96 members (32 per owner) despite
the persisted 4,096-member head. Production callers continue to pass 32.
The corrected run traces positions 1, 64, 512, 2,048, and 4,096 for each of
200 disjoint training queries, yielding 1,000 isolated reruns against the
full persisted head.

The ordered truth-hit union is 48, 83, 126, 139, and 147 of 2,000 exact
training truth items as those five positions are added. Positions 2,048 and
4,096 add only 13 and 8 new truth hits after the earlier probes, while
position 512 adds 43. This is useful evidence for a gateway selector screen,
not evidence that a sparse position rule should be promoted.

## Validation

- PG18 extension feature check with
  `distann-head-attribution-benchmark`: pass, exit 0.
- PG18 `ecaz-cli` check: pass, exit 0; one pre-existing dead-code warning.
- `git diff --check`: pass.
- PG18 feature and featureless checks after the full-head fix: pass.
- The corrected `ecaz bench suite` 100k run: pass, release profile, three
  physical owners, remote-owner checks passed, 4,096-member head, and 1,000
  traces at the five preregistered sparse positions.
- Control recall@32 was `0.9205`, CI `[0.9078, 0.9316]`, with 200 queries and
  2,000 distinct top-10 trials; warm latency was `40.30 ms`; construction was
  `934711 ms`; physical generation was `2,496,626,688` bytes and index-space
  amplification was `1.351147`.
- Exact-truth mapping covered all trace hit IDs and matched all 4,096 head
  membership entries. The compact analysis is packet-local under `artifacts/`.

The next step is the preregistered fixed-cap screen: construct one training
gateway set-cover selector and one diversity-aware returned-seed selector,
then compare both with the frozen control at 100k. Only one useful candidate
may proceed to the required 10k/50k/100k confirmation.

## Review focus

Please review the suite validation, position bounds, SQL parameter construction,
artifact registration, the full-head clamp correction, and the fact that this
remains an explicit diagnostic rather than a default or persisted policy.
