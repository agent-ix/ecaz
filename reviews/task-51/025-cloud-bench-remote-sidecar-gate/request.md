# Review Request: Cloud Bench Remote Suite Execution + Failed Sidecar Gate

## Scope

This packet covers a narrow `ecaz cloud bench` fix and the failed AWS attempt to run the new RaBitQ8 sidecar variants on the preserved 1M IVF/RaBitQ database.

The intended AWS suite was sidecar-only:

- `rabitq8`
- `rabitq8ls`
- `rabitq8c3`
- `rabitq8c4`

No vchord, pgvectorscale, DiskANN, or unchanged baselines were rerun.

## Code Change

`crates/ecaz-cloud/src/commands/bench.rs` now runs the suite on the DB host through the existing `ecaz` SSM wrapper instead of running locally against the private VPC IP. It writes the suite config onto `/var/lib/pgsql/build/ecaz`, runs `target/release/ecaz bench suite run` against `/var/run/postgresql`, uploads packet artifacts to the benchmark S3 prefix, and syncs them back locally.

This fixes the observed failure mode where local `cloud bench` attempted to connect to `10.42.1.122:5432` from outside the VPC.

## AWS Outcome

No benchmark numbers were produced. Treat this as a failed/incomplete AWS final gate.

- Stack restored from `snap-0758119609e81ab7f`.
- Branch install returned ok on DB host `10.42.1.122`.
- Original `cloud bench` failed before measurement on local private-IP timeout.
- Patched remote-host `cloud bench` started SSM command `897e691d-cd68-4837-bcf1-b0d9cea44ccd`, but did not complete inside the bounded wait.
- Shutdown snapshot recorded as `snap-0b72153293b0b749b`.
- Final status: profile `10k-medium` is `down`, `$0.00/hr running`.

## Evidence

Benchmark packet:

- `benchmarks/task51-aws-rabitq8c4-sidecar-final-gate/manifest.md`
- `benchmarks/task51-aws-rabitq8c4-sidecar-final-gate/suite.json`
- `benchmarks/task51-aws-rabitq8c4-sidecar-final-gate/artifacts/suite-audit-full-sidecar-local.log`
- `benchmarks/task51-aws-rabitq8c4-sidecar-final-gate/artifacts/suite-dry-run-full-sidecar-local.log`
- `benchmarks/task51-aws-rabitq8c4-sidecar-final-gate/artifacts/cloud-bench-full-sidecar.log`
- `benchmarks/task51-aws-rabitq8c4-sidecar-final-gate/artifacts/precheck-preserved-1m-ivf-rabitq.log`
- `benchmarks/task51-aws-rabitq8c4-sidecar-final-gate/artifacts/cloud-bench-remote-full-sidecar.log`
- `benchmarks/task51-aws-rabitq8c4-sidecar-final-gate/artifacts/cloud-down-after-stalled-sidecar.log`

Validation:

- `cargo build -p ecaz-cli --release --no-default-features` passed after the `cloud bench` change.
- `cargo fmt -p ecaz-cloud` completed with the repo's existing stable-rustfmt warnings about unstable import grouping options.

## Review Focus

Please review whether the `cloud bench` remote execution path is the right standard fix before another AWS gate is attempted. The benchmark itself should not be interpreted as a performance result.
