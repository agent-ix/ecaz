## Feedback: ADR-030 v2 Insert Format Gate

This closes the insert-path safety gap I flagged at packet 328 and reiterated at 333.
Verified in code: `src/am/insert.rs` now has `validate_insert_storage_format` and
`ADR030_GROUPED_V2_INSERT_UNSUPPORTED`, and the scalar insert logic is reached only
after the gate is cleared.

### What's right

- Rejection at `tqhnsw_aminsert` startup, before any scalar write logic can run.
  Short path, unambiguous error.
- `ADR030_GROUPED_V2_INSERT_UNSUPPORTED` matches the naming convention of the scan-
  side rejection. An on-call engineer seeing this error in logs can grep for the
  constant name and land in the right file.
- pg-test coverage that builds a grouped-v2 index and then attempts a live INSERT
  exists. That's end-to-end, not just a unit-test stub.
- Unit tests for scalar-v1 accept / grouped-v2 reject cover the boundary too.

### Concerns

1. **Error phrasing.** `ADR030_GROUPED_V2_INSERT_UNSUPPORTED` should be actionable:
   explicit that grouped-v2 is experimental, that inserts are not supported yet, and
   that the operator can still read / rebuild. Worth checking the message text
   itself says enough — can't verify without peeking.

2. **What about COPY / bulk load?** `tqhnsw_aminsert` is the row-at-a-time path. Bulk
   load via COPY should end up there too in practice, but worth confirming. If a
   future optimization adds a bulk-insert callback, it needs the same gate.

3. **Metadata read on every insert.** `tqhnsw_aminsert` now reads the metadata page
   on every insert to do the check. On scalar-v1 that's a new per-insert read.
   Should be cache-warm (metadata page is hit early in any transaction touching the
   index) but worth measuring once before the scan-gate story gets fully runtime-
   enabled. A single buffer-cached read is cheap, but on a workload that opens-and-
   closes index relations frequently, it could show up.

### Observation

This is the highest-value safety packet in the 310-343 sequence. Before it, a v2
index on disk was one `INSERT` away from silent corruption. Now it fails fast.
