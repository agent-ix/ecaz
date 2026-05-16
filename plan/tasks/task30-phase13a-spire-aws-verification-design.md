# Task 30 Phase 13a: SPIRE AWS Verification Design

Status: design draft, pending reviewer acceptance
Owner: coder1 / SPIRE AWS verification track
Priority: P1 — blocks Phase 13b runbook finalization and any Phase 13
infrastructure allocation.

## Goal

Convert the entry/exit-gate file `task30-phase13-spire-aws-verification.md`
into a complete, reviewer-acceptable design manifest covering topology,
datasets, workload matrix, pass/fail thresholds, observability surface,
fault drills, reporting layout, and cost guardrails. Phase 13a is the
*decision* phase; Phase 13b turns those decisions into an operator
runbook + helper scripts; the parent Phase 13 gate file is the entry/exit
contract that both sub-phases roll up to.

Phase 13a is not infrastructure work. No AWS resources are provisioned by
Phase 13a; the only deliverables are this design file and any Phase 13.0
counter-prerequisite packets it identifies.

## Entry State

- Parent `task30-phase13-spire-aws-verification.md` exists with Phase 12 and
  Phase 12c.4 entry-gate boxes marked complete or reviewer-deferred.
- Phase 12c packet `763` documents the READ schema-drift disposition.
- Local readiness boundaries are defined in
  `docs/SPIRE_LOCAL_READINESS.md` and
  `docs/SPIRE_LOCAL_CAPACITY_TARGETS.md`.
- Local operator coverage exists in `docs/SPIRE_LIBPQ_RUNBOOK.md` and
  `docs/SPIRE_DIAGNOSTICS.md`.
- The SPIRE counter and GUC surface inventoried in Phase 13a.5 is the
  surface present on the head SHA captured by the Phase 13a packet.

## Non-Goals

- Implementing missing counters that Phase 13 evidence requires. Those are
  Phase 13.0 prerequisites (Phase 13a.5) and must each land as their own
  packet before Phase 13b execution begins; if any prerequisite is
  reviewer-deferred, Phase 13a records the deferral and Phase 13b carries
  the operator-impact note.
- Multi-coordinator HA, cross-shard non-vector query execution, DDL
  propagation, or cross-shard embedding UPDATE moves. These remain ADR
  scope per Phase 11 / 12 non-goals.
- Billion-scale product claims. Phase 13 evidence is bounded by the
  dataset identities listed in Phase 13a.2.
- A second cloud (GCP, Azure). Phase 13 is AWS-only; any other cloud is a
  separate phase with its own packet.
- Authoring the operator runbook. Procedural content lives in Phase 13b.

## Phase 13a.1: Topology (P1)

Document and obtain reviewer acceptance for every topology decision below.

- [ ] **PostgreSQL build path.** Decision: self-managed PostgreSQL 18 on
  EC2. Rationale: the ecaz extension loads a custom `.so` and registers a
  custom access method and CustomScan node; Amazon RDS and Aurora do not
  permit either. The Phase 13 evidence cites "AWS/EC2 self-managed PG18",
  not "AWS/RDS"; the parent gate file's "AWS/RDS-class verification"
  wording will be amended at Phase 13 closeout.
- [ ] **Coordinator node.** Decision: `r6i.4xlarge` (16 vCPU, 128 GiB
  RAM). Storage: 200 GiB gp3, 3000 IOPS, 125 MiB/s. OS: Amazon Linux
  2023. PG settings: `max_prepared_transactions >= 64`,
  `shared_buffers = 32 GiB`, `work_mem = 64 MiB`,
  `maintenance_work_mem = 2 GiB`. The `max_prepared_transactions` value is
  load-bearing per `docs/SPIRE_LIBPQ_RUNBOOK.md`; if it is not set the
  coordinator INSERT path fails closed.
- [ ] **Remote nodes.** Decision: 3 remotes, each `r6i.2xlarge`
  (8 vCPU, 64 GiB), 100 GiB gp3. Rationale: exercises governance caps
  `ec_spire.remote_search_max_nodes` and
  `ec_spire.remote_search_max_pids_per_node` with non-minimal fanout; the
  1-remote and 2-remote shapes are already covered by Phase 12
  multicluster fixtures.
- [ ] **Placement.** Decision: one VPC, one private subnet, one AZ for
  baseline. Single-AZ removes cross-AZ latency variance from headline
  evidence; cross-AZ runs as a separate workload row (Phase 13a.3.i).
