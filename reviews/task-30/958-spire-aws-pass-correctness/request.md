# SPIRE AWS pass-correctness — Initial AWS Spend (Graviton 4)

Task: 30 / Phase 13 (SPIRE AWS Verification)
Sub-phase coverage: 13b.3 → 13b.10 (provision → teardown), bundled
with bootstrap-plumbing fixes that were latent before this packet.
Branch: `task-30-phase13-spire-aws-prep`
Status: in-flight (coder B / SPIRE lane)
Follow-on to: `reviews/task-30/957-spire-aws-prep-local-verification/`

## Context

Packet 957 closed the **local** SPIRE prep baseline: 10k/50k/100k corpora
+ RaBitQ indexes on a dedicated PG18 cluster, all 6 multicluster
fixtures green, multi-disk tablespace placement verified, `preflight`
clean. This packet is the **first real AWS spend** — driving
`make -C infra/spire-aws pass-correctness` against freshly provisioned
infrastructure in `us-west-2`, on **AWS Graviton 4** (`r8g`) hardware.

Before the AWS run could start, three latent gaps in the AWS lane had to
be fixed (none of these existed in packet 957's scope — they were
discovered while assembling this packet):

- **F3** — `infra/spire-aws/terraform.tfvars` did not exist (only
  `.example`); five required vars had no defaults.
- **F4 (rev2)** — `scripts/spire-aws/bootstrap-node.sh` was a stub.
  Initially patched to add PGDG repo, but final implementation
  mirrors the proven `infra/cloud/terraform/cloud-init/db.sh.tftpl`
  approach instead: install AL2023 native `postgresql18-server`
  packages (PG18 18.3 ships in AL2023 base repos for aarch64 — verified
  via `dnf info postgresql18-server`), install Rust + cargo-pgrx, clone
  ecaz, run `cargo pgrx install --sudo --release` on each node from
  the requested git ref. ~10 min per-node build, parallelizable across
  the 4-node topology.
- **F5** — `pass-correctness` chained `provision → install-extension`
  with no actual extension binary path. Original plan added a
  package/upload-tarball stage; pivoted to the per-node build pattern
  above (eliminates Docker, qemu, cross-arch packaging, S3 tarball
  staging) once the cloud-init prior art was discovered. Final chain:
  `provision → install-extension (clones + builds on each node)
  → register-remotes → ...`.

