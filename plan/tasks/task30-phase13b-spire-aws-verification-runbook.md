# Task 30 Phase 13b: SPIRE AWS Verification Runbook

Status: runbook draft, pending reviewer acceptance
Owner: coder1 / SPIRE AWS verification track
Priority: P1 — blocks any Phase 13 execution; pairs with
`task30-phase13a-spire-aws-verification-design.md`.

## Goal

Produce a reviewer-accepted operator runbook plus the supporting
Terraform module and helper scripts that turn the Phase 13a design
decisions into an executable AWS verification pass. Phase 13b honours
the project philosophy in `AGENTS.md` and `CLAUDE.md`: "ecaz-cli is
control" — Terraform brings up infra, `ecaz` subcommands do everything
else, and no operator step is a hand-rolled `aws ec2 …` or `psql`
pipeline.

The operator-facing surface for one Phase 13 pass is **two commands**:

```
make -C infra/spire-aws provision
make -C infra/spire-aws pass-correctness   # or pass-representative
```

Every other Phase 13b.N section in this runbook is the same workflow
broken out one stage at a time. Each stage cites a decision section in
Phase 13a (referenced as **13a.N** below) and does not introduce new
topology, dataset, threshold, or counter facts. If a runbook author or
operator discovers one mid-work, the design file is amended first and
Phase 13a is re-accepted.

## Entry State

- Phase 13a is reviewer-accepted: every box in 13a.1..10 is decided or
  recorded as a deferral.
- Phase 13.0 counter prerequisites (Phase 13a.5.1..4) are landed or
  deferred-with-operator-impact-note.
- Parent gate file `task30-phase13-spire-aws-verification.md` cites
  Phase 13a and this Phase 13b as the next-level sub-phases.

## Non-Goals

- Adding new topology, dataset, threshold, or counter decisions. Those
  belong in Phase 13a.
- Provisioning AWS resources from this file. Provisioning happens only
  when an operator runs `make -C infra/spire-aws provision` against a fresh
  Phase 13 packet.
- Folding the orchestration scripts into native `ecaz aws <subcommand>`
  subcommands in `ecaz-cli`. That is the recorded next-step in Phase
  13a.10 but is not a Phase 13b deliverable.
- A second cloud (GCP, Azure). AWS-only per Phase 13a non-goals.

## Conventions

- The local agent sandbox invokes the CLI by absolute path:
  `/Users/peter/.cargo/bin/ecaz` (per `CLAUDE.md`). On AWS hosts, after
  install, the CLI is on `PATH` and invoked as `ecaz`. Both forms appear
  below where they apply.
- Coordinator and remote shells are reached via AWS Session Manager
  (`aws ssm start-session --target <instance-id>`) — see **13a.1.d**.
- Every Makefile target reads the topology from
  `$(ARTIFACT_DIR)/aws-topology.json` (produced by `make setup`) and
  writes transcripts under
  `review/<NN>-phase13-spire-aws-<slice>/artifacts/` per **13a.7**.
- All times are UTC. The packet manifest carries the ISO-8601 timestamp
  of each run.

## Phase 13b.1: Terraform module and helper scripts (P1)

Phase 13b.1 is the single deliverable that turns Phase 13a's decisions
into an executable workflow. Reviewer acceptance of Phase 13b.1 is the
gate for Phase 13b.2..N.

- [ ] **`infra/spire-aws/`** — Terraform module that provisions the
  Phase 13a.1 topology. Files:
  - [ ] `versions.tf` — provider pin (AWS ~> 5.0, random ~> 3.5).
  - [ ] `variables.tf` — region, AZ, AMI, instance types, remote
    count, storage sizes, owner / auto-stop tags.
  - [ ] `main.tf` — VPC, private subnet, two security groups
    (`ecaz-spire-aws-coord`, `ecaz-spire-aws-remote`), S3 gateway endpoint,
    SSM / Secrets Manager interface endpoints, IAM role + instance
    profile, S3 artifact bucket, Secrets Manager secrets with
    `random_password`, coordinator and remote `aws_instance` resources.
  - [ ] `outputs.tf` — every id and ARN plus a `topology` JSON object
    consumed by `scripts/spire-aws/*.sh`.
  - [ ] `terraform.tfvars.example` — Phase 13a.1 defaults pre-populated.
  - [ ] `Makefile` — one target per Phase 13b stage plus the
    `pass-correctness` and `pass-representative` one-shot passes.