- [ ] **Network.** Decision: no public ingress, operator access via
  Session Manager or a bastion in a separate subnet. Security group
  permits PG port 5432 only between coordinator SG and each remote SG,
  and bastion SG to coordinator SG. No NAT egress from data-plane
  instances after install; S3 via VPC endpoint. Latency floor target:
  RTT coordinator <-> remote p50 ≤ 1.5 ms in the same-AZ baseline.
- [ ] **Secrets and conninfo.** Decision: one AWS Secrets Manager secret
  per remote named `ecaz-spire-aws-remote-<n>`, carrying keys `host`, `port`,
  `dbname`, `user`, `password`, `sslmode`, `sslrootcert`. Coordinator
  reads secrets by name (no raw conninfo in SQL), matching the secret-name
  indirection enforced by `docs/SPIRE_LIBPQ_RUNBOOK.md`. `sslmode` is
  `verify-full` end to end; any deviation is a reviewer-accepted deferral.

## Phase 13a.2: Datasets (P1)

- [ ] **Correctness tier.** Identity `ec_spire_aws_synth_10k`; 10,000 rows
  at dim 1536; source `ecaz corpus generate` with seed 42. Use: smoke and
  dedupe/identity checks. Query split: 100 queries.
- [ ] **Representative tier.** Identity
  `qdrant-dbpedia-openai3-large-1536-1m`; 1,000,000 rows at dim 1536;
  source `ecaz corpus fetch`. Use: headline correctness and performance
  evidence. Query split: 1,000 queries.
- [ ] **Optional stress tier.** Identity `ec_spire_aws_synth_10m`;
  10,000,000 rows at dim 1536; source `ecaz corpus generate` (seed 42,
  chunked). Only run after Representative passes and the reviewer accepts
  the optional run. Query split: 10,000 queries.
- [ ] **Load path.** Stage raw artifacts in S3; the coordinator runs
  `ecaz corpus prepare` then `ecaz corpus load --profile ec_spire`.
  Remotes receive partitions via the standard SPIRE write path; no
  out-of-band copy. The `ec_hnsw_real_*` parquet preparer profile is used
  for prepare; `ec_spire` is used for load. The pairing is intentional.
- [ ] **Truth cache.** Re-use `--truth-cache-file` per dataset across
  recall runs. The cache file is part of the packet artifacts and carries
  the dataset identity in its name.

## Phase 13a.3: Workload matrix (P1)

Each row below is one Phase 13b sub-packet. Every row produces a recall
number (where applicable), p50/p95/p99 latency, throughput, and the SPIRE
diagnostic snapshot listed in Phase 13a.5.

- [ ] **13a.3.a Vector ORDER BY LIMIT through CustomScan.** Representative
  dataset; k ∈ {10, 100}; sweep `ec_spire.nprobe` ∈ {8, 16, 24, 32};
  concurrency ∈ {1, 4, 8}. `ecaz bench recall` + `ecaz bench latency`.
- [ ] **13a.3.b Vector ORDER BY LIMIT, transport sweep.** Representative
  dataset; k = 10; sweep `ec_spire.remote_tuple_transport` ∈ {auto,
  json_tuple_payload_v1, pg_binary_attr_v1}; concurrency ∈ {1, 4}.
  Transport-bytes evidence depends on Phase 13.0 counter (Phase 13a.5).
- [ ] **13a.3.c Coordinator-routed INSERT.** Representative dataset; batch
  size ∈ {1, 64, 1024}; concurrency ∈ {1, 4}. Verifies 2PC happy path;
  placement-directory counts move as expected.
- [ ] **13a.3.d Non-embedding UPDATE.** Representative dataset; rows-per-tx
  ∈ {1, 100}; concurrency ∈ {1, 4}. No remote prepare; placement directory
  unchanged.
- [ ] **13a.3.e DELETE.** Representative dataset; rows-per-tx ∈ {1, 100};
  concurrency ∈ {1, 4}. Exercises DML CustomScan delete executor.
- [ ] **13a.3.f PK SELECT.** Representative dataset; k = 1;
  concurrency ∈ {1, 8, 32}. Local-only baseline; no remote dispatch.
- [ ] **13a.3.g Degraded-mode read.** Representative dataset; k = 10; one
  remote stopped; `ec_spire.remote_search_consistency_mode = degraded`.
