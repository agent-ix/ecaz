# Task 172: ec_distann Real Multi-Instance Benchmark Gate

Status: **COMPLETE — final gate merged by admin via PR #72** (2026-08-08;
merge commit `97b17e77f0c611464d5cfbfaa6c219ba0c2a4200`). Packet
`reviews/task-172/011-final-gate/` records the decision-bearing 10k/50k/100k
matrix and final verdict. Packets 008–010 are review-closed ACCEPT
(`reviews/task-172/008-status-activation/feedback/2026-08-08-01-reviewer.md`,
`reviews/task-172/009-nfr021-prerequisite-integrity/feedback/2026-08-08-01-reviewer.md`,
and `reviews/task-172/010-macos-release-cli/feedback/2026-08-08-01-reviewer.md`).
Task 204's
measurement proof, Task 205's corrected bounded-L A/B, Task 206's traversal
regime disposition, and Task 208's mechanical NFR-021/NFR-022 gates are now
review-closed prerequisites. The final matrix passed the cross-scale NFR-021
check and showed no recall collapse; the local distributed fixture did not
demonstrate a performance win over the single-instance control. Task 219 owns
the separate Pareto-recall decision. Outside review was bypassed by the
explicit admin-merge request for PR #72.
Owner: coder. One branch off the current ec_distann line.
Priority: P0 corrective benchmark gate.

## SHELVE NOTE (2026-07-10)

Task 172 is shelved and we are shifting to option (a): **build the FR-078 real
sharded build/publish path first, then benchmark.** Rationale:

- The only available multi-instance surface (`ecaz dev distann-multicluster
  local-multinode-pg18`) builds the COMPLETE global Vamana graph independently
  on every node and partitions only serving ownership; the "disjoint" drill
  deletes non-owned heap rows and tombstones (not removes) the replicated index.
  It is a **replicated-serving control**, not a sharded index.
- Every gate quantity is therefore an artifact of replication, not the design:
  storage is ~3× inflated (measured 5.65× vs an expected ~1.9× sharded, so the
  replicated model fails NFR-018 while the real design likely passes); recall
  identity is real but does not exercise cross-shard hop-round traversal; and
  distributed latency reflects local full-graph search + eager remote
  materialization, not per-hop network cost — the reason the latency sweep timed
  out repeatedly (900s/700s/600s) on this fixture.
- Benching the replicated fixture cannot answer NFR-017/018/019. Physical
  FR-078 placement is a fail-closed prerequisite for all gate measurements.

**Retained as valid evidence (functional, not gate):** the read-path,
fanout/merge, 12-drill NFR-020 fault matrix, recall-oracle, and single-vs-multi
identity results in packet 001 stand as replicated-serving-control evidence and
must never be promoted as distributed latency/storage/scaling numbers.

## Why

Task 166 produced useful **single-instance** `ec_distann` benchmark evidence, but
it did not exercise the distributed DistANN path: no `ec_distann.roster`, no
remote `ec_distann_expand_nodes`, no remote row materialization, no multi-process
latency, and no cluster-level storage summation.

The first Task 172 preflight also did not exercise the intended physical storage
topology. It built a complete graph on every node and partitioned only serving
ownership; its destructive "disjoint" drill then deleted non-owned heap rows
while leaving the replicated graph records tombstoned in each index. That is a
useful transport/fanout control, but it is **not a sharded index** and none of its
latency, throughput, storage, or scaling rows may satisfy this gate.

For a multi-instance AM, single-instance numbers are not a product gate. We need
a benchmark that proves the actual distributed path on real PostgreSQL
instances, with the same evidence quality expected of other index gates.

## Goal

Produce a release-build `ecaz bench suite` benchmark packet for the real
**physically hash-sharded local multi-instance** `ec_distann` path at 10k / 50k
/ 100k. `ec_distann` remains one logical global Vamana graph; "sharded" here
means that each graph record and its co-placed full-precision vector are stored
on exactly one FR-078 hash owner, not that each node builds an independent ANN
graph. Measure:

- recall@10;
- p50 / p95 / p99 latency;
- throughput under concurrency;
- per-node and summed cluster storage;
- build/load time;
- remote-path engagement counters proving the query used remote expansion and
  remote materialization rather than the single-node degenerate path;
- distributed-process telemetry detailed enough to debug, optimize, and model
  larger deployments.

The suite must support two execution modes:

- **benchmark mode**: low-overhead gate mode for recall, latency, throughput,
  storage, and load/build. It captures only cheap counters needed to prove the
  distributed path was engaged and to label the run.
- **full metrics mode**: instrumentation-heavy diagnostic mode for attribution,
  debugging, and scaling estimates. Its latency numbers are diagnostic and must
  not be used as the primary product latency gate unless the packet also proves
  instrumentation overhead is negligible.

The result must explicitly compare distributed `ec_distann` against the
single-instance `ec_distann` control on the same corpus/query/commit, and may
reuse Task 166 comparator AMs only if the commit, corpus, host, and protocol are
unchanged or re-run under this packet.

## Scope

This task is explicitly **local first**: multiple PostgreSQL instances on the
same benchmark host, separate data directories/sockets/ports, no cloud
dependency. The point is to measure the actual distributed executor/transport
path before adding network/instance-placement variables.

- Extend `ecaz bench suite` as needed. This must become a first-class ecaz CLI
  benchmark-suite capability, not a packet-local script.
- Launch and manage a 3-PG18-instance local fixture from the suite runner: one
  coordinator plus two data nodes, separate data directories/sockets/ports,
  release extension installed on every node.
- Load the staged real DBpedia 10k / 50k / 100k corpora and queries.
- Build and publish the benchmark surface in the FR-078 physical placement
  topology:
  - construct/stitch one coherent global graph, then hand off each graph record
    and its full-precision vector to exactly one `hash(vec_id) mod roster` owner;
  - each serving node stores only its owned graph records and co-placed vector
    rows; the coordinator stores only routing/head/epoch metadata plus an owned
    shard if it is itself in the serving roster;
  - no serving node may retain a complete graph replica, and build-then-delete /
    tombstone pruning of a replicated index does not qualify as physical
    sharding;
  - a replicated-global graph with serving-ownership filtering may be retained
    only as a separately named control lane. It cannot satisfy any Task 172 gate
    row or the NFR-018 distributed-storage result.