- [ ] **`scripts/spire-aws/`** — thin shell wrappers over `ecaz`.
  - [ ] `bootstrap-node.sh` — runs once per node via SSM; installs PG18
    and the ecaz extension tarball.
  - [ ] `install.sh` — SSM-driven invocation of `bootstrap-node.sh`
    across coordinator and remotes.
  - [ ] `register.sh` — iterates remotes from the topology JSON and
    calls `ecaz dev sql --file register-remotes.sql` for each.
  - [ ] `load.sh` — one tier per call (`correctness`, `representative`,
    `stress`); chains `ecaz corpus generate|fetch|prepare|load|inspect`.
  - [ ] `smoke.sh` — `ecaz dev sql --file smoke-customscan-read.sql`
    plus `ecaz bench spire-pipeline` against the Correctness corpus.
  - [ ] `bench.sh` — `ecaz bench suite run --config suite-<tier>.json`.
  - [ ] `fault.sh` — one drill per call; uses `aws ec2 stop-instances`
    / `aws secretsmanager put-secret-value` for the injection step and
    `ecaz dev sql` for the recovery and diagnostic capture.
- [ ] **`scripts/spire-aws/*.sql`** — direct-SQL helpers.
  - [ ] `verify-required-gucs.sql` — `SHOW` every load-bearing GUC and
    `max_prepared_transactions`; lists `ec_spire.*` + extension
    version.
  - [ ] `register-remotes.sql` — wraps
    `ec_spire_register_remote_node_descriptor()` with `psql` variables.
  - [ ] `smoke-customscan-read.sql` — `EXPLAIN ANALYZE` of vector
    `ORDER BY LIMIT`; remote-node and handoff-summary checks.
  - [ ] `write-insert.sql`, `write-update.sql`, `write-delete.sql` —
    parameterised batch DML with pre/post placement snapshots.
  - [ ] `inject-2pc-orphan.sql` — opens an INSERT + `PREPARE
    TRANSACTION` and leaves it dangling for the reaper drill.
- [ ] **`scripts/spire-aws/suite-{correctness,representative,stress}.json`**
  — `ecaz bench suite run` configurations covering the Phase 13a.3 read
  rows for each tier.

Phase 13b.1 is reviewer-acceptable when:

- `terraform init && terraform validate` passes against `infra/spire-aws/`.
- Every SQL helper parses against a local PG18 with the ecaz extension
  installed (use `ecaz dev sql --file ... --raw`).
- Every shell helper passes `bash -n` and `shellcheck`.

## Phase 13b.2: Pre-flight (P1)

- [ ] Parent gate file lists every Phase 12 / Phase 12c.4 item as
  complete or reviewer-deferred.
- [ ] Phase 13a is reviewer-accepted.
- [ ] Phase 13.0 counter prerequisites (Phase 13a.5.1..4) are landed
  *or* the deferral is recorded in this run's parent packet
  `request.md`.
- [ ] Phase 13d read-profile surface is landed and reviewer-accepted.
- [ ] Parent packet `review/<NN>-phase13-spire-aws-verification/` exists
  with `request.md` and `artifacts/manifest.md` on a feature branch.
- [ ] AWS account quota for `r6i.4xlarge` + 3× `r6i.2xlarge` confirmed
  in the target region.
- [ ] Cost-tag set per **13a.8** is defined in
  `infra/spire-aws/terraform.tfvars` (`owner`, `auto_stop_at`).
- [ ] `aws` CLI is logged in with a role that can manage EC2, VPC,
  Secrets Manager, S3, and IAM in the target region.
