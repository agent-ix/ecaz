# Task 30 Phase 13e: SPIRE AWS Production Gap Closure

Status: ready to implement
Owner: coder1 / SPIRE AWS production track
Priority: P0 before any AWS product-scale claim

## Goal

Implement the missing production capabilities required for AWS-scale SPIRE proof:
real remote shard placement, real distributed CustomScan queries over remote
data, measured parallel fanout, and evidence-gated connection pooling.

The target is a fixed coordinator plus N remote topology with operator-driven
static sharding. Online rebalancing, background repair, and coordinator HA are
out of scope for this phase.

## 13e.1 Static Remote Placement And Distributed Load

- [x] Add a distributed placement config consumed by `ecaz corpus load --profile ec_spire`.
- [x] Load each shard onto its owning remote, build and publish the remote SPIRE index, and register the remote descriptor on the coordinator.
- [x] Publish coordinator placement-directory entries with remote `node_id` values through production code, not test-only placement rewrites.
- [x] Add a local PG18 1-coordinator plus 3-remote fixture proving remote placements exist after load.

Acceptance:

- [x] Coordinator placement snapshot shows remote placements by node.
- [x] No AWS/local production fixture calls `tests.ec_spire_test_rewrite_placement_node`.
- [x] Empty or local-only placement directories fail the distributed smoke gate.

## 13e.2 Distributed CustomScan Read Path

- [x] Use `EcSpireDistributedScan` as the production AWS read path.
- [x] Select local and remote PIDs from the coordinator placement directory.
- [x] Fan out to remote nodes, receive candidate plus heap tuple payload rows, merge globally by score, and return visible SQL tuples.
- [x] Preserve strict/degraded semantics and expose skipped/failure counters.

Acceptance:

- [x] `EXPLAIN` contains `Custom Scan (EcSpireDistributedScan)`.
- [x] Top-k results include remote-owned rows and match exact baseline recall thresholds.
- [x] Strict remote failure returns no partial rows.
- [x] Degraded remote failure returns partial rows with skip diagnostics.

## 13e.3 Parallel Fanout And Performance Evidence

- [x] Keep the async production transport as the hot read path; diagnostic SQL libpq helpers are not performance evidence.
- [x] Prove candidate and heap receive overlap across remotes.
- [x] Add production read profile capture to every local and AWS suite row.
- [ ] Drive all read matrices through `ecaz bench suite`.

Acceptance:

- [x] Local slow/fast remote fixture proves fast remote is not serialized behind slow remote.
- [ ] AWS correctness tier captures selected PIDs, dispatch count, connect/TLS time, candidate time, heap time, payload bytes, merge time, timeout/cancel counts.
- [ ] Representative tier captures p50/p95/p99 latency and recall.

## 13e.4 Evidence-Gated Connection Pooling

- [ ] Do not implement pooling before corrected AWS smoke/profile evidence.
- [ ] Implement bounded per-backend pooling only if connect/TLS setup is at least 15% of read p95 latency or at least 1 ms p50 per query.
- [ ] If triggered, key the pool by node descriptor generation, secret name, remote index identity, TLS mode, user/db, and statement-timeout class.
- [ ] Invalidate on descriptor change, auth/TLS failure, schema drift, endpoint identity mismatch, or disconnect.

Acceptance:

- [ ] If the gate is not met, packet records "pooling not justified" with profile rows.
- [ ] If the gate is met, pooled reads reduce connect/TLS count and improve latency without stale-identity reuse.

## Review And Evidence Rules

- [ ] Each implementation slice gets a code commit and a task-local review packet under `reviews/task-30/`.
- [ ] Test and benchmark logs live under packet-local `artifacts/`.
- [ ] AWS proof cannot begin until 13e.1 and 13e.2 pass locally.
- [ ] Product-scale claims require accepted AWS correctness, performance, and operations packets.
