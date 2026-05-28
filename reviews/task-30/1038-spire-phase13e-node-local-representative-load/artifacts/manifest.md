# Artifact Manifest: 1038 Node-Local Representative Load

- head SHA: `2020771db`
- task bucket: `reviews/task-30/1038-spire-phase13e-node-local-representative-load`
- lane: local pre-AWS harness hardening
- fixture: representative AWS pass harness, no EC2 provisioning
- storage format: unchanged; representative pass uses `SPIRE_AWS_STORAGE_FORMAT` default `rabitq`
- rerank mode: unchanged from suite configuration
- isolated/shared surface: not applicable; this packet validates the AWS load harness before provisioning

## Artifacts

### `bash-n.log`

- command: `bash -n scripts/spire-aws/load.sh scripts/spire-aws/preflight-representative-performance.sh`
- timestamp: 2026-05-27T17:30:xx-07:00
- result: exit code 0

### `representative-preflight.log`

- command: `ARTIFACT_DIR=reviews/task-30/1038-spire-phase13e-node-local-representative-load/artifacts scripts/spire-aws/preflight-representative-performance.sh`
- timestamp: 2026-05-27T17:30:xx-07:00
- key result: `SPIRE representative performance preflight passed`
- coverage: preflight now guards the node-local coordinator/remote load path and post-load tunnel restart guard.

### `cargo-build-ecaz-cli.log`

- command: `cargo build --bin ecaz --package ecaz-cli`
- timestamp: 2026-05-27T17:23:06-07:00
- key result: `Finished dev profile`
- note: one pre-existing `dead_code` warning for `LoadedDistributedPlacementConfig.path`

### `git-diff-check.log`

- command: `git diff --check`
- timestamp: 2026-05-27T17:30:xx-07:00
- result: exit code 0
