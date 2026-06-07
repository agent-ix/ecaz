# Task 85 Cloud SSM Observability Timeout Manifest

- Packet: `reviews/task-85/002-cloud-ssm-observability-timeout/`
- Branch: `task-85-spire-product-scale-pareto`
- Code surface: `crates/ecaz-cloud/src/ssm.rs`

## Change

`ssm::run_shell` now prints the SSM command id after `send-command` succeeds,
and the local AWS CLI invocations for both `send-command` and
`get-command-invocation` are bounded by a 60-second timeout.

`cloud bench` also uploads suite configs larger than 6KB to S3 instead of
embedding them in the SSM shell heredoc. The Task 85 baseline suite is 9.3KB,
and the diagnosed retry showed the remote shell blocked at the inline `cat`
step.

## Validation

- `cargo-fmt-check-ecaz-cloud.log`
  - Command: `cargo fmt --check -p ecaz-cloud`
  - Result: passed.
- `cargo-test-ecaz-cloud.log`
  - Command: `cargo test -p ecaz-cloud`
  - Result: passed, 11 tests.
