---
task: 183
packet: 006-full-scale-decision
role: coder
status: open
date: 2026-07-17
head: 8609926e9
---

# Review request: Task 183 STOP decision

Task 183 produced no benchmark candidate for full-scale confirmation:

- Phase 1 rejected exact-neighbor traversal: recall was 0.9605 versus 0.9625
  for RaBitQ, while warm p50 rose from 43.8 ms to 113.1 ms.
- Phase 2 built two deterministic alternative cap-4,096 heads. Their persisted
  head digests differed, but exact scoring returned the same ordered top-32
  seeds as control for every held-out query, and all three arms measured 0.9625
  recall. Both alternatives were rejected.
- Phase 3 required a Phase 2 winner, so its cap/routing experiments were
  skipped under the pre-registered condition.
- Phase 4 found that remote payload materialization consumed 26.955 ms of the
  40.20 ms warm mean (67.05%). Head scoring consumed 2.272 ms (5.65%) and seed
  selection 0.101 ms (0.25%). Dominance outside the eligible Task 183 changes
  was pre-registered to yield no latency candidate.
- The permitted small-corpus branch is explicitly declined. Task 182's trained
  arm was recall-flat and 4.3 ms slower at 10k, but it is an opt-in/default-off
  build policy; current-sample construction already remains the unchanged 10k
  default. An automatic corpus-size substitution would introduce a new
  production policy and unresolved threshold, and no 10k stage profile or
  isolated bypass implementation was selected.

The full-scale 10k/50k/100k matrix was conditional on a selected candidate.
Running it now would compare no change and would provide no promotion evidence,
so it is skipped rather than misrepresented as task closeout measurement.
Task 182's already-complete 10k/50k/100k evidence remains the production
baseline; Task 183 changed no production behavior, default, or persisted format.

Decision: **STOP Task 183 with no candidate.** Task 184 is the focused
measurement-first follow-up for remote payload materialization. It will
decompose owner execution, transport/bytes, coordinator decode/copy, request
scheduling, and unused eager work before selecting at most one isolated change.

Please review the conditional-skip reasoning, cross-packet evidence chain, and
the Task 184 handoff. This request remains open under the packet workflow;
outside review is recorded below and was not a prerequisite to recording the
completed measurement outcome.

Outside review ACCEPT is recorded at
`feedback/2026-07-17-01-reviewer.md`. F183-1 is resolved by the explicit
small-corpus disposition above; the STOP decision is unchanged.
