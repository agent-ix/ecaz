---
task: 184
packet: 003-isolated-candidate
role: coder
status: open
date: 2026-07-19
head: 245c2054f
---

# Review request: bounded incremental payload candidate

Runner checkpoint `eacd00264` adds a backwards-compatible same-generation
materialization arm to `ecaz bench suite`. Candidate checkpoint `245c2054f`
implements only the packet-002 preregistered MAT-01/MAT-04 candidate behind the
existing attribution benchmark feature and a default-zero GUC. Production
builds contain neither the GUC nor the candidate branch.

Correctness-runner checkpoint `7c2254e21` adds an opt-in suite-driven semantic
matrix. It compares eager and batch-10 ordered JSON output for unfiltered
identity, first-window rejection, multiple-window rejection, null payloads, and
wide compressed-inline varlena projection/qual behavior; it also requires mixed local/remote
executor consumption. After timed arms have completed, a cursor fetch proves a
remote first batch completed, deliberately stops the remote owners, and
requires the next demanded batch to error before the owners are restarted.
Every scenario is emitted as a structured suite result row with provenance.

The eager path still fetches every remote ranked hit. With batch size 10, the
candidate retains only remote `vec_id` identity in ranked output slots. When the
executor reaches a pending slot, it fetches the remaining pending remote rows
in that deterministic global-ranked 10-slot window, capped at the proven search
prefix. Qual rejection naturally advances into subsequent windows; this is not
a `LIMIT k` cutoff. Existing search deepening and its finite cap remain the work
bound.

Payload fetches reuse the existing concurrent owner endpoint, planner-derived
projection attnums, schema and epoch fingerprints, pooled connections, ordered
decode checks, tombstone handling, and error propagation. A failure in any
later batch aborts the query; no materialized prefix is returned as complete.
Stable-prefix deepening begins a rebuilt window at the executor's consumed
cursor so already consumed remote rows are not fetched again.

Feature and normal PG18 builds pass, as does the focused deterministic-window
test.

## Completed evidence and proceed decision

The checked-in suite completed both the 10k semantic gate and matched 100k A/B.
Every preregistered semantic scenario passed, including output identity, quals
rejecting one and multiple windows, NULL and wide varlena payload datums,
projection/qual use, mixed local/remote winners, and a real remote-owner outage
after a completed first batch that aborted the later fetch.

At 100k both variants retained 0.9625 distinct recall (95% CI
0.9532--0.9700) on 200 queries / 2,000 trials and selected the same seed IDs.
Warm mean latency improved from 39.30 to 23.40 ms (-40.5%); p95 from 50.50 to
26.50 ms (-47.5%); p99 from 55.60 to 27.50 ms (-50.5%); and max from 56.20 to
28.10 ms (-50.0%). Remote materialization fell from 25.910 to 10.292 ms/query.
Requested remote payloads fell from 26.84 to 6.64/query, exactly matching remote
executor consumption, and logical payload bytes fell from 496,003 to 122,707
per query. Storage and construction were shared and identical.

Topology, remote engagement, query separation, and unanimous release extension
provenance passed. This is a material end-to-end and tail win with no observed
quality, semantic, failure, storage, or construction tradeoff, so packet 003
records **PROCEED** to the required packet-004 10k/50k/100k confirmation. The
candidate remains feature-gated and default-off until that full decision.
