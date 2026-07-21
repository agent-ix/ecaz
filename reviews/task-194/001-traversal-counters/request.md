---
task: 194
packet: 001-traversal-counters
role: coder
status: review_requested
date: 2026-07-21
seq: 1
---

# Task 194 counter-slice pre-registration

This packet carries forward Task 187's complete nine-way contract. The current
implementation exposes only aggregate `local_expand`, `remote_expand`, and
`traversal_total` timers; those are insufficient for selecting a traversal
candidate. This slice adds feature-gated, warmup-reset
counters for coordinator partition/frontier, connection/session state,
request encode/bytes, owner reads/decode, owner scoring, response
encode/bytes, transport wait/owner stragglers, coordinator decode/frontier
insert, and hop/batch/cache/repeat work. The hop/batch/request/response/
frontier/repeat counters are emitted as attribution-work rows. The feature
build now also exposes coordinator partition/request encode, owner graph-read
and scoring, transport wait, coordinator decode, and frontier-insert timers.
The owner endpoint's internal request encode/response encode split remains a
follow-on refinement; these counters are explicitly labeled in the packet
results rather than presented as a false exact reconciliation.
