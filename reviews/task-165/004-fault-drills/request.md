# Review request — Task 165 M3 slice 4: fault-drill matrix (TC-042 / NFR-020)

**Branch:** `task-165-ec-distann-m3`. Fourth M3 slice.

## What landed

`test_ec_distann_fault_drills_distinct_classes` — the endpoint-level fault
drills, each asserting the NFR-020 bar (an ERROR carrying a distinct
machine-readable `[EC_*]` class, never a wrong or silent result):

- **epoch_mismatch** → `[EC_EPOCH_MISMATCH]` (retriable, SQLSTATE 40001).
- **missing_node_record** (owned-but-absent structural fault) → `[EC_RECORD_MISSING]`.
- **bad input** (malformed fingerprint width) → `[EC_BAD_INPUT]`.
- **placement_drift** (non-owned id under a 2-node roster) → `[EC_PLACEMENT]`.

These are the read-endpoint drills testable single-node in a pg_test txn; the
transport-level cases (connection_reset_mid_batch, remote_statement_timeout,
remote_backend_termination, simulated_network_partition,
hop_round_failure_mid_beam) run against the committed loopback fixture and land
with the multinode fault fixture + 50k recall bench.

## Evidence (`artifacts/test-evidence.log`)

`fault_drills_distinct_classes` green (part of the 90+ distann pg_test suite).

## Remaining M3

FR-082 3-worker lifecycle (epoch manifest/publish/retire); transport-level fault
drills on the loopback fixture; delta drain at epoch build (FR-083-AC-2); 50k
multinode distinct_recall bench.

## Ask

Review the fault-class taxonomy and the drill coverage. Not closing the request.
