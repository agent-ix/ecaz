# Review Request: Node-Local Representative AWS Load

Task: Phase 13e representative performance gate.

Code commit: `2020771db` (`Use node-local loads for SPIRE representative AWS`)

## Context

The previous representative AWS rerun reached real `ec_real_100k` corpus load but failed before SPIRE build/query with:

- `COPY finish failed for ec_spire_aws_repr_1m_corpus`
- `connection closed`

The failure happened while streaming the large representative TSV through the SSM PostgreSQL port-forward. That path is adequate for small correctness data but fragile for the representative tier.

## Change

- `scripts/spire-aws/bootstrap-node.sh` now builds and installs `/usr/local/bin/ecaz` on each AWS node from the same vendored source bundle used for the extension build.
- `scripts/spire-aws/load.sh` stages representative coordinator and remote TSV inputs to the artifact S3 bucket, runs `ecaz corpus load` node-local through SSM, and downloads node-side load/reset/inspect logs back into the packet.
- Correctness and stress load paths are left on the existing operator/tunnel flow.
- `scripts/spire-aws/preflight-representative-performance.sh` now fails closed unless the representative load script retains the node-local coordinator and remote load path and bootstrap installs the CLI binary.

## Validation

- `artifacts/bash-n.log`: shell syntax passed.
- `artifacts/representative-preflight.log`: representative preflight passed with the new node-local-load guards.
- `artifacts/cargo-build-ecaz-cli.log`: CLI binary compiles locally.
- `artifacts/git-diff-check.log`: whitespace check passed.

## Remaining

This is a local harness hardening checkpoint. The representative AWS proof still needs a fresh Graviton run to capture p50/p95/p99 latency, recall, and pooled-vs-unpooled profile evidence.

