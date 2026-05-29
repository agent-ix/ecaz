# Review Request: SPIRE Representative Order Gate

Task: `plan/tasks/task30-phase13e-spire-aws-production-gap-closure.md`

Code commit: `09f3265bf`

## Summary

This checkpoint tightens the local representative-performance preflight so the
AWS pass cannot reorder the user-prioritized latency/recall/pooling path:

- `pass-representative-performance-body` must run:
  `preflight-representative-performance -> provision -> install-extension -> verify-representative-performance-tunneled`;
- `verify-representative-performance-tunneled` must run:
  `with-ssm-port-forwards.sh -> load-representative -> register-representative -> smoke-representative -> bench-representative-priority -> bench-representative-pooling -> summarize-representative-performance -> verify-representative-performance-summary`;
- existing preflight checks still require the performance pass to exclude
  `fault-*` reruns.

No AWS was started.

## Validation

- `bash -n scripts/spire-aws/preflight-representative-performance.sh`
- `bash scripts/spire-aws/preflight-representative-performance.sh`
- `git diff --check 09f3265bf^ 09f3265bf -- scripts/spire-aws/preflight-representative-performance.sh`

## Artifacts

- `artifacts/manifest.md`
- `artifacts/git-show-stat.log`
- `artifacts/bash-syntax.log`
- `artifacts/preflight-representative-performance.log`

The existing untracked dry-run manifest at
`scripts/spire-aws/artifacts/representative-pooling/suite-manifest.json` was
left in place and was not staged.