- [ ] **13a.3.h Strict-mode fail-closed read.** Representative dataset;
  k = 10; one remote stopped;
  `ec_spire.remote_search_consistency_mode = strict`. Statement must
  return an error and no rows.
- [ ] **13a.3.i Cross-AZ read (optional).** Representative dataset;
  k = 10; remotes spread across two AZs; nprobe ∈ {8, 16, 32};
  concurrency ∈ {1, 4}. Latency degradation only; recall identical.
- [ ] **13a.3.j Stage E fault subset.** The four CI-gated cases —
  `remote_statement_timeout`, `local_cancel`, `epoch_mismatch`,
  `version_skew` — re-run against the AWS topology.
- [ ] **13a.3.k Stage E lifecycle subset.** Reviewer-selected non-
  destructive lifecycle cases re-run against the AWS topology.

Warmup policy: 50 queries discarded per (sweep × concurrency) cell before
sampling. Per-cell sample size: 1,000 timed queries for Representative,
100 for Correctness.

## Phase 13a.4: Pass / fail thresholds (P1)

- [ ] **Recall@10 (Correctness).** ≥ 0.99 at any nprobe in the sweep.
  Source: `ecaz bench recall` truth-cache compare.
- [ ] **Recall@10 (Representative).** ≥ 0.95 at nprobe = 32.
- [ ] **p50 vector ORDER BY (k=10, conc=1, Representative).** Calibrated;
  floor = 1.5× the local Phase 12 p50 on the same dataset and git SHA.
- [ ] **p99 vector ORDER BY (k=10, conc=1, Representative).** Calibrated;
  floor = 2.5× the local Phase 12 p99 on the same dataset and git SHA.
- [ ] **Remote fanout.** ≤ `ec_spire.remote_search_max_nodes` for every
  read row. Source: EXPLAIN `remote_fanout` on
  `EcSpireDistributedScan`.
- [ ] **Per-row payload bytes (13a.3.b).** ≤
  `ec_spire.max_remote_payload_bytes_per_row`. Source: Phase 13.0 counter
  (Phase 13a.5).
- [ ] **Batch row count (13a.3.b).** ≤
  `ec_spire.max_remote_payload_rows_per_batch`. Source: handoff summary
  `candidate_row_count`.
- [ ] **Degraded read (13a.3.g).** Rows return,
  `degraded_skipped_dispatch_count > 0`,
  `first_degraded_skip_category` matches the injected reason. Source:
  `ec_spire_remote_search_production_executor_session_summary`.
- [ ] **Strict-mode read (13a.3.h).** Statement errors with sanitized
  category matching the injected fault; no rows returned. Source:
  Postgres error log + `docs/SPIRE_LIBPQ_RUNBOOK.md` sanitized
  categories.
- [ ] **2PC recovery (13a.3.c).** Synthetic orphan reaped within one
  reaper invocation. Source:
  `ec_spire_reap_orphaned_remote_prepared_xacts(node_id)`.
- [ ] **Schema drift.** Drift in coord-only, remote-only, both-sides each
  yield the corresponding sanitized category; no silent wrong rows.
  Source: `write_payload.rs` fingerprint guard + Phase 12c.4 disposition.

The "calibrated" latency floors are computed by running the same sweep
on a single-host local PG18 instance with the same dataset before any
AWS run. The local baseline is part of the Phase 13 packet (filed under
`artifacts/local-baseline-*.log`) so the multiplier is checkable.

## Phase 13a.5: Observability inventory and gaps (P1)

### 13a.5.0 Already exposed and citable

- [ ] **GUCs:**
  `ec_spire.remote_search_consistency_mode`,
  `ec_spire.remote_tuple_transport`,
  `ec_spire.remote_search_connect_timeout_ms`,
  `ec_spire.remote_search_statement_timeout_ms`,
  `ec_spire.max_remote_payload_bytes_per_row`,
  `ec_spire.max_remote_payload_rows_per_batch`,
  `ec_spire.remote_search_max_nodes`,
  `ec_spire.remote_search_max_pids`,
  `ec_spire.remote_search_max_pids_per_node`,
  `ec_spire.remote_search_max_concurrent_dispatches`,
  `ec_spire.remote_search_max_concurrent_dispatches_per_node`,
  `ec_spire.nprobe`,
  `ec_spire.rerank_width`,
  `ec_spire.max_candidate_rows`,
  `ec_spire.adaptive_nprobe`,
  `ec_spire.adaptive_nprobe_score_gap_micros`,
  `ec_spire.cost_*` (six entries).
