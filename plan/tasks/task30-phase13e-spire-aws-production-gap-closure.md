# Task 30 Phase 13e: SPIRE AWS Production Gap Closure

Status: local functionality gates passed; AWS correctness core passed on Graviton; next AWS proof is representative latency/recall plus pooling A/B before fault rerun
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
- [x] Drive all read matrices through `ecaz bench suite`.

Acceptance:

- [x] Local slow/fast remote fixture proves fast remote is not serialized behind slow remote.
- [x] AWS correctness tier captures selected PIDs, dispatch count, connect/TLS time, candidate time, heap time, payload bytes, merge time, timeout/cancel counts.
- [ ] Representative tier captures p50/p95/p99 latency and recall.

## 13e.4 Evidence-Gated Connection Pooling

- [x] Do not implement pooling before corrected AWS smoke/profile evidence.
- [x] Implement bounded per-backend pooling only if connect/TLS setup is at least 15% of read p95 latency or at least 1 ms p50 per query.
- [x] If triggered, key the pool by node descriptor generation, secret name, remote index identity, TLS mode, user/db, and statement-timeout class.
- [x] Invalidate on descriptor change, auth/TLS failure, schema drift, endpoint identity mismatch, or disconnect.

Acceptance:

- [ ] If the gate is not met, packet records "pooling not justified" with profile rows.
- [ ] If the gate is met, pooled reads reduce connect/TLS count and improve latency without stale-identity reuse.
  Local PG18 gates and packet `998-spire-phase13e-pooling-evidence-local`
  prove pooled reuse reduces follow-up socket opens and that a failed pooled
  remote connection is dropped before post-restart reuse. Reviewer feedback on
  packet `998` confirms the local pooling mechanism evidence is complete; AWS
  representative latency proof remains pending.

## Review And Evidence Rules

- [ ] Each implementation slice gets a code commit and a task-local review packet under `reviews/task-30/`.
- [ ] Test and benchmark logs live under packet-local `artifacts/`.
- [x] AWS proof cannot begin until 13e.1 and 13e.2 pass locally.
- [ ] AWS proof uses the established Graviton/aarch64 lane from Phase 13a/13b
  and the checked-in SPIRE runbook/module. New hardware shapes, regions, or
  setup procedures require an explicit task/runbook amendment before any
  provisioning run.
- [ ] Product-scale claims require accepted AWS correctness, performance, and operations packets.

## Current AWS Evidence Note

- 2026-05-26/27 Graviton correctness reruns reached real remote placement and
  distributed reads: three remote shards, three remote dispatches,
  `remote_heap_candidates`, zero socket opens on warm pooled read profiles, and
  suite output for recall/latency/production read profile. The degraded fault
  drill also returned `degraded_ready` with one skipped stopped remote.
- The latest completed AWS attempt failed during restored-node SQL readiness
  after PostgreSQL was restarted on the remote. The SSM port-forward readiness
  check has since been hardened to wait for Session Manager's explicit opened
  port log line, and fault restore now restarts the operator tunnel on each SQL
  readiness attempt.
- A follow-up AWS rerun after that fix was intentionally interrupted during
  install at operator request. The provisioned Graviton EC2 instances were
  terminated and the local Terraform state no longer lists EC2 instances.
- The next AWS proof should prioritize representative p50/p95/p99 latency,
  recall, and pooled-vs-unpooled production read profile evidence through
  `pass-representative-performance`. Fault rerun/resilience evidence remains
  valuable, but is lower priority than the representative performance and
  pooling packet.
- Local AWS harness hardening packets `1009` through `1026` now make that
  representative pass fail closed before provisioning unless the priority path
  runs the representative preflight, excludes fault reruns, and verifies suite
  plus summary evidence for latency/recall, pooled-vs-unpooled socket reduction,
  p50/p95/p99 latency improvement, zero recall regression, and endpoint identity
  profile counters. Packet `1014` embeds a good/bad summary self-check in the
  preflight so summary-gate regressions are caught locally before AWS resources
  are started. Packets `1016` and `1017` require complete representative sweep
  evidence for the suite-configured top-k=10 nprobe cells and reject priority /
  pooling sweep mismatches. Packet `1019` makes the summary verifier print the
  accepted suite-driven nprobe list, so the AWS packet can show exactly which
  recall/latency sweep was accepted before any fault rerun work resumes. Packet
  `1021` adds a representative `recall@k >= 0.95` floor for the priority and
  pooling suites, and packet `1022` makes pooling A/B dry-run/status output show
  the actual `PGOPTIONS` pool-size settings (`0` versus `16`) in packet-local
  evidence. Packet `1024` adds a local preflight gate requiring the
  representative performance pass to use the AWS teardown watchdog and a
  representative-tier timeout before any EC2 provisioning starts. Packet `1026`
  requires the representative performance pass to run the ordered
  preflight/provision/install/verify chain, and the tunneled verify step to run
  load/register/smoke/priority bench/pooling bench/summarize/verify in order.