(F6 from the early draft of this request — "no AL2023-compatible
build" — is rendered moot by the per-node-native-build model.)

## Scope

Plumbing fixes (committed before AWS spend):

1. `infra/spire-aws/terraform.tfvars` populated for `us-west-2` / AZ
   `us-west-2a` / arm64 AL2023 AMI (`ami-04e0d7d889f694536`) / Graviton 4
   instance types (`r8g.4xlarge` coord + 3 × `r8g.2xlarge` remotes) /
   cost-tag owner `kreneskyp` and `auto_stop_at` deadline.
   `terraform.tfvars` added to `.gitignore`.
2. `scripts/spire-aws/bootstrap-node.sh` rewritten to mirror
   `infra/cloud/terraform/cloud-init/db.sh.tftpl`: install AL2023
   native `postgresql18-server / contrib / server-devel`, then build +
   install ecaz on the node via `cargo pgrx install --sudo --release
   --pg-config /usr/bin/pg_config` from the operator-supplied
   `ECAZ_GIT_REF`.
3. `scripts/spire-aws/install.sh` passes `ECAZ_GIT_URL` and
   `ECAZ_GIT_REF` env vars through SSM `send-command` to each node;
   `--timeout-seconds 1800` + `aws ssm wait command-executed
   --cli-read-timeout 2000` to cover the ~10 min per-node build.
4. `infra/spire-aws/Makefile` exports `ECAZ_GIT_REF = $(git rev-parse
   HEAD)` and `ECAZ_GIT_URL = https://github.com/agent-ix/ecaz.git`
   by default so a `make pass-correctness` from the working tree
   builds the working-tree SHA on every node. No `package` or
   `upload-tarball` targets; chain restored to
   `provision → install-extension → register-remotes → ...`.

AWS run:

5. `make -C infra/spire-aws ARTIFACT_DIR=$(pwd)/reviews/task-30/958-spire-aws-pass-correctness/artifacts pass-correctness`
   — drives the chain: `provision → install-extension →
   register-remotes → load-correctness → smoke-correctness →
   bench-correctness → fault-degraded → fault-strict → teardown`.
6. Suite: `scripts/spire-aws/suite-correctness.json` — 10k synthetic
   corpus (`ec_spire_aws_synth_10k`), recall k=10 across 8/16/32
   probes, latency c=1 × 200 iterations.

## Build Locality

Each EC2 node performs its own `cargo pgrx install --release` from the
pinned git SHA. No laptop-side Docker / qemu / tarball. Pattern reuses
the proven Phase 13 Graviton 4 cloud-init (see
`benchmarks/cloud-10k-graviton-preopt-baselines/manifest.md`).
A follow-up packet may add a GHA `package` job on
`ubuntu-24.04-arm` (native arm64 runner) to produce a shippable tarball
artifact for users who don't want to wait for an on-node build.

## Deliverables

- `artifacts/manifest.md` — ledger with one row per Makefile stage,
  recording head SHA, suite identity, command, AWS topology identity
  (VPC ID, AZ, coord instance ID, remote instance IDs, S3 bucket
  name), tarball SHA256, timestamp, and key result line.
- `artifacts/aws-topology.json` — terraform output captured by
  `make provision`.
- `artifacts/tarball-upload.log` — `make upload-tarball` transcript +
  tarball SHA256.
- `artifacts/install-*.log` — one log per node from SSM `send-command`.
- `artifacts/register-remotes/` — coordinator-side registration output
  + `verify-required-gucs.sql` transcript.
- `artifacts/load-correctness/`, `smoke-correctness/`,
  `bench-correctness/` — per-stage logs and JSON outputs.
- `artifacts/bench-correctness/suite-manifest.json` + `results.jsonl`
  — FR-038 canonical outputs from `ecaz bench suite`.
- `artifacts/fault-degraded/`, `fault-strict/` — fault matrix logs.
- `artifacts/teardown.log` — `terraform destroy` output.
- `artifacts/post-teardown-instance-check.log` — `aws ec2
  describe-instances` confirming zero ecaz/spire instances remain.

## Out of Scope

- `pass-representative` (1M `qdrant-dbpedia`, dim 1536). F2
  page-overflow risk from packet 957 is unprobed at 1M scale; this
  packet does **not** clear the gate for `pass-representative`.
- `pass-stress` (10M synthetic). Reviewer-gated per Phase 13a.9.
- CI `package` matrix job (Graviton 4 + amd64 multi-arch shipping
  pipeline). Will be its own packet; this packet's local-Docker+qemu
  build is interim.
- Graviton 5 verification — deferred until AWS access.

## Cost Estimate

`r8g.4xlarge` ≈ $0.86/hr; `r8g.2xlarge` ≈ $0.43/hr × 3 = $1.29/hr.
Coord + 3 remotes ≈ $2.15/hr instance time. EBS gp3 (200 GB +
3 × 100 GB) ≈ $0.04/hr. Estimated `pass-correctness` wall time ~1–2
hours including provision + teardown. **Expected total spend:
$4–$10.** Hard ceiling on instance hours enforced by `auto_stop_at`
tag (Phase 13a.8) and operator-side `aws ec2 describe-instances`
post-teardown verification.

## Findings to be Recorded as the Run Progresses

- F3 — terraform.tfvars baseline (resolved by this packet)
- F4 — bootstrap PGDG repo (resolved by this packet)
- F5 — pass-correctness chain gap (resolved by this packet)
- F6 — AL2023 / Graviton 4 build path (resolved by this packet)
- Any new findings surfaced during the AWS run (e.g. AWS quota,
  SSM permission gaps, Secrets Manager regional behavior) recorded
  inline in `artifacts/manifest.md` as they arise.

See `artifacts/manifest.md` for the live ledger.
