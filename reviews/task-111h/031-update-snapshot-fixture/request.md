# Review Request: Task 111h Update/Snapshot Rerank Payload Fixture

This packet requests review for commit `ad518f14a`, which adds a PG18
correctness fixture for update-path and snapshot-visible compact rerank payloads.

Code changed:

- `src/tests/ec_ivf.rs`

The new fixture
`test_ec_ivf_index_placement_update_snapshot_payload` covers the remaining
Task 111h correctness gap called out in packet 030:

- creates an `index` placement `coarse_rerank` f16 IVF index,
- verifies the SQL plan uses the IVF index,
- declares an index-ordered cursor before an `UPDATE`,
- updates row `id = 0` from `[1,0]` to `[0,1]`,
- verifies the pre-update cursor snapshot still ranks the old tuple/payload,
- verifies a fresh old-query snapshot no longer sees the old `id = 0` version,
- verifies a fresh new-query snapshot ranks the updated tuple from its new
  compact payload,
- verifies the index-side f16 counter path still reads zero heap source-vector
  bytes and scores `dims*2` compact payload bytes.

Validation:

- Initial attempt failed before test execution because the test identifier was
  64 characters and pgrx rejected PostgreSQL identifier truncation:
  `artifacts/cargo-pgrx-test-pg18-update-snapshot.log`.
- Focused rerun passed:
  `artifacts/cargo-pgrx-test-pg18-update-snapshot-pass.md`.

Non-claim: this packet does not close Task 111h. It only closes the
update/snapshot fixture gap from the PG18 correctness checklist row. Remaining
111h closeout items include legacy `0x2A` baseline evidence, table-owned storage
evidence/replacement rationale, copy/slab cleanup or benchmark-away evidence,
and final cold/remote/generalization evidence.
