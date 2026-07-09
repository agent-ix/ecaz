# Task 172: ec_distann Real Multi-Instance Benchmark Gate

Status: proposed (2026-07-09). Depends on: Tasks 165 and 166.
Owner: coder (to be assigned). One branch off the current ec_distann line.
Priority: P0 corrective benchmark gate.

## Why

Task 166 produced useful **single-instance** `ec_distann` benchmark evidence, but
it did not exercise the distributed DistANN path: no `ec_distann.roster`, no
remote `ec_distann_expand_nodes`, no remote row materialization, no multi-process
latency, and no cluster-level storage summation.

For a multi-instance AM, single-instance numbers are not a product gate. We need
a benchmark that proves the actual distributed path on real PostgreSQL
instances, with the same evidence quality expected of other index gates.

## Goal

Produce a release-build `ecaz bench suite` benchmark packet for the real
**local multi-instance** `ec_distann` path at 10k / 50k / 100k, measuring:

- recall@10;
- p50 / p95 / p99 latency;
- throughput under concurrency;
- per-node and summed cluster storage;
- build/load time;
- remote-path engagement counters proving the query used remote expansion and
  remote materialization rather than the single-node degenerate path;
- distributed-process telemetry detailed enough to debug, optimize, and model
  larger deployments.

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
- Build the benchmark surface under the current honest distribution model:
  - replicated-global graph with serving ownership is acceptable only as a
    distinct control lane and must be labeled as such;
  - disjoint-shard storage must be measured if the current branch claims it is
    implemented; otherwise the packet must explicitly mark disjoint storage as a
    blocker and not claim a distributed storage gate.
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

## Distributed Telemetry Requirements

The benchmark must capture per-scale and per-sweep aggregate metrics for the
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
- per-scale recall, latency, storage, and load logs;
- throughput/concurrency logs;
- node startup logs and roster manifest;
- remote-engagement audit log;
- distributed telemetry JSONL / trace artifact;
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
| distribution lane | replicated-serving control, disjoint-shard if claimed |
| query mode | distributed roster active |
| sweep | `ec_distann.top_k` default sweep `[16,32,64,100,200]` unless changed by profile registry |
| concurrency | at least 1, 2, 4, 8, 16 for throughput unless resource-limited |
| metrics | recall, latency, throughput, storage, load/build, remote engagement, distributed telemetry |

## Acceptance Criteria

1. Distributed recall is measured at 10k / 50k / 100k and compared against the
   same-commit single-instance `ec_distann` control. Any point below
   `single_instance_recall - 0.001` is a blocker unless the packet explicitly
   records a no-promote verdict.
2. Distributed latency is measured at every scale and sweep. The packet reports
   the overhead ratio versus single-instance `ec_distann` and versus the Task 166
   comparator AMs where comparable.
3. Storage is reported as cluster-total bytes, not just coordinator-local index
   bytes. If replicated storage is used, the 3x replication cost must be shown.
4. The packet proves the remote path was used. A run with an empty roster,
   missing remote engagement counters, or only local AM scans is invalid.
5. The final verdict reclassifies Task 166 correctly: Task 166 remains
   single-instance evidence; Task 172 is the distributed benchmark gate.
6. The local multi-instance fixture is reusable from `ecaz bench suite` without
   packet-specific glue, so future distann distributed benchmark packets can
   invoke the same suite step/config surface.
7. Distributed telemetry is rich enough to attribute latency to coordinator,
   remote expansion, remote materialization, connection/session setup, and
   merge/dedup work. A packet with only aggregate recall/latency/storage numbers
   is incomplete.
8. The verdict includes a measured throughput curve and a stated scaling model
   for 1m and 10m rows, or explicitly records why the current telemetry cannot
   support such an estimate.

## Non-Goals

- New ANN algorithm changes.
- Incremental insert performance; Task 167 owns inserted-row parity and DML
  behavior.
- Cloud deployment. Local multi-instance is the target for this corrective gate;
  cloud/network RTT sensitivity can be a follow-up only after local evidence
  exists.

## References

- Task 165: real multi-instance functional fixture and disjoint-shard drill.
- Task 166: single-instance `ec_distann` benchmark gate, now a control lane.
- NFR-017: DistANN latency/recall gate.
- NFR-018: DistANN space amplification.
- FR-081: query orchestration.
- FR-082: epoch lifecycle and consistency.
