# Review request — Task 165 004-P1: transport fault drills

**Branch:** `task-165-ec-distann-m3`. First real coverage of the reviewer's
004-P1 transport fault matrix on the AM scan path (not just the endpoint).

## What landed

`fault-drills.sql` drives the real `ORDER BY <#> LIMIT k` scan (which expands
node-1-owned vec_ids over the transport) under injected faults, asserting the
NFR-020 "error-or-identical-to-baseline" contract:

- **connection_reset** (node-1 unreachable port) → ERROR `[EC_INTERNAL]`, fail closed.
- **missing_remote_target** (node-1 nonexistent database) → ERROR `[EC_INTERNAL]`.
- **no-false-reject**: a session epoch bump without content divergence returns
  the baseline result — the FR-082 fingerprint is content-based, so it must not
  falsely reject. A genuine content-divergent epoch_mismatch is covered at the
  endpoint by the pg_test fault drill (`[EC_EPOCH_MISMATCH]`, retriable).
- **recovery**: restoring the good roster returns the baseline 10 rows.

## Evidence (`artifacts/fault-drills.log`, release, v3 p3_idx)

Baseline 10 → fault ERROR → fault ERROR → no-false-reject 10 → recovery 10.

## Remaining 004-P1 (honest scope)

This covers the connection/reachability + epoch cases + recovery. The full
TC-042 matrix still needs `remote_statement_timeout`,
`remote_backend_termination`, `simulated_network_partition`, and
`hop_round_failure_mid_beam` on a true 3-worker fixture, plus the FR-082
build/publish/retire lifecycle and epoch-swap-under-load. Those need a real
multi-instance harness (not same-instance loopback) and land next.

## Ask

Review the fail-closed contract on the scan path and the drill coverage. Not
closing the request.
