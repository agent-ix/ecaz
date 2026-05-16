# Review Request: SPIRE Phase 13 AWS Preflight Prep

**Requester:** coder1
**Date:** 2026-05-15
**Code commit:** `85aff3cd927de3325c17d15711de5e379a5d561e`
**Review focus:** non-provisioning prep needed before spending AWS time on
Phase 13 verification.

## Summary

This checkpoint does not provision AWS resources. It makes the Phase 13 AWS
operator surface preflightable and aligns the runbook with the checked-in
automation:

- fixes `infra/spire-aws/Makefile` so `REPO_ROOT` resolves to the repository
  root instead of `infra/`;
- adds the runbook target names `preflight`, `provision`,
  `install-extension`, `register-remotes`, `pass-correctness`, and
  `pass-representative`, while preserving compatibility aliases;
- adds a non-provisioning `make -C infra/spire-aws preflight` target covering
  Terraform formatting/init/validation, shell syntax, optional shellcheck, and
  suite JSON parsing;
- commits the Terraform provider lock file and ignores generated Terraform
  caches/state;
- makes `scripts/spire-aws/*.sh` robust to caller working directory by
  resolving `REPO_ROOT`;
- makes `install.sh` stage `bootstrap-node.sh` into the artifact bucket and
  pass the required bootstrap environment to SSM;
- updates AWS smoke to capture the Phase 13d production read profile rowset
  and to smoke the selected dataset prefix for correctness or representative
  passes;
- updates `ecaz dev sql` so AWS scripts can use global `--host`, `--port`,
  `--user`, `--password`, and repeated `--set NAME=VALUE` psql variables while
  preserving the local pgrx default path.

## Files To Review

- `infra/spire-aws/Makefile`
- `infra/spire-aws/.terraform.lock.hcl`
- `scripts/spire-aws/*.sh`
- `scripts/spire-aws/smoke-customscan-read.sql`
- `crates/ecaz-cli/src/cli.rs`
- `crates/ecaz-cli/src/commands/dev/mod.rs`
- `crates/ecaz-cli/src/commands/dev/sql.rs`
- `plan/tasks/task30-phase13b-spire-aws-verification-runbook.md`
- `.gitignore`

## Validation

- `make -C infra/spire-aws preflight` passed. It ran `terraform fmt -check`,
  `terraform init -backend=false`, `terraform validate`, `bash -n
  scripts/spire-aws/*.sh`, optional `shellcheck`, and `jq empty` for the three
  suite JSON files.
- `cargo check -p ecaz-cli` passed, with only the pre-existing unused import
  warning in the extension crate.
- `cargo run -p ecaz-cli -- dev sql --help` passed and shows `--host`,
  `--user`, `--password`, `--database`, and `--set`.
- `make -C infra/spire-aws -n pass-correctness pass-representative` passed and
  shows the expected target chain without provisioning.
- `git diff --check` passed before the code commit.

## Known Limits Before AWS

- This packet is preflight-only. It does not prove AWS account quota, AWS CLI
  login, AMI selection, extension tarball upload, or runtime EC2 behavior.
- `shellcheck` is not installed in the local environment, so `preflight`
  reported `shellcheck not found; skipping shellcheck`. The target will run
  shellcheck automatically where available.
- Phase 13a/13b/13d still need reviewer acceptance before the AWS pass should
  start.
- The two unrelated dirty Python test files already present in the worktree are
  not part of this checkpoint.

## Reviewer Questions

1. Is the `ecaz dev sql` global-connection behavior acceptable for AWS scripts,
   or should AWS SQL execution get a separate non-dev CLI command before the
   first run?
2. Is the `preflight` target sufficient as the no-spend gate before
   `provision`, given that account quota, AMI choice, and tarball upload remain
   operator-owned checks?
