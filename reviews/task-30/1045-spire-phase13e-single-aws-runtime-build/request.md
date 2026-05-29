# Review Request: Build Graviton Runtime Once Per AWS Run

## Summary

This checkpoint removes the remaining per-node Rust build cost from the established SPIRE AWS install path.

The local package tarball provides extension SQL/control, but its shared library is built for the local workstation architecture. For the Graviton representative lane, the previous harness uploaded vendored source and made every node build `libecaz.so` and the `ecaz` CLI. After `1044` parallelized SSM dispatch, that still meant multiple redundant Rust builds in each AWS run.

The new install flow is:

- coordinator receives the source tarball, builds the Graviton runtime once, installs it locally, and uploads `ecaz-runtime-linux-aarch64.tar.gz`
- remotes receive no source tarball; they wait for the coordinator-published runtime package and install `lib/ecaz.so` plus `bin/ecaz`
- coordinator remote conninfo still runs only after all node install commands complete
- existing fallback behavior remains for paths where no source/runtime package is available

## Evidence

- `artifacts/bash-n-spire-aws.log`: syntax validation for all `scripts/spire-aws/*.sh`.
- `artifacts/install-runtime-selfcheck.log`: local mocked install run proving:
  - four install commands are submitted before waits
  - coordinator gets `source=ecaz-source.tar.gz build=1 wait=0`
  - remotes get `source=` and `build=0 wait=1`
  - coordinator conninfo is reached after install waits
- `artifacts/aws-running-check.log`: no pending/running/stopping EC2 instances in `us-west-2` during this local-only checkpoint.

## Notes

No AWS resources were started for this checkpoint. This is intended to make the next representative Graviton pass spend one Rust build per run instead of one Rust build per node.
