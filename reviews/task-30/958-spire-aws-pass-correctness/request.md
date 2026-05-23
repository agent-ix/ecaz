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

Before the AWS run could start, four latent gaps in the AWS lane had to
be fixed (none of these existed in packet 957's scope — they were
discovered while assembling this packet):

- **F3** — `infra/spire-aws/terraform.tfvars` did not exist (only
  `.example`); five required vars had no defaults.
- **F4** — `scripts/spire-aws/bootstrap-node.sh` ran `dnf install
  postgresql18-server` against AL2023's default repos, which do not
  ship PG18. PGDG yum repo was never added.
- **F5** — `pass-correctness` chained `provision → install-extension`
  with no step to stage the ecaz extension tarball into the artifact
  bucket between them. The bucket only exists post-provision; the
  tarball had to exist pre-install. Operator-manual `aws s3 cp` step
  was implied by the runbook but absent from the Makefile chain.
- **F6** — No build path produced an AL2023-compatible extension
  tarball. The local pgrx build in packet 957 produces an Ubuntu 24
  / x86_64 glibc binary; EC2 nodes here are **AL2023 / aarch64
  (Graviton 4 / Neoverse V2)**. ABI-incompatible.

## Scope

Plumbing fixes (committed before AWS spend):

1. `infra/spire-aws/terraform.tfvars` populated for `us-west-2` / AZ
   `us-west-2a` / arm64 AL2023 AMI / Graviton 4 instance types
   (`r8g.4xlarge` coord + 3 × `r8g.2xlarge` remotes) / cost-tag owner
   `kreneskyp` and `auto_stop_at` deadline. `terraform.tfvars` added
   to `.gitignore` (file content is local-only; the AMI ID + region are
   recorded in this request).
2. `scripts/spire-aws/bootstrap-node.sh` installs the PGDG yum repo
   for EL-9-aarch64 and disables the AL2023 built-in postgresql module
   before `dnf install postgresql18-server`.
3. `scripts/spire-aws/build-tarball.sh` (new) builds the ecaz
   extension inside an `amazonlinux:2023` aarch64 container, tuned for
   Neoverse V2 (`RUSTFLAGS=-C target-cpu=neoverse-v2`,
   `CFLAGS/CXXFLAGS=-mcpu=neoverse-v2`). Output:
   `target/ecaz-spire-aws-<short-sha>.tar.gz` plus
   `target/ecaz-spire-aws-latest.tar.gz` symlink-style copy.
4. `infra/spire-aws/Makefile` gains two targets: `package` (Docker
   `--platform linux/arm64` runs the build script) and
   `upload-tarball` (stages the artifact into `s3://<bucket>/<key>`
   from the live `aws-topology.json`). Both inserted into
   `pass-correctness` and `pass-representative` chains.

AWS run:

5. `make -C infra/spire-aws ARTIFACT_DIR=$(pwd)/reviews/task-30/958-spire-aws-pass-correctness/artifacts pass-correctness`
   — drives the full chain: `package → provision → upload-tarball →
   install-extension → register-remotes → load-correctness →
   smoke-correctness → bench-correctness → fault-degraded →
   fault-strict → teardown`.
6. Suite: `scripts/spire-aws/suite-correctness.json` — 10k synthetic
   corpus (`ec_spire_aws_synth_10k`), recall k=10 across 8/16/32
   probes, latency c=1 × 200 iterations.

## Local Verification Inputs

Build host: x86_64 laptop, Docker + qemu user-mode (binfmt registered
via `tonistiigi/binfmt --install arm64`). qemu emulation produces
correct aarch64 binaries because codegen flags are explicit
(`target-cpu=neoverse-v2`) — see follow-up packet for the CI matrix
that replaces qemu with GHA's native `ubuntu-24.04-arm` runner.

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