- [ ] **SQL diagnostic functions:**
  `ec_spire_remote_search_degraded_skip_report(index_oid, requested_epoch, query, selected_pids, top_k, consistency_mode)`,
  `ec_spire_remote_search_production_executor_session_summary(index_oid, requested_epoch, query, selected_pids, top_k)`,
  `ec_spire_remote_search_production_scan_handoff_summary(index_oid, query, top_k)`,
  `ec_spire_remote_search_production_read_profile(index_oid, query, top_k)`,
  `ec_spire_reap_orphaned_remote_prepared_xacts(node_id)`,
  `ec_spire_index_active_snapshot_diagnostics(index_oid)`,
  `ec_spire_index_placement_snapshot(index_oid)`,
  `ec_spire_remote_node_snapshot(index_oid)`,
  `ec_spire_register_remote_node_descriptor(index_oid, node_id, descriptor_generation, conninfo_secret_name, remote_index_identity, remote_index_regclass, descriptor_state, last_served_epoch, min_retained_epoch, extension_version, last_error)`.
- [ ] **EXPLAIN properties on `EcSpireDistributedScan`:**
  `remote_fanout`, effective `nprobe`, effective `rerank_width`,
  `tuple_transport_status`.
- [ ] **`SpireActiveSnapshotDiagnostics` fields:**
  `placement_count`, `available_placement_count`,
  `stale_placement_count`, `unavailable_placement_count`,
  `skipped_placement_count`, `object_count`,
  `routing_object_bytes`, `leaf_object_bytes`, `delta_object_bytes`.
- [ ] **Schema-drift fingerprint guard:** three sanitized categories
  (coord-side / remote-side / both-sides) in
  `src/am/ec_spire/coordinator/remote_candidates/write_payload.rs`.

### 13a.5.1..4 Phase 13.0 prerequisite counters (P1)

Each item below must land as its own packet before Phase 13b execution
begins. If any item is reviewer-deferred, the deferral is recorded here
and Phase 13b carries the operator-impact note.

- [ ] **13a.5.1 Per-query tuple-transport bytes.** Today only a static
  `tuple_transport_status` text is exposed. Add an integer
  `tuple_transport_bytes` counter to the scan state, surfaced via the
  handoff summary function and EXPLAIN.
- [ ] **13a.5.2 Per-query route count.** `routing_child_count` exists as
  a summary diagnostic but not as a per-query EXPLAIN field; add it.
- [ ] **13a.5.3 Per-query strict-failure and timeout/cancel counts.**
  Today failures appear as `first_degraded_skip_category` strings; add
  three integer counters (`strict_fail_count`, `remote_timeout_count`,
  `remote_cancel_count`) to the session summary.
- [ ] **13a.5.4 Placement write-contention exposure.** The local test
  harness measures p99 advisory-lock hold time; expose that as a
  cumulative counter in `SpireActiveSnapshotDiagnostics`.

## Phase 13a.6: Fault drills (P1)

Each drill becomes a Phase 13b sub-packet.

- [ ] **13a.6.a Remote down, degraded.** Stop one remote; re-run 13a.3.a
  with `consistency_mode = degraded`. Verify rows return,
  `degraded_skipped_dispatch_count > 0`, and the placement directory
  shows the affected placements as `Stale` or `Unavailable`.
- [ ] **13a.6.b Remote down, strict.** Same fault, with
  `consistency_mode = strict`. Verify the statement errors, no rows,
  sanitized category matches the injected fault.
- [ ] **13a.6.c Orphaned 2PC.** Inject one prepared xact by killing the
  coordinator backend mid-prepare; verify
  `ec_spire_reap_orphaned_remote_prepared_xacts(node_id)` reaps it on
  the next invocation and the placement directory converges.
- [ ] **13a.6.d Required GUC missing.** Start a remote with
  `max_prepared_transactions = 0`; verify the coordinator INSERT path
  fails closed with the sanitized category from
  `docs/SPIRE_LIBPQ_RUNBOOK.md`.
- [ ] **13a.6.e Schema drift.** `ALTER` a non-embedding column on one
  side; verify the fingerprint guard fires with the right sanitized
  category (coord-only / remote-only / both-sides). Revert before the
  next drill.