- [ ] `gh` CLI is logged in for packet pushes.
- [ ] The ecaz extension tarball for the head SHA has been uploaded to
  the Phase 13 artifact bucket (see Phase 13b.4 note).

If any box is unchecked, stop.

## Phase 13b.3: Provision (P1)

```
make -C infra/spire-aws provision
```

What it does:

- Runs `terraform init` and `terraform apply -auto-approve` against
  `infra/spire-aws/` with the Phase 13a.1 topology baked into the module
  defaults.
- Writes the `topology` Terraform output to
  `$(ARTIFACT_DIR)/aws-topology.json`.

Verification:

- [ ] `aws-topology.json` lists one coordinator and three remotes.
- [ ] Every instance carries the cost-tag set from **13a.8**.
- [ ] No security group permits public ingress.

If any check fails, run `make -C infra/spire-aws teardown` and amend the
Terraform module before retrying.

## Phase 13b.4: Install extension on every node (P1)

```
make -C infra/spire-aws install-extension
```

What it does:

- Reads the topology JSON.
- Issues one `aws ssm send-command` that pulls `bootstrap-node.sh` from
  the artifact S3 bucket and runs it on the coordinator and every
  remote.
- Captures one `install-<instance-id>.log` per node.

**Prerequisite:** the ecaz extension tarball must already live in the
bucket at the key `bootstrap-node.sh` expects (`ecaz-latest.tar.gz` by
default). Stage it via `aws s3 cp target/.../ecaz.tar.gz s3://$BUCKET/`
before running `make install`.

Verification:

- [ ] Every install log ends with `SELECT extversion FROM pg_extension`
  printing the same version.

## Phase 13b.5: Register remotes on the coordinator (P1)

```
make -C infra/spire-aws register-remotes
```

What it does:

- Runs `verify-required-gucs.sql` against the coordinator (and is
  re-runnable against each remote with `--host <remote-ip>`).
- For each remote in the topology JSON, calls
  `ec_spire_register_remote_node_descriptor()` via
  `register-remotes.sql`.
- Prints the coordinator-side `ec_spire_remote_node_snapshot()` as the
  baseline `remote-node-snapshot-baseline.log`.

Verification:

- [ ] `remote-node-snapshot-baseline.log` shows three rows with
  `descriptor_state = 'active'`.
- [ ] `verify-gucs-coord.log` reports `max_prepared_transactions >= 64`
  on the coordinator. Re-run for each remote when investigating fault
  drill 13b.7.d.

## Phase 13b.6: Load corpus (P1)

```
make -C infra/spire-aws load-correctness
# or
make -C infra/spire-aws load-representative
# (optional, reviewer-gated)
make -C infra/spire-aws load-stress
```

What it does:

- `load-correctness` synthesises a 10k-row dim-1536 corpus on the
  coordinator and loads it via `ecaz corpus load --profile ec_spire`.
- `load-representative` runs `ecaz corpus fetch / prepare / load` for
  `qdrant-dbpedia-openai3-large-1536-1m`.
- `load-stress` synthesises and loads the 10M-row tier per **13a.2**.

Verification:

- [ ] `corpus-inspect-<tier>.log` reports the expected row count and
  shows `ec_spire` as a ready profile.

## Phase 13b.7: Smoke verification (P1)

```
make -C infra/spire-aws smoke
```

What it does:

- Runs `smoke-customscan-read.sql` (EXPLAIN ANALYZE + remote-node and
  handoff-summary checks).
- Captures one
  `ec_spire_remote_search_production_read_profile(index_oid, query, 10)`
  rowset as `production-read-profile-smoke.log`.
- Runs `ecaz bench spire-pipeline --include-remote` against the
  Correctness corpus.

Verification:

