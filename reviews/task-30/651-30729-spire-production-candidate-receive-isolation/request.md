# Review Request: SPIRE Production Candidate Receive Isolation

- code commit: `1ad44e4264a9f6fdf8c37ebb534fc91b915611bb`
- reviewer focus: packet 30727 P2 receive-side isolation before C5 AM scan integration
- phase: Phase 11 Stage C, C1 production fanout executor

## Summary

This checkpoint closes packet 30727 P2 by adding a multi-node PG18 receive
fixture. The fixture sends one healthy loopback `rabitq` remote together with
failed receive requests in the same adapter batch, then asserts that:

- the ready remote returns exactly one candidate batch;
- every candidate in the ready batch remains scoped to the ready node;
- failed remotes return no candidate batch and no candidate rows;
- each failed row preserves its own failure category.

The production candidate-receive failure helper now reports
`remote_candidate_receive_failed` instead of reusing
`remote_transport_failed`, so adapter row status matches the executor stage
that consumed the failure.

## Failure Coverage

The test covers the triggerable receive failure categories in one fanout batch:

- `candidate_invalid_parameters`
- `conninfo_parse_failed`
- `connect_failed`
- `remote_query_failed`
- `candidate_decode_failed`
- `candidate_batch_validation_failed`

`statement_timeout_setup_failed` is not forced in this fixture because the
adapter reads statement-timeout behavior from the shared session GUC; forcing
that setup failure would also break the ready loopback request and would not
exercise per-node isolation.

## Validation

Artifacts are packet-local under `artifacts/`.

- `cargo fmt --check`
  - log: `artifacts/cargo-fmt-check.log`
  - result: pass, with existing rustfmt stable-channel warnings.
- `cargo check --no-default-features --features pg18`
  - log: `artifacts/cargo-check-pg18.log`
  - result: pass.
- `cargo pgrx test pg18 test_ec_spire_prod_receive_isolates_node_failures`
  - log: `artifacts/cargo-pgrx-test-receive-isolation.log`
  - result: `1 passed; 0 failed`.
- `git diff 1961ec9c5b8c1bed11a152ba6930d80c367e340e 1ad44e4264a9f6fdf8c37ebb534fc91b915611bb --check`
  - log: `artifacts/git-diff-check.log`
  - result: pass.

## Requested Review

Please check whether this is sufficient to unblock C5 AM scan integration:

- Is the receive-isolation fixture broad enough for packet 30727 P2?
- Is changing failed candidate receive rows to
  `remote_candidate_receive_failed` the right stage-level status?
- Is the statement-timeout setup caveat acceptable here, or should that get a
  separate non-isolation test?
