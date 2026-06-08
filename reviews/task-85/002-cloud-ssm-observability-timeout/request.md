# Task 85 Review Request: Cloud SSM Observability Timeout

## Summary

This checkpoint fixes the immediate Task 85 AWS baseline blockers from packet
001: `ecaz cloud install` and `ecaz cloud bench` could sit in an opaque wait,
and the first diagnosed benchmark retry showed the remote shell blocked at the
inline suite-config `cat` step.

The code change is intentionally narrow:

- bound each local AWS CLI SSM call (`send-command` and
  `get-command-invocation`) with a 60-second timeout;
- print the SSM `command_id` immediately after a successful send, before the
  long poll begins;
- lower the cloud bench inline suite-config threshold to 6KB so packet suites
  like Task 85's 9.3KB config are uploaded to S3 instead of embedded in the SSM
  shell heredoc.

This does not change benchmark semantics or remote scripts. It only makes cloud
operations fail/diagnose locally instead of silently waiting forever.

## Validation

- `cargo fmt --check -p ecaz-cloud`: passed.
- `cargo test -p ecaz-cloud`: passed, 11 tests.

## Requested Review

Please review whether this is the right minimal cloud-wrapper fix before
retrying the Task 85 AWS 1M/q500 baseline suite from packet 001.
