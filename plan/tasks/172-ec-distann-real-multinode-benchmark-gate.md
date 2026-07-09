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
- per-node and summed cluster storage;
- build/load time;
- remote-path engagement counters proving the query used remote expansion and
  remote materialization rather than the single-node degenerate path.

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

## Required Evidence

Packet: `reviews/task-172/001-real-multinode-benchmark/`.

Artifacts:

- suite config checked into the packet;
- `artifacts/suite-manifest.json`;
- `artifacts/results.jsonl`;
- per-scale recall, latency, storage, and load logs;
- node startup logs and roster manifest;
- remote-engagement audit log;
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
| metrics | recall, latency, storage, load/build, remote engagement |

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
