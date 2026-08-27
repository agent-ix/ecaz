---
task: 235
packet: 003-2pc-lifecycle-fault-matrix
agent: Codex
role: coder
model: gpt-5
date: 2026-08-25
seq: 01
---

# Task 235 PG18 2PC and lifecycle fault matrix

Please review the Task 235 production-code checkpoint
`b871d5481376df87c60ae486d68bb78519944c21` and the completed fixed-harness
benchmark run captured with runtime/runner checkpoint
`b802fe3690beb53f9b2695332a163a9d1a8fb56f`.

The final release+`pg_test` three-node PG18 matrix ran over Task 236's
verify-full mutual-TLS transport with client-certificate authentication and
plaintext rejection. Extension preflight matched the clean repository head on
all nodes. The fixture passed exactly 23 scenarios and emitted 107 records:
eight lifecycle replay cells, one operator status-unavailable cell, and
fourteen write/recovery cells.

The lifecycle matrix injects a lost acknowledgement after each production
handoff/publish/retire/cancelled-reclaim participant operation. It asserts the
mixed participant state, drives the existing recovery API, verifies the final
participant and coordinator decision state, repeats recovery idempotently, and
checks reclaimed relation residue. Build handoff begin/stage/seal recovery uses
the intentional `abort_then_new_build` contract because a new backend must not
silently recapture a different source snapshot for the same build identity.

The write matrix covers clean commit; failure before mutation; failure after
endpoint mutation; owner backend death during mutation; coordinator death
after prepare followed by immediate owner crash/restart; lost precommit-intent,
commit-prepared, and rollback-prepared acknowledgements; one-owner partial
completion; missing intent detection; full prepared-slot saturation; and owner
death during routed tombstone application. Every cell asserts source rows,
source-map rows, owner graph/current/row state, directory validity, prepared
transactions, nonterminal intents, and the required retry result. Duplicate
recovery always emits zero actions.

The matrix exposed and closed three concrete defects while becoming
secure-current:

- cancelled-generation reclaim compared a coordinator audit UUID with a
  participant-local logical UUID; recovery now validates the coordinator UUID
  against the generation descriptor and the participant UUID/node against the
  descriptor roster;
- Task 235 reaper/operator fixture sessions replaced secure roster entries with
  plaintext socket conninfos; every recovery path now preserves the selected
  secure fixture; and
- the owner crash cell restarted PostgreSQL through a plaintext-only helper;
  it now restores SSL, both secure listeners, and secret-backed conninfos.

Focused `remote_transport::tests` passed 19/19. The packet also contains the
clean-head CLI build, SSL PG18 release install, full final console, compact
matrix, exact commands, hashes, timestamps, and fixture provenance.

Please focus review on lost-ack decision authority, coordinator/owner crash
recovery, lifecycle replay authorization, the cancelled-generation identity
fix, secure transport preservation during recovery/restart, and whether the 23
cells fully cover Task 235 acceptance items 1--3.

## 2026-08-26 throughput closeout update

The one blocker carried by reviewer feedback is now complete. The registered
`ecaz bench suite` ran the fixed harness at 10k/50k/100k with
`skip_single_control: false`, five 32-row trials per scale, secure physical
transport, and exact extension provenance. The immutable benchmark source is
`benchmarks/task235-write-transport-throughput-ab/`.

| Scale | Control physical rows/s (CV; 95% CI) | Candidate physical rows/s (CV; 95% CI) | Candidate delta |
|---|---:|---:|---:|
| 10k | 0.868135 (8.89%; 0.772338--0.963933) | 1.011184 (13.09%; 0.846871--1.175498) | +16.48% |
| 50k | 0.507188 (1.70%; 0.496489--0.517887) | 0.580209 (5.68%; 0.539275--0.621142) | +14.40% |
| 100k | 0.353847 (4.04%; 0.336102--0.371593) | 0.386153 (2.35%; 0.374878--0.397427) | +9.13% |

As preregistered, 50k is the decision scale and 100k corroborates it. Neither
shows a write-throughput regression. The coder does not claim a throughput
win because the arms are sequential fresh fixtures; the result supports only
that no Task 235 cost was observed. The 10k intervals overlap and cannot
resolve the effect.

The required recall/read-latency/storage side evidence is also present at all
three scales. Recall differs by at most 0.0005, storage by one 8 KiB page, and
all candidate post-insert exact-recall gates pass. Warm read latency is mixed
across fresh fixtures and is not attributed to this write-only change.

The final candidate preflights unanimously report release SHA `b802fe3690b`
with feature `pg18`; its extension source tree is identical to reviewed
candidate `b871d5481`. Earlier invocations that reported control SHA
`387c2137f` were excluded before disposition, and none of their numbers appear
here. The final suite removed every candidate run directory after artifact
capture.

Coder recommendation: **ACCEPT Task 235**. The task remains review-open until
the outside reviewer supplies the final verdict.
