# Review Request: SPIRE Representative Preflight Rerun

Task: `plan/tasks/task30-phase13e-spire-aws-production-gap-closure.md`

Evidence-only checkpoint at head `1810830a6`.

## Summary

This packet records a fresh local readiness rerun for the remaining Phase 13e
representative performance path. No AWS provisioning was started.

The preflight passed and therefore still gates the representative AWS pass on:

- priority and pooling suite coverage for representative latency, recall, and
  production read profile rows;
- recall-floor thresholds at `nprobe=32`;
- pooled-vs-unpooled `PGOPTIONS` coverage;
- the ordered preflight/provision/install/verify Makefile path;
- the ordered tunneled verify path
  load/register/smoke/priority bench/pooling bench/summarize/verify;
- the representative watchdog timeout floor and summary-gate self-checks.

I also ran a read-only EC2 state check for the established `us-west-2` lane. It
returned `0` non-terminated instances.

## Validation

- `bash scripts/spire-aws/preflight-representative-performance.sh`
- `aws ec2 describe-instances --region us-west-2 --filters Name=instance-state-name,Values=pending,running,stopping,stopped --query 'length(Reservations[].Instances[])' --output text`

## Artifacts

- `artifacts/manifest.md`
- `artifacts/preflight-representative-performance.log`
- `artifacts/aws-us-west-2-nonterminated-count.log`
- `artifacts/aws-us-west-2-nonterminated-instances.log`

The existing untracked SPIRE artifact directory under
`scripts/spire-aws/artifacts/` was left untouched and was not staged.
