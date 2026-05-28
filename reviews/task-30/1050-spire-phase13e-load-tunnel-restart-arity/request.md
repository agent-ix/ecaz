# Review Request: Load Tunnel Restart Arity

## Summary

This packet covers commit `16bde343bdc2f563f491f8f98d2880974dea987b`
(`Fix representative load tunnel restart args`).

The failed representative AWS attempt in packet `1049` did not fail inside
SPIRE build/scan. The coordinator node-local real-corpus load completed, then
the harness failed while restarting the SSM port forward:

`restart-ssm-port-forward.sh: line 10: 3: local port required`

The fix makes `scripts/spire-aws/load.sh` pass the required restart arguments:
`label instance_id port artifact_dir`. It applies both to the coordinator
restart after node-local load and to the all-node restart after remote loads.

The representative preflight now requires and runs
`scripts/spire-aws/check-load-tunnel-restart-local.sh`, which statically guards
this exact call shape before any future AWS provision.

## Evidence

Artifact manifest: `artifacts/manifest.md`

Key local validation:

- `artifacts/bash-n.log`: syntax passed for `load.sh`,
  `preflight-representative-performance.sh`,
  `check-load-tunnel-restart-local.sh`, and `restart-ssm-port-forward.sh`.
- `artifacts/load-tunnel-restart-local.log`: local guard passed.
- `artifacts/preflight-representative-performance.log`: representative
  preflight passed with the new guard wired in.
- `artifacts/representative-pass-dry-run.log`: the standard representative
  pass dry-run passed operator/state/permission/representative preflight and
  did not provision.
- `artifacts/aws-running-after-local-gate.log`: no pending/running/stopping EC2
  instances remained in `us-west-2`.

## Reviewer Focus

Please check the argument wiring in `load.sh`: the restart helper should always
receive a concrete `instance_id` from the topology, not infer it from the
operator port or label.