- [ ] The EXPLAIN plan contains `EcSpireDistributedScan`.
- [ ] `remote_fanout` equals 3 (the registered remote count).
- [ ] `tuple_transport_status` is `ready`.
- [ ] The production read profile reports one socket open, one regclass
  probe, one endpoint-identity query, one candidate receive, and one heap
  receive per successful remote dispatch.

If the smoke fails, do not proceed to the matrix. File a child packet
with the failing logs and the install / register transcripts.

## Phase 13b.8: Workload matrix (P1)

One sub-packet per matrix row from **13a.3**. The read rows run via the
suite configurations:

```
make -C infra/spire-aws bench-correctness
make -C infra/spire-aws bench-representative
make -C infra/spire-aws bench-stress   # reviewer-gated
```

Each invocation produces `suite-manifest-<tier>.json` and
`suite-results-<tier>.jsonl` under the artifact directory. The suite
configurations cover the read rows from **13a.3.a** (k=10 and k=100,
concurrency 1/4/8) and **13a.3.f** (PK SELECT at concurrency 32).
Every read sub-packet also captures a packet-local
`production-read-profile-<tier>-k<k>-c<concurrency>.log` rowset for at
least the representative query sample used to explain any latency
regression.

Rows that the suite schema does not cover today are driven directly:

- [ ] **13b.8.b Transport sweep (row 13a.3.b).** For each value in
  {`auto`, `json_tuple_payload_v1`, `pg_binary_attr_v1`}:

  ```
  ecaz dev sql --host <coord-ip> --user ecaz_coord --database postgres \
    --sql "SET ec_spire.remote_tuple_transport = '<value>'"

  ecaz bench latency --host <coord-ip> --user ecaz_coord --database postgres \
    --prefix ec_spire_aws_repr_1m --profile ec_spire \
    --k 10 --sweep 32 --concurrency 1 --iterations 1000 \
    --log-output <pkt>/artifacts/bench-latency-transport-<value>-c1.log

  ecaz bench spire-pipeline --host <coord-ip> --user ecaz_coord --database postgres \
    --prefix ec_spire_aws_repr_1m \
    --queries-limit 100 --remote-tuple-transport <value> \
    --include-remote --include-query-metrics \
    --log-output <pkt>/artifacts/bench-pipeline-transport-<value>.log
  ```

  If counter **13a.5.1** is deferred, grade 13a.3.b on latency and
  throughput only and record the deferral in the sub-packet.
- [ ] **13b.8.c Write rows (13a.3.c, 13a.3.d, 13a.3.e).** Drive each
  with `ecaz dev sql --file scripts/spire-aws/write-*.sql`:

  ```
  ecaz dev sql --host <coord-ip> --user ecaz_coord --database postgres \
    --file scripts/spire-aws/write-insert.sql \
    --set prefix=ec_spire_aws_repr_1m --set batch=64 --set rows=10000 \
    --log-output <pkt>/artifacts/write-insert-b64.log
  ```

  Same shape for `write-update.sql` and `write-delete.sql` with their
  own `:per_tx` and `:rows` variables.
- [ ] **13b.8.e Stage E subset (13a.3.j, 13a.3.k).** Re-run the four
  CI-gated Stage E fault cases against the AWS coordinator using the
  SQL fixtures from `crates/ecaz-cli/src/dev/spire_multicluster/`.
  Lifecycle subset per the reviewer's selection in **13a.3.k**.
- [ ] **13b.8.f Local baseline pair.** Every row with a latency
  threshold runs the same sweep locally on a Phase 12 host with the
  same dataset and git SHA. Log under
  `<pkt>/artifacts/local-baseline-*.log`.

## Phase 13b.9: Fault drills (P1)

One sub-packet per drill from **13a.6**.

```
make -C infra/spire-aws fault-remote-down-degraded         # 13a.6.a
make -C infra/spire-aws fault-remote-down-strict           # 13a.6.b
make -C infra/spire-aws fault-orphaned-2pc              # 13a.6.c
make -C infra/spire-aws fault-missing-guc      # 13a.6.d (operator-driven, see below)
make -C infra/spire-aws fault-schema-drift     # 13a.6.e (operator-driven, see below)
make -C infra/spire-aws fault-auth-failure             # 13a.6.f
```