- [ ] **13a.6.f Auth / certificate failure.** Rotate the Secrets Manager
  secret to an invalid password; verify the sanitized auth category and
  that no unsanitized leak appears in the error message.

## Phase 13a.7: Packet skeleton and reporting (P1)

- [ ] **Parent packet.**
  `review/<NN>-phase13-spire-aws-verification/` where `<NN>` is the next
  free packet number in the agent's range (coder1: 1-9999, coder2:
  10000-19999).
- [ ] **Children.** One sub-packet per matrix row or fault drill, named
  `review/<NN>-phase13-spire-aws-<slice>/` where `<slice>` matches the
  Phase 13a section id (e.g. `13a3a-read-k10`).
- [ ] **Per-packet structure.** Matches `AGENTS.md`:

  ```
  review/<NN>-phase13-spire-aws-<slice>/
    request.md
    artifacts/
      manifest.md
      aws-topology.json
      aws-env.json
      corpus-load-<dataset>.log
      corpus-inspect-<dataset>.log
      bench-recall-<row>.log
      bench-recall-<row>.csv
      bench-latency-<row>.log
      bench-latency-<row>.csv
      bench-spire-pipeline-<row>.log
      diag-handoff-summary-<row>.json
      diag-session-summary-<row>.json
      diag-degraded-skip-<row>.json
      diag-placement-snapshot-<row>.json
      diag-remote-node-snapshot-<row>.json
      fault-drill-<id>.log
      local-baseline-<row>.log
      teardown.log
    feedback/
  ```
- [ ] **Manifest header (mandatory fields).** head SHA, packet topic,
  ISO-8601 timestamp, AWS region, AZ, AMI, kernel, PG version,
  extension version, dataset identity, topology hash, sanitized
  instance IDs and secret ARNs.
- [ ] **Result CSV schema (shared across matrix rows).**

  ```
  row_id,dataset,k,sweep_axis,sweep_value,concurrency,
  mean_ms,p50_ms,p95_ms,p99_ms,throughput_qps,
  recall_at_k,remote_fanout,
  candidate_row_count,merged_candidate_count,duplicate_vec_id_count,
  degraded_skipped_dispatch_count,first_degraded_skip_category,
  tuple_transport_bytes,route_count,
  strict_fail_count,remote_timeout_count,remote_cancel_count
  ```

  Columns sourced from Phase 13a.5.1..4 prerequisites are left empty if
  the counter is deferred, and the deferral is repeated in `request.md`.

## Phase 13a.8: Cost guardrails (P2)

- [ ] Every Phase 13 instance carries the tag set
  `Project=ecaz`, `Phase=13-spire-aws-verification`,
  `Owner=<gh-handle>`, `AutoStop=<ISO-8601-deadline>`.
- [ ] The runbook stops instances within 24h of provisioning unless a
  reviewer-accepted extension is in the packet.
- [ ] Snapshots taken before teardown carry the same tag set. No
  snapshot lives past 30 days without a reviewer-accepted extension.
- [ ] Total expected spend per Phase 13 pass is captured in the parent
  packet manifest; any pass that exceeds the estimate by >2× is
  annotated.

## Phase 13a.10: Operator surface (P1)

Phase 13's operator philosophy is "ecaz-cli is control." The Phase 13b
deliverables are organized so that no operator step requires a hand-rolled
`aws ec2 …` or `psql` pipeline:

- [ ] **Terraform module at `infra/spire-aws/`** brings up the entire AWS
  topology decided in Phase 13a.1. One file per concern (`versions.tf`,
  `variables.tf`, `main.tf`, `outputs.tf`, `terraform.tfvars.example`).
  The `topology` output is a JSON object consumed by every downstream
  script.
- [ ] **Makefile at `infra/spire-aws/Makefile`** exposes one target per
  Phase 13b stage (`provision`, `install-extension`, `register-remotes`, `load-<tier>`,
  `smoke`, `bench-<tier>`, `fault-<drill>`, `teardown`), plus the
  one-shot pass targets `pass-correctness` and `pass-representative`.
- [ ] **Orchestration scripts at `scripts/spire-aws/`** are thin shell
  wrappers over `ecaz` subcommands. Each script reads the topology JSON
  and writes its transcripts to the packet artifact directory.
