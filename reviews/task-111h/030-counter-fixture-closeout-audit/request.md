# Review Request: Task 111h Counter and Fixture Closeout Audit

This packet requests review for a read-only closeout audit over Task 111h
counter/admin and PG18 fixture checklist rows.

No code changed and no new tests were run. The packet adds:

- `artifacts/counter-fixture-closeout-audit.md`
- `artifacts/manifest.md`

Main readout:

- The EXPLAIN/admin/counter checklist row is covered for the IVF scan/admin
  surfaces used by Task 111h. Placement, format, payload bytes, source bytes,
  group page reads, metadata bytes, decode time, score time, and slab-copy bytes
  are exposed through EXPLAIN and the PG18 debug counter snapshot.
- The PG18 fixture row is only partially closed. Existing fixtures cover
  create/build, live insert, delete/vacuum, mixed direct-pointer/full-chain
  fallback, partial final groups, and no query-time source-vector conversion for
  persisted compact payloads. I did not find a dedicated update-path fixture or
  an explicit MVCC/snapshot-visible rerank payload fixture, so that task row
  should remain open for those two cases.
- The owned-copy/double-copy checklist row remains open. The current batched
  compact path still materializes survivor payloads into `payload_slab`, and the
  rabitq4 PG18 fixture intentionally asserts that copy cost is visible.

Please review whether the audit correctly maps existing committed evidence to
the Task 111h checklist and whether the remaining gaps are stated narrowly
enough.

Non-claim: this packet is not a final Task 111h closeout. It does not add
legacy `0x2A` benchmark evidence, cold/remote benchmark evidence, table-owned
compact storage evidence, update/MVCC snapshot fixtures, or a copy-cost fix.
