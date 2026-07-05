# Manifest

- head SHA: `16bde343bdc2f563f491f8f98d2880974dea987b`
- task bucket: `reviews/task-30`
- packet: `reviews/task-30/1050-spire-phase13e-load-tunnel-restart-arity`
- timestamp: `2026-05-28T15:57:16Z`
- lane: Phase 13e representative AWS performance preflight
- fixture: local/static harness gate plus dry-run on established Graviton operator config
- storage format: SPIRE distributed remote placements
- rerank mode: production read profile suites; dry-run only, no provisioning
- isolated one-index-per-table vs shared-table surface: one-index-per-table representative SPIRE surface

## Artifacts

| Artifact | Command | Key result |
| --- | --- | --- |
| `bash-n.log` | `bash -n scripts/spire-aws/load.sh scripts/spire-aws/preflight-representative-performance.sh scripts/spire-aws/check-load-tunnel-restart-local.sh scripts/spire-aws/restart-ssm-port-forward.sh` | shell syntax passed |
| `load-tunnel-restart-local.log` | `scripts/spire-aws/check-load-tunnel-restart-local.sh` | `SPIRE AWS load tunnel restart local self-check passed` |
| `preflight-representative-performance.log` | `scripts/spire-aws/preflight-representative-performance.sh` | `SPIRE representative performance preflight passed` |
| `representative-pass-dry-run.log` | `scripts/spire-aws/run-representative-performance-pass.sh --artifact-dir reviews/task-30/1050-spire-phase13e-load-tunnel-restart-arity/artifacts` | refreshed `auto_stop_at`, passed operator/state/permission/representative preflights, and stopped before provisioning |
| `aws-running-after-local-gate.log` | EC2 pending/running/stopping inventory check in `us-west-2` | no rows |

## Context

The preceding AWS attempt in packet `1049` reached real representative corpus
load on the coordinator, then failed before remote loads because
`load.sh` called `restart-ssm-port-forward.sh` as `label port`. The restart
script requires `label instance_id port artifact_dir`.

This packet fixes that call shape for the coordinator post-load restart and for
the all-node restart after remote loads. The representative preflight now runs
the local static guard before any future AWS provision.