- [ ] **SQL helpers at `scripts/spire-aws/*.sql`** are the only direct-SQL
  pieces of the workflow. They wrap `ec_spire_register_remote_node_descriptor`,
  the load-bearing GUC verification, the CustomScan plan check, the
  write drivers, and the 2PC orphan injection — every other operator
  surface goes through `ecaz`.
- [ ] **Suite configurations at `scripts/spire-aws/suite-*.json`** map each
  Phase 13a.3 read row onto an `ecaz bench suite run` invocation, so a
  full read-side matrix is one command per tier.
- [ ] **Future ergonomic step (recorded, not blocking):** fold the
  `scripts/spire-aws/*.sh` orchestration into native `ecaz aws <subcommand>`
  subcommands in `ecaz-cli`. See Phase 13b.1.

The operator-surface decisions above are part of the Phase 13a manifest
because they constrain Phase 13b deliverables; changes here amend the
design rather than the runbook.

## Phase 13a.9: Open decisions for reviewer (P1)

These are the items the reviewer must accept (or override) before Phase
13b execution begins.

- [ ] EC2 self-managed PG18 is the build path; RDS and Aurora rejected
  (Phase 13a.1).
- [ ] Single-AZ baseline versus cross-AZ baseline (Phase 13a.1 and
  Phase 13a.3.i).
- [ ] Three remotes is the default fanout (Phase 13a.1).
- [ ] `sslmode=verify-full` is mandatory (Phase 13a.1).
- [ ] Representative dataset is `qdrant-dbpedia-openai3-large-1536-1m`
  (Phase 13a.2).
- [ ] Latency floors are `1.5×` (p50) and `2.5×` (p99) of local baseline
  (Phase 13a.4); reviewer may set tighter or looser multipliers.
- [ ] Phase 13.0 prerequisites 13a.5.1..4 must land before the AWS
  packet, versus being individually deferable with operator-impact
  notes.
- [ ] Stress tier is reviewer-gated (Phase 13a.2 and Suggested Packet
  Sequence below).

## Suggested Packet Sequence

1. **P1 — Phase 13a design acceptance.** This file is reviewed; every
   box in Phase 13a.1..9 is decided or recorded as deferred.
2. **P1 — Phase 13.0 counter prerequisites.** One packet per item in
   Phase 13a.5.1..4 that the reviewer requires before AWS; deferred
   items are recorded on the parent gate file.
3. **P1 — Phase 13b runbook + Terraform module + helper scripts.**
   Deliverables: `infra/spire-aws/` (versions.tf, variables.tf, main.tf,
   outputs.tf, terraform.tfvars.example, Makefile),
   `scripts/spire-aws/*.sh` (install, register, load, smoke, bench,
   fault), `scripts/spire-aws/*.sql` (verify-required-gucs,
   register-remotes, smoke-customscan-read, write-{insert,update,
   delete}, inject-2pc-orphan), and `scripts/spire-aws/suite-*.json`.
   See `task30-phase13b-spire-aws-verification-runbook.md`.
4. **P1 — Phase 13b correctness pass.** Synthetic 10k corpus; smallest
   matrix subset; fault drills 13a.6.a/b.
5. **P2 — Phase 13b representative read pass.** 13a.3.a/b/i; fault
   drills 13a.6.e/f.
6. **P2 — Phase 13b representative write pass.** 13a.3.c/d/e; fault
   drills 13a.6.c/d.
7. **P2 — Phase 13b Stage E subset.** 13a.3.j/k.
8. **P3 — Phase 13b stress pass (optional).** Only if the reviewer
   accepts after P2 passes.
9. **P3 — Phase 13 closeout packet.** Aggregates child packets and
   updates the parent gate file's exit criteria.

## Exit Criteria

- Every box in Phase 13a.1..10 is reviewer-accepted as decided or as a
  recorded deferral.
- Every Phase 13.0 counter prerequisite (Phase 13a.5.1..4) is either
  landed in its own packet or recorded as a reviewer-accepted deferral
  with the operator-impact note that Phase 13b will carry.
- `task30-phase13b-spire-aws-verification-runbook.md` references this
  design by section id (Phase 13a.N) and does not introduce new
  topology, dataset, threshold, or counter facts.
- The parent file `task30-phase13-spire-aws-verification.md` is updated
  to cite Phase 13a (this file) and Phase 13b as the next-level
  sub-phases.
- Phase 13b may proceed under the same evidence-tier rules established
  by Phase 12c.