`fault.sh` automates injection + recovery + diagnostic capture for the
drills where the workflow is fully scripted. `fault-missing-guc` and
`fault-schema-drift` require an SSM session on a remote to flip a
postgresql.conf value or run an `ALTER` — the script prints the
operator step and captures the verification snapshot.

Each drill captures:

- `fault-<drill>.log` — orchestration transcript.
- `fault-<drill>-bench.log` — re-run latency probe under the fault.
- `fault-<drill>-session-summary.log`,
  `fault-<drill>-placement.log` — diagnostic snapshots.

Verification per **13a.4**:

- [ ] `degraded`: rows returned;
  `degraded_skipped_dispatch_count > 0`; placement directory shows the
  affected placements as `Stale` / `Unavailable`.
- [ ] `strict`: bench command exits non-zero with the sanitized
  category from `docs/SPIRE_LIBPQ_RUNBOOK.md`.
- [ ] `orphaned-2pc`: `pg_prepared_xacts` empty after the reap;
  placement directory converges.
- [ ] `missing-guc`: coordinator INSERT fails closed with the sanitized
  category; restore `max_prepared_transactions` and re-verify.
- [ ] `schema-drift`: fingerprint guard fires with the right sanitized
  category (coord-only / remote-only / both-sides); revert.
- [ ] `auth-failure`: read errors with sanitized auth category; restore
  the secret and re-verify.

## Phase 13b.10: Capture artifacts (P1)

For every Make target above:

- [ ] The transcript lives in
  `review/<NN>-phase13-spire-aws-<slice>/artifacts/`.
- [ ] `manifest.md` carries every field from **13a.7**: head SHA,
  packet/topic, ISO-8601 timestamp, dataset identity, AWS region, AMI,
  PG version, extension version, AZ, sanitized instance IDs, sanitized
  secret ARNs, git diff state.
- [ ] Each artifact has a manifest entry with `lane / fixture`,
  `command`, and `key result lines`.
- [ ] `request.md` summarizes the slice and cites packet-local
  artifacts by filename.
- [ ] Push immediately after committing. Local-only files are invisible
  to other agents per `AGENTS.md`.

## Phase 13b.11: Teardown (P1)

```
make -C infra/spire-aws teardown
```

What it does: `terraform destroy -auto-approve` against
`infra/spire-aws/`.

Operator steps before / after:

- [ ] Capture EBS snapshots manually with the cost-tag set if forensic
  retention is desired (`aws ec2 create-snapshot`); Terraform destroy
  does not snapshot by default.
- [ ] After destroy, verify the cost-tag report shows no live
  `Phase=13-spire-aws-verification` resources except reviewer-accepted
  retained snapshots.
- [ ] Push the closeout `teardown.log` and the final tag-report to the
  parent packet.

## Phase 13b.12: Cookbook (P2)

**Full correctness pass, one command:**

```
make -C infra/spire-aws pass-correctness
```

This chains `setup → install → register → load-correctness → smoke →
bench-correctness → fault-degraded → fault-strict → teardown`.

**Full representative pass, one command:**

```
make -C infra/spire-aws pass-representative
```

Chains `setup → install → register → load-representative → smoke →
bench-representative → fault-2pc → fault-schema-drift → teardown`.

**Re-run one matrix row on an already-provisioned topology:**

```
ARTIFACT_DIR=review/<NN>-phase13-spire-aws-13a3a-rerun/artifacts \
  scripts/spire-aws/bench.sh representative \
  $(infra/spire-aws/.../aws-topology.json) \
  $ARTIFACT_DIR
```

## Phase 13b.13: Troubleshooting (P2)

