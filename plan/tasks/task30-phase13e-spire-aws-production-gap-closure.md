# Task 30 Phase 13e: SPIRE AWS Production Gap Closure

Status: local functionality gates passed; AWS correctness core passed on Graviton; representative AWS latency/recall, suite-gated pooling A/B, and operations fault-restore evidence complete; final closeout review requested
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
- [x] Representative tier captures p50/p95/p99 latency and recall.

## 13e.4 Evidence-Gated Connection Pooling

- [x] Do not implement pooling before corrected AWS smoke/profile evidence.
- [x] Implement bounded per-backend pooling only if connect/TLS setup is at least 15% of read p95 latency or at least 1 ms p50 per query.
- [x] If triggered, key the pool by node descriptor generation, secret name, remote index identity, TLS mode, user/db, and statement-timeout class.
- [x] Invalidate on descriptor change, auth/TLS failure, schema drift, endpoint identity mismatch, or disconnect.

Acceptance:

- [x] If the gate is not met, packet records "pooling not justified" with profile rows.
  Not applicable: the gate was met by AWS profile evidence, so the pooling
  implementation and payoff evidence path below applies.
- [x] If the gate is met, pooled reads reduce connect/TLS count and improve latency without stale-identity reuse.
  Local PG18 gates and packet `998-spire-phase13e-pooling-evidence-local`
  prove pooled reuse reduces follow-up socket opens and that a failed pooled
  remote connection is dropped before post-restart reuse. Reviewer feedback on
  packet `998` confirms the local pooling mechanism evidence is complete; AWS
  packet `1063` confirms on the preserved Graviton cluster that disabling the
  pool opens one socket per dispatch with `connect_p50` around `19-20 ms`,
  while the default pool opens zero sockets and reduces q=20 production-read
  `total_p50` by `9-11 ms`. Packet `1065` completes the suite-gated q=1000
  representative pooling A/B: socket opens drop from `3000` to `0`,
  connect p95 drops from `19 ms` to `0 ms`, production total p95 improves from
  `59 ms` to `49 ms`, coordinator latency p95 improves from `120.175 ms` to
  `107.893 ms`, and recall delta remains `0`.

## Review And Evidence Rules

- [x] Each implementation slice gets a code commit and a task-local review packet under `reviews/task-30/`.
  Phase 13e implementation, local validation, AWS correctness, representative
  performance/pooling, operations, and final closeout evidence are split across
  task-local packets `957` through `1067` and pushed on
  `diskann-aws-optimization`.
- [x] Test and benchmark logs live under packet-local `artifacts/`.
- [x] AWS proof cannot begin until 13e.1 and 13e.2 pass locally.
- [x] AWS proof uses the established Graviton/aarch64 lane from Phase 13a/13b
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
- Packet `1028` reran the representative performance preflight at current head
  and verified zero non-terminated EC2 instances in `us-west-2`. Packet `1029`
  verified the active `terraform.tfvars` still targets the established
  Graviton lane (`us-west-2`, `us-west-2a`, `m7g.large`, three remotes), local
  Terraform state has no managed resources, and the combined preflight passes
  only when the documented pre-existing S3 bucket residue override is enabled.
  Without `SPIRE_AWS_ALLOW_PREEXISTING_RESIDUE=1`, permission preflight still
  fails on old `ecaz-spire-aws-*` buckets because the operator identity lacks
  `s3:ListBucketVersions`; the next AWS run must either use that reviewed
  residue exception with packet-local evidence or run after the permission /
  residue issue is resolved.
- Packet `1030` adds an operator preflight guard requiring `auto_stop_at` to be
  at least `18000` seconds after preflight time, so the representative run
  cannot start with less than the four-hour watchdog budget plus buffer. The
  active Graviton `terraform.tfvars` passes this guard as of the packet run.
- Packet `1031` adds `scripts/spire-aws/run-representative-performance-pass.sh`
  as the standard dry-run-by-default entrypoint for the remaining AWS proof. It
  requires a task-local `ARTIFACT_DIR`, runs the current preflight stack with
  the reviewed residue exception by default, prints the exact
  `pass-representative-performance` command, and only provisions when rerun with
  `--execute` after explicit AWS approval.
- Packet `1032` wires that dry-run entrypoint into
  `make -C infra/spire-aws plan-representative-performance` and makes the
  representative preflight require the entrypoint to stay executable.
