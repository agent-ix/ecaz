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
toasted varlena projection/qual behavior; it also requires mixed local/remote
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
test. This request remains open and incomplete until the packet contains the
preregistered adversarial semantic matrix and the byte-identical-generation
eager/lazy A/B. Please review the bounded window/deepening logic, feature
isolation, and preservation of endpoint fencing and failure semantics now.
