# Task 85 Cloud SSM Observability Timeout Manifest

- Packet: `reviews/task-85/002-cloud-ssm-observability-timeout/`
- Branch: `task-85-spire-product-scale-pareto`
- Code surface: `crates/ecaz-cloud/src/ssm.rs`

## Change

`ssm::run_shell` now prints the SSM command id after `send-command` succeeds,
and the local AWS CLI invocations for both `send-command` and
`get-command-invocation` are bounded by a 60-second timeout.

## Validation

- `cargo-fmt-check-ecaz-cloud.log`
  - Command: `cargo fmt --check -p ecaz-cloud`
  - Result: passed.
- `cargo-test-ecaz-cloud.log`
  - Command: `cargo test -p ecaz-cloud`
  - Result: passed, 10 tests.