- Run a fail-closed topology audit before any benchmark measurement. The suite
  must prove from the serving relations/index directories that:
  - every corpus vec_id has exactly one physical graph record across the roster;
  - pairwise graph-record ownership intersections are empty and the union equals
    the corpus vec_id set;
  - every record is on its FR-078 hash owner with its full-precision vector
    co-placed on the same node;
  - no node retains non-owned live or tombstoned graph records from a full
    replica; and
  - per-node owned-record counts and physical bytes are emitted as structured
    result rows (including FR-078's 100k balance check).
  Missing audit fields or any failed invariant invalidates all downstream
  recall, latency, throughput, storage, and scaling rows for that run.
- Run recall and latency with `ec_distann.roster`, `ec_distann.local_node_id`,
  and `ec_distann.epoch` set so the coordinator uses the distributed
  CustomScan/remote transport path.
- Sum storage across all participant nodes. Report both:
  - coordinator-visible table/index bytes; and
  - cluster-total bytes across coordinator + data nodes.
- Emit a remote-engagement audit per scale/sweep: remote expand calls, remote
  materialize calls, remote-owned output rows or equivalent counters/log lines.
- Emit structured distributed telemetry, not only pass/fail logs. The suite
  runner should collect these as normalized JSONL rows and packet-local logs so
  follow-up optimization work can attribute costs without re-running the packet.
- Keep full telemetry optional at invocation time. The default gate run should
  be benchmark mode; full metrics mode should be enabled by an explicit suite
  option/config field and write separate artifacts/result rows.

## Distributed Telemetry Requirements

Full metrics mode must capture per-scale and per-sweep aggregate metrics for the
distributed process, and either full per-query rows or a documented sampled
per-query trace. At minimum:

- query shape: scale, top_k sweep, k, query count, concurrency, cache state;
- orchestration: hop rounds executed, owners touched, local vs remote candidate
  counts, early-exit count, restart/retry count;
- remote expansion: number of `ec_distann_expand_nodes` calls, vec_ids requested
  and returned, neighbor rows returned, remote SQL time, coordinator wait time,
  serialization/deserialization bytes if available;
- remote materialization: number of materialize calls, remote-owned output rows,
  payload bytes, missing/tombstone rows, remote SQL time, coordinator decode time;
- connection behavior: connect count, pooled connection reuse, session-GUC setup
  count/time, connection failures/timeouts;
- coordinator work: merge/dedup time, exact rerank count, final rows produced,
  executor/CustomScan time where measurable;
- node work: per-node rows owned, rows expanded, rows materialized, CPU/rss/IO
  samples if the local harness can collect them cheaply;
- storage: heap/index/table bytes per node, cluster-total bytes, replicated vs
  disjoint-shard accounting explicitly separated.

These metrics should be emitted by `ecaz bench suite` as first-class result rows
or linked packet-local JSONL artifacts. Grepping PostgreSQL logs manually is not
acceptable as the primary telemetry path.

Benchmark mode should still emit enough low-overhead fields to validate that it
used the distributed path: roster hash/spec label, non-empty remote owner count,
remote expand call count, remote materialize call count, and remote-owned result
row count. If those fields are zero or absent, the latency/recall run is invalid.

The packet must include a small overhead audit: run one representative scale and
sweep point in both benchmark mode and full metrics mode, then report the
instrumentation overhead. This audit is for interpreting diagnostics; it must
not replace the lean benchmark-mode gate rows.

## Throughput and Scaling Analysis

The benchmark must include at least one throughput sweep per scale with the
distributed roster active, using concurrency levels sufficient to expose the
local saturation curve, for example `1, 2, 4, 8, 16` unless the runner records a
clear resource limit. Report:

- QPS, p50, p95, p99, max latency per concurrency level;
- per-node utilization or the closest available CPU/rss/IO proxy;
- remote calls/query and remote bytes/query at each concurrency level;
- the first observed bottleneck: coordinator CPU, data-node CPU, connection
  pool, IPC/socket latency, heap/vector materialization, or storage IO.

The final verdict must include a capacity-planning section. It should use the
measured telemetry to estimate latency and throughput behavior beyond 100k, at
least for 1m and 10m rows, with assumptions stated explicitly. The model can be
simple, but it must be grounded in measured terms: per-query hop count, remote
calls, remote bytes, exact-rerank/materialization work, storage bytes/row, and
observed QPS saturation. If the data is insufficient for a credible estimate,
the verdict must say that and identify the missing measurement.

## Required Evidence

Packet: `reviews/task-172/001-real-multinode-benchmark/`.

Artifacts:

- suite config checked into the packet;
- `artifacts/suite-manifest.json`;
- `artifacts/results.jsonl`;
- topology/placement audit log and normalized JSONL rows proving physical
  hash-shard disjointness, exact corpus coverage, record/vector co-placement,
  and absence of full-index replicas or tombstoned non-owner residue;
- per-scale recall, latency, storage, and load logs;
- throughput/concurrency logs;
- node startup logs and roster manifest;
- benchmark-mode remote-engagement audit log;
- full-metrics-mode distributed telemetry JSONL / trace artifact;
- instrumentation overhead audit;
- larger-scale capacity estimate markdown or verdict section;
- storage summation log with per-node and total bytes;
- final verdict markdown.

All commands must be driven by `ecaz bench suite` and recorded in the packet
manifest. If a necessary suite step does not exist yet, add it to `ecaz-cli`
first and then use it from the packet config.

Packet-local shell sweepers, one-off SQL runners, copied fragments from
`distann_multicluster`, or manual setup transcripts are invalid evidence for
this task. They may be used only during development debugging; the committed
benchmark packet must use the suite runner.

## Minimum Matrix

| Axis | Values |
|------|--------|
| scale | 10k, 50k, 100k |
| nodes | 3 PG18 instances |
| distribution lane | physically hash-sharded global graph (required); replicated-serving control (optional, non-gate) |
| query mode | distributed roster active |
| sweep | `ec_distann.top_k` default sweep `[16,32,64,100,200]` unless changed by profile registry |
| concurrency | at least 1, 2, 4, 8, 16 for throughput unless resource-limited |
| modes | benchmark mode for gate rows; full metrics mode for attribution rows |
| metrics | recall, latency, throughput, storage, load/build, remote engagement, distributed telemetry |

## Acceptance Criteria

1. A suite-driven topology preflight proves the FR-078 physical hash-shard
   invariants before measurements run: exactly one graph record and co-placed
   vector per vec_id across the roster, exact corpus coverage, empty pairwise
   ownership intersections, correct hash owner, no non-owner record residue,
   and the 100k balance check. A replicated index filtered by serving ownership,
   including a build-then-delete/tombstone variant, fails this criterion.
2. Distributed recall is measured at 10k / 50k / 100k and compared against the
   same-commit single-instance `ec_distann` control. Any point below
   `single_instance_recall - 0.001` is a blocker unless the packet explicitly
   records a no-promote verdict.
3. Distributed latency is measured at every scale and sweep. The packet reports
   the overhead ratio versus single-instance `ec_distann` and versus the Task 166
   comparator AMs where comparable.
4. Storage is reported as cluster-total bytes for the physically sharded graph,
   not just coordinator-local index bytes. Any optional replicated control must
   report its replication cost separately and cannot supply the NFR-018 verdict.
5. The packet proves the remote path was used. A run with an empty roster,
   missing remote engagement counters, or only local AM scans is invalid.
6. The final verdict reclassifies Task 166 correctly: Task 166 remains
   single-instance evidence; Task 172 is the distributed benchmark gate.
7. The local multi-instance fixture is reusable from `ecaz bench suite` without
   packet-specific glue, so future distann distributed benchmark packets can
   invoke the same suite step/config surface.
8. Distributed telemetry is rich enough to attribute latency to coordinator,
   remote expansion, remote materialization, connection/session setup, and
   merge/dedup work. A packet with only aggregate recall/latency/storage numbers
   is incomplete.
9. The verdict includes a measured throughput curve and a stated scaling model
   for 1m and 10m rows, or explicitly records why the current telemetry cannot
   support such an estimate.
10. Primary recall/latency/throughput verdicts are taken from benchmark mode, not
   full metrics mode. Full metrics mode is separately labeled, its overhead is
   measured, and its rows are used for attribution and modeling.

## Non-Goals

- New ANN algorithm changes.
- Incremental insert performance; Task 167 owns inserted-row parity and DML
  behavior.
- Cloud deployment. Local multi-instance is the target for this corrective gate;
  cloud/network RTT sensitivity can be a follow-up only after local evidence
  exists.

## References

- Task 165: real multi-instance replicated-control fixture and destructive
  delete/tombstone "disjoint" drill (functional control only; non-qualifying
  topology).
- Task 166: single-instance `ec_distann` benchmark gate, now a control lane.
- Task 179: physical owner generations, streamed handoff, publication, frozen
  row tier, and topology preflight; the hard implementation prerequisite.
- NFR-017: DistANN latency/recall gate.
- NFR-018: DistANN space amplification.
- FR-081: query orchestration.
- FR-082: epoch lifecycle and consistency.
