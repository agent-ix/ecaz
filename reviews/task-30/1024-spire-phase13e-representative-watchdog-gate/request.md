# Review Request: SPIRE Representative Watchdog Gate

Task: `plan/tasks/task30-phase13e-spire-aws-production-gap-closure.md`

Code commit: `b21c718c9`

## Summary

This checkpoint tightens the local representative-performance preflight before
any AWS representative run:

- verifies `pass-representative-performance` routes through
  `run-pass-with-watchdog.sh pass-representative-performance-body`;
- verifies the representative-performance watchdog default timeout is at least
  `14400` seconds;
- keeps the existing checks that the representative performance pass runs
  preflight before provision and excludes `fault-*` reruns;
- adds an embedded negative self-check proving the preflight rejects a watchdog
  file whose representative-performance timeout is too short.

This is local-only hardening for the user-prioritized latency/recall/pooling
AWS pass. No AWS was started.

## Validation

- `bash -n scripts/spire-aws/preflight-representative-performance.sh`
- `bash scripts/spire-aws/preflight-representative-performance.sh`
- `git diff --check b21c718c9^ b21c718c9 -- scripts/spire-aws/preflight-representative-performance.sh`

## Artifacts

- `artifacts/manifest.md`
- `artifacts/git-show-stat.log`
- `artifacts/bash-syntax.log`
- `artifacts/preflight-representative-performance.log`

The existing untracked dry-run manifest at
`scripts/spire-aws/artifacts/representative-pooling/suite-manifest.json` was
left in place and was not staged.
