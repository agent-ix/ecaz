---
task: 188
packet: 008-final-finding-disposition
role: coder
status: open
date: 2026-07-27
head: 30febc169
---

# Review request: final disposition of Task 188 findings

This packet records the final disposition of every actionable finding raised
against Task 188 packets 001–007. The packet does not add a new benchmark
matrix: the accepted batch-10 A/B, paired recall rows, efficient stage-counter
diagnostic, and runner equivalence evidence already exist in packet-local
artifacts and are cited below.

## Disposition

| finding | disposition | evidence / change |
| --- | --- | --- |
| P1-1: missing frontier, reachability, and graph-quality attribution | closed as an explicit scope correction | Task 188 now states that Phase 1 was a search-budget screen only. The three attribution families remain unrun and unselected; they are not presented as refuted or irreducible. A new task is required if those audits are reopened. |
| P1-2: selection rule not applied | closed | The corrected explicit-batch-10 matrix applies the rule: BW8 has positive paired recall deltas at 50k/100k, no control wins, lower warm mean/p95 at all scales, zero storage delta, and shared builds. The eager-0 50k regression is not acceptance evidence. |
| P1-3: unpaired recall comparison | closed | Packet 005 `results.jsonl` contains the three `physical_benchmark_paired_recall` rows: 0/0/200, 5/0/195, and 7/0/193, with positive bootstrap intervals at 50k/100k. |
| P1-4: materialization regime mismatch | closed | Packet 005 reran BW4/BW8 at explicit batch-10; omitted variant fields now inherit lazy-10. |
| P2-1: candidate paired with a non-promotion head | closed as scope clarification | BW8 is paired only with the exact-scored 16,384 training-landmark head and is accepted only as an isolated research candidate. Task 188 changes no production head or default. |
| P2-2: instrumented/uninstrumented latency cross-citation | closed | Packet 004 and packet 006 label packet-002 eager-0 stage rows separately from packet-005 batch-10 latency and packet-006 p50/stage evidence. |
| P2-3: unmerged Task 186 entry evidence | closed | Packet 001 now states that the cited Task 186 branch was unmerged and that its hierarchy result was a query-time/arbitrary-representative prototype, not a production routing conclusion. |
| M1: worker default equivalence | closed | Packet 007 contains the pre-refactor/current PG18 equivalence run and reports matching latency within noise. |
| M2: re-warm after reconnect | closed | The shared worker now warms every fresh client batch. |
| M3: emit worker batch provenance | closed | Latency and physical result rows emit `worker_batch_size`; suite expansion exposes the option. |
| M4: qualify efficient diagnostic tails | closed | Packet 006 cites p50 only for the reconnect-contaminated diagnostic distribution and separates direct stage attribution. |
| F1: backend memory growth | moved out | Reviewer disposition created Task 200; it is not caused by Task 188’s zero-extension-source harness changes and is not a Task 188 merge gate. |
| F2: remote-candidate explanation | closed | Packet 006 promotes the 25.86/29.56 eager-0 versus 6.64/6.62 batch-10 transition and explains batching/deduplication semantics. |
| F3: suite option reachability | closed | `worker_batch_size` is reachable from suite latency steps; the distann physical path retains its own `benchmark_backend_batch_size`. |

The resulting decision remains: accept BW8 as the sole isolated search-budget
research candidate, make no production change in Task 188, and do not claim
that the unrun graph-quality families were eliminated.

See [finding disposition](artifacts/finding-disposition.md) and
[manifest](artifacts/manifest.md) for the packet-local evidence map.
