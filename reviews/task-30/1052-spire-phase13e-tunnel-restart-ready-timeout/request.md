# Review Request: Phase 13e Tunnel Restart Readiness

## Summary

The representative AWS rerun reached the real corpus and coordinator build path, but failed before remote shard load because the coordinator SSM tunnel restart logic did not tolerate slow Session Manager readiness:

- real corpus prepared as `ec_real_100k` with 100,000 corpus rows and 1,000 query rows
- coordinator node-local load succeeded and built `ec_spire_aws_repr_1m_idx` in 134.58s
- failure occurred after coordinator load during tunnel restart, before remote load/register/smoke/bench
- watchdog teardown completed and an independent EC2 check shows no pending/running/stopping instances

This change fixes the harness, not SPIRE core scan code:

- starts SSM tunnel processes in their own process group when `setsid` is available
- cleanup/restart now kills the tunnel process group instead of only one parent PID
- restart waits the full configured readiness timeout for a slow SSM startup instead of retrying every 10 seconds and leaving late listeners behind
- local self-check now includes a fake slow SSM startup that only becomes ready after 12 seconds

## Evidence

See `artifacts/manifest.md` for the full artifact list.

Key local gates:

- `artifacts/load-tunnel-restart-local.log`: tunnel restart self-check passed
- `artifacts/representative-pass-dry-run.log`: Graviton representative preflight passed without provisioning
- `artifacts/aws-running-after-local-gate.log`: no pending/running/stopping EC2 instances after local gate

Key AWS failure classification:

- `artifacts/aws-failure/coordinator-load-representative.log`: coordinator real-corpus load/build succeeded
- `artifacts/aws-failure/tunnel-restart-after-node-local-load.log`: restart readiness timeout followed by local port bind conflicts
- `artifacts/aws-failure/aws-running-after-failure.log`: no pending/running/stopping EC2 instances after teardown

## Review Focus

- Check that process-group cleanup cannot kill the parent harness shell.
- Check that the new restart wait behavior avoids the observed late-listener race.
- Check that the local fake-SSM regression is narrow and does not call AWS.
