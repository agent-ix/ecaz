# Review request — Task 165 M3 slice 3: FR-083 write endpoint

**Branch:** `task-165-ec-distann-m3`. Third M3 slice.

## What landed

`ec_distann_apply_record_writes(index_regclass, epoch_fingerprint, tombstone_vec_ids)`
— the FR-083 remote write endpoint (write counterpart to FR-079's
`ec_distann_expand_nodes`). This slice implements the **tombstone-set**
operation: the coordinator routes a delete to the hash-owning node, which
tombstones the records it owns. Validates the caller's epoch fingerprint
(retriable mismatch, distinct SQLSTATE) and every vec_id's ownership (placement
error) before any write — the same FR-079/FR-082 pre-flight as the read
endpoint — then applies `tombstone_by_vec_ids`.

New-record append + back-edge amendment (the M5 incremental-insert operations)
are deferred; the endpoint signature is the tombstone slice of that surface.

## Evidence (`artifacts/test-evidence.log`)

`test_ec_distann_apply_record_writes_tombstones`: tombstones 2 records via the
endpoint under a valid fingerprint; a wrong fingerprint yields the retriable
epoch-mismatch error (FR-082-AC-2). **90 pg_tests pass, 0 failed**; clippy clean.

## Ask

Review the write endpoint's epoch/ownership pre-flight and the tombstone
application. Not closing the request.