- Packet `1033` makes the representative execute entrypoint refuse to reuse an
  artifact directory that already contains representative topology, suite, or
  summary output unless the operator passes `--reuse-artifact-dir` explicitly.
- Packet `1034` moves the same artifact-directory guard into
  `preflight-representative-performance`, so direct Make execution also fails
  before provisioning when `ARTIFACT_DIR` is the legacy default or already
  contains representative output.
- Packet `1035` adds a direct-Make start marker between representative
  preflight and provisioning, so an interrupted `pass-representative-performance`
  run reserves its packet before EC2 resources can be created.
- Packet `1063` captures a targeted q=20 pooled-vs-unpooled
  production-read-profile comparison on the preserved packet 1062 Graviton
  cluster: `socket_open_sum` drops from `53/60/60/60` to `0/0/0/0`,
  `connect_p50` drops from `19-20 ms` to `0 ms`, and `total_p50` improves by
  `9-11 ms` across nprobe `8,16,24,32` with identical recall rows. This
  answers the immediate pooling-payoff question but does not replace the full
  q=1000 representative suite acceptance evidence.
- Packet `1064` preserves an interrupted start of the full representative
  suite. Smoke checks still proved `EcSpireDistributedScan` and
  `remote_heap_candidates`, but the operator requested nightly shutdown while
  the suite was entering `13a3a-recall-k10`; all selected suite steps remained
  pending. All `us-west-2` EC2 instances were stopped and no SSM tunnel or
  benchmark process remained. The next AWS work is to deliberately restart the
  stopped Graviton instances and complete `bench-representative-priority`,
  `bench-representative-pooling`, `summarize-representative-performance`, and
  `verify-representative-performance-summary`.
- Packet `1065` completes that restart against the preserved packet 1062
  Graviton cluster without provisioning, reinstalling, reloading data, or
  rebuilding topology. The representative suite captured p50/p95/p99 latency
  for nprobe `8,16,24,32`, recall for nprobe `8,16,24,32,64`, q=1000
  production read profiles for k=10 and k=100 at nprobe `64`, and q=1000
  pooled-vs-unpooled A/B evidence at nprobe `64`. The verifier accepted
  `latency:8 16 24 32 recall:8 16 24 32 64 production:64 pooling:64`.
  Representative k=10 recall reaches `0.9573` at nprobe `64`; production
  k=10 reports coordinator p50/p95/p99 `99.298/107.870/117.529 ms`,
  `remote_heap_candidates`, `dispatch_sum=3000`, `socket_open_sum=0`,
  `total_p50=46 ms`, `total_p95=49 ms`, and zero timeout/cancel/degraded
  skips. The pooling suite shows disabled-vs-enabled socket opens `3000 -> 0`,
  connect p95 `19 ms -> 0 ms`, production total p95 `59 ms -> 49 ms`, and
  recall delta `0`. After the run, all `us-west-2` EC2 instances were stopped
  and a packet-local no-pending/running/stopping check was captured.
- Packet `1066` completes the operations fault-restore rerun against the same
  preserved packet 1062 Graviton cluster without provisioning, reinstalling,
  reloading data, or rebuilding topology. A first attempt is preserved in the
  packet and failed on a harness-only representative query-vector assumption;
  commit `6f11d0c8a` fixed the reusable fault harness to select the first
  available query row with `ORDER BY id LIMIT 1`. The successful rerun shows
  degraded mode returning remote heap candidates from the two available
  remotes with `status=degraded_ready`, `returned_candidate_count=10`, and
  `degraded_skipped_dispatch_count=1`; strict mode fails closed when node 2 is
  stopped; both restore paths restart PostgreSQL and reach SQL readiness after
  one attempt; final post-restore smoke returns to strict 3-remote
  `EcSpireDistributedScan` / `remote_heap_candidates` with zero timeout,
  cancel, or degraded-skip counters. All `us-west-2` instances were then
  stopped and verified with no pending/running/stopping instance rows.
- Packet `1067` is the final closeout review request. It does not introduce a
  new AWS run; it maps the task-file requirements to the accepted-or-submitted
  evidence packets for correctness, performance/pooling, operations, and AWS
  cost-safety verification. The final product-scale claim remains pending
  outside reviewer acceptance of that closeout request.
