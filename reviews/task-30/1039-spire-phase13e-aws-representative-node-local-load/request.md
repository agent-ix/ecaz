# Task 30 Review Request: AWS Representative Node-Local Load Failure

## Summary

This packet records the first AWS representative run after switching heavy representative loads to node-local SSM execution. The run used the established Graviton lane in `us-west-2a` with one coordinator and three remotes on `m7g.large`.

The previous page-size failure did not recur. The run failed earlier in the node-local coordinator load setup while downloading the 2.0 GiB representative corpus from S3 to the node staging directory:

```text
download failed ... [Errno 28] No space left on device
failed to run commands: exit status 1
```

The harness teardown completed cleanly at `2026-05-28T02:02:03Z`, and a direct EC2 verification after teardown returned no pending/running/stopping/stopped instances in `us-west-2`.

## Evidence

- `artifacts/run-representative-performance-pass.log`: full AWS pass transcript.
- `artifacts/coordinator-load-representative.ssm.json`: failed SSM invocation payload.
- `artifacts/coordinator-load-representative-error.log`: extracted root error.
- `artifacts/ec2-post-teardown-verify.log`: direct post-teardown EC2 verification.
- `artifacts/aws-pass-watchdog.log`: watchdog/teardown evidence.

## Follow-Up

The follow-up code fix is in packet `1040-spire-phase13e-node-local-staging-fix`: move large node-local staging off `/tmp`, grow the root filesystem, and clean build staging after installing the node-local CLI.
