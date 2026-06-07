# Task 85 Review Request: Cloud SSM Observability Timeout

## Summary

This checkpoint fixes the immediate Task 85 AWS baseline blocker from packet
001: `ecaz cloud install` and `ecaz cloud bench` could sit in an opaque local
wait with no fresh visible SSM invocation and no command id printed.

The code change is intentionally narrow:

- bound each local AWS CLI SSM call (`send-command` and
  `get-command-invocation`) with a 60-second timeout;
- print the SSM `command_id` immediately after a successful send, before the
  long poll begins.

This does not change benchmark semantics or remote scripts. It only makes cloud
operations fail/diagnose locally instead of silently waiting forever.

## Validation

- `cargo fmt --check -p ecaz-cloud`: passed.
- `cargo test -p ecaz-cloud`: passed, 10 tests.

## Requested Review

Please review whether this is the right minimal cloud-wrapper fix before
retrying the Task 85 AWS 1M/q500 baseline suite from packet 001.