| Symptom | Likely cause | Action |
|---------|--------------|--------|
| `terraform apply` fails on Secrets Manager creation | Region quota or previous identical secret in 7-day deletion window | Re-apply with `aws secretsmanager list-secrets --include-planned-deletion` cleared, or pick a fresh secret name prefix |
| SSM `send-command` returns `InvalidInstanceId` | SSM agent not yet up on the instance | Wait for instance `running` state plus 1–2 minutes; rerun `make install` |
| `EcSpireDistributedScan` absent from plan | Extension missing on a remote, or registration row missing | Re-run `make install` and `make register`; verify `SELECT extversion FROM pg_extension` per node |
| `tuple_transport_status` not `ready` | Schema drift, or extension version skew | Run the schema-drift fault check; verify extversion identical across cluster |
| Coordinator INSERT fails closed at prepare | `max_prepared_transactions` missing on a remote | Run `verify-required-gucs.sql` against every remote |
| Sanitized auth error on every read | Secret rotation in flight | Inspect Secrets Manager rotation status; restore prior version |
| Higher-than-expected latency variance | Cross-AZ traffic against expectation | Inspect `aws-topology.json`; verify AZ field |
| Reaper returns 0 but orphans persist | Wrong `node_id` | Cross-reference node id from `ec_spire_remote_node_snapshot()` |

## Known caveats

- `ecaz aws <subcommand>` does not exist as a native CLI surface yet.
  Phase 13b uses `make -C infra/spire-aws …` plus
  `scripts/spire-aws/*.sh`; the philosophical next step recorded in
  Phase 13a.10 is to fold these into `ecaz-cli`.
- `ecaz remote register` does not exist as a CLI subcommand either.
  Phase 13b registers remotes via `register-remotes.sql` invoked
  through `ecaz dev sql`.
- `ecaz dev install ecaz-pg-test` targets a local pgrx tree only; the
  AWS install path uses the S3-staged tarball driven by
  `bootstrap-node.sh`.
- Counter prerequisites **13a.5.1..4** may be partially deferred at
  reviewer discretion; every deferred counter is repeated in the parent
  packet with its operator impact.
- The Stress tier is reviewer-gated; `pass-representative` does not
  include it.
- The existing repo has no prior Terraform module for AWS benchmarking;
  the convention here is greenfield. If the project later adopts a
  shared TF style from a sibling repo, the Phase 13 module is the
  single thing to align.

## Suggested Packet Sequence

1. **P1 — Phase 13b.1 Terraform module + helper scripts.** Commit
   `infra/spire-aws/` and `scripts/spire-aws/`; verify `terraform validate`
   passes and every SQL helper parses against local PG18.
2. **P1 — Phase 13b runbook acceptance.** This file is reviewed
   alongside Phase 13b.1.
3. **P1 — Correctness pass.** `make all-correctness` against a fresh
   packet; fault drills 13b.9 `degraded` / `strict`.
4. **P2 — Representative read pass.** `make load-representative
   smoke bench-representative` + fault drills 13b.9 `schema-drift` /
   `auth`.
5. **P2 — Representative write pass.** Phase 13b.8.c write rows + fault
   drills 13b.9 `orphaned-2pc` / `missing-guc`.
6. **P2 — Stage E subset.** Phase 13b.8.e.
7. **P3 — Stress pass (optional).** Only after reviewer accepts.
8. **P3 — Closeout.** Aggregate sub-packets, update parent gate file
   exit criteria.

## Exit Criteria

- Phase 13b.1 deliverables (`infra/spire-aws/`, `scripts/spire-aws/`) are
  committed and reviewer-accepted.
- Every sub-packet from Phase 13b.8 and 13b.9 is reviewer-accepted.
- Phase 13b.10 captures are complete for every executed slice.
- Phase 13b.11 teardown completes and the cost-tag report shows no
  live Phase 13 resources except reviewer-accepted retained snapshots.
- Any deferred Phase 13.0 counter is repeated in the parent packet
  with operator impact, matching the Phase 12c.4 pattern.
- The parent file `task30-phase13-spire-aws-verification.md` cites
  this Phase 13b and Phase 13a as the next-level sub-phases.
