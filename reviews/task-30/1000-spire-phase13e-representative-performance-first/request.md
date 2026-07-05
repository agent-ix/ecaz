# Review Request: SPIRE Representative Performance First

Requester: coder1
Date: 2026-05-27
Head SHA: `417ada8a6a941340807640e8e483bbfe7a674d7d`
Review focus: verify the new no-fault representative AWS pass and pooled-vs-unpooled suite wiring.

## Summary

This slice moves the next AWS proof toward representative latency, recall, and
pooling evidence before fault reruns.

- `ecaz bench suite` now supports per-step `pgoptions` for latency and
  `spire-pipeline` steps, with the value recorded in the suite manifest.
- `scripts/spire-aws/suite-representative-pooling.json` adds a checked-in
  representative pooling A/B suite:
  - `13e4-pooling-disabled-profile-k10`: pool size 0;
  - `13e4-pooling-enabled-profile-k10`: pool size 16;
  - both run the production distributed read profile with query metrics,
    recall, remote-placement requirement, and nprobe sweep `8,16,24,32`.
- `infra/spire-aws` now has `pass-representative-performance`, which runs
  representative load/register/smoke, full representative bench, and the
  pooling A/B suite, but skips resilience/fault drills.

No AWS provisioning or EC2 execution was run for this packet.

## Validation

- `cargo test -p ecaz-cli suite`
  - `30 passed; 0 failed`
- `cargo fmt --check`
  - passed; rustfmt emitted existing stable-toolchain warnings about ignored
    nightly-only import options.
- `make -C infra/spire-aws preflight`
  - passed after rerunning outside the sandbox for Terraform provider discovery.
- `target/debug/ecaz bench suite run --dry-run ... --config scripts/spire-aws/suite-representative-pooling.json`
  - manifest records the expected `PGOPTIONS` for disabled and enabled pooling.
- `bash -n scripts/spire-aws/bench.sh scripts/spire-aws/run-pass-with-watchdog.sh`
  - passed.
- `jq empty scripts/spire-aws/suite-representative-pooling.json scripts/spire-aws/suite-representative.json`
  - passed.

## Next AWS Command

When AWS is explicitly resumed, the prioritized command is:

```sh
SPIRE_AWS_ALLOW_PREEXISTING_RESIDUE=1 \
make -C infra/spire-aws \
  ARTIFACT_DIR=reviews/task-30/<next>-spire-phase13e-aws-representative-performance/artifacts \
  pass-representative-performance
```

That command is intentionally narrower than `pass-representative`: it targets
representative p50/p95/p99, recall, and pooling A/B evidence before fault reruns.
