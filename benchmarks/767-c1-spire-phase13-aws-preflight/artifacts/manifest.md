# Artifact Manifest: SPIRE Phase 13 AWS Preflight Prep

Head SHA: `85aff3cd927de3325c17d15711de5e379a5d561e`
Packet/topic: `767-c1-spire-phase13-aws-preflight`
Lane: Phase 13 AWS preflight prep
Fixture: static local validation only; no AWS resources provisioned
Storage format: n/a
Rerank mode: n/a
Surface: `infra/spire-aws/`, `scripts/spire-aws/`, `ecaz dev sql`
Timestamp: 2026-05-15
Isolated one-index-per-table or shared-table surfaces: n/a

## Commands

1. `make -C infra/spire-aws preflight`
   - Result: passed.
   - Artifact: `make-preflight.log`
   - Key lines:
     - `Terraform has been successfully initialized!`
     - `Success! The configuration is valid.`
     - `shellcheck not found; skipping shellcheck`
     - `jq empty scripts/spire-aws/suite-correctness.json scripts/spire-aws/suite-representative.json scripts/spire-aws/suite-stress.json`

2. `cargo check -p ecaz-cli`
   - Result: passed.
   - Artifact: `cargo-check-ecaz-cli.log`
   - Key lines: `Finished dev profile`; warning only for the pre-existing
     unused-import cluster in `src/am/mod.rs`.

3. `cargo run -p ecaz-cli -- dev sql --help`
   - Result: passed.
   - Artifact: `ecaz-dev-sql-help.log`
   - Key lines:
     - `Run SQL against local pgrx PostgreSQL or a global connection target`
     - `--host <HOST>`
     - `--user <USER>`
     - `--password <PASSWORD>`
     - `--set <SET>`

4. `make -C infra/spire-aws -n pass-correctness pass-representative`
   - Result: passed.
   - Artifact: `make-pass-dry-run.log`
   - Key lines:
     - `terraform apply -auto-approve`
     - `scripts/spire-aws/install.sh`
     - `scripts/spire-aws/register.sh`
     - `PREFIX=ec_spire_aws_synth_10k ... smoke.sh`
     - `PREFIX=ec_spire_aws_repr_1m ... smoke.sh`
     - `terraform destroy -auto-approve`

5. `git diff --check`
   - Result: passed before code commit `85aff3cd927de3325c17d15711de5e379a5d561e`.
