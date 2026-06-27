# Table-Owned Compact Payload Audit

## Conclusion

`rerank_placement = 'table'` is not viable as a Task 111h product path without a
new PostgreSQL storage surface. The replacement for 111h is:

- exact table/source storage: `rerank_placement = 'source'`,
  `rerank_format = 'f32'`;
- compact persisted storage: `rerank_placement = 'index'` with packed rerank
  groups for f16, RaBitQ-4, RaBitQ-8, and TurboQuant;
- reserved future surface: `rerank_placement = 'table'`, still rejected until a
  separate DDL, MVCC, maintenance, and benchmark plan exists.

This is not a claim that PostgreSQL cannot store compact payloads. It is a claim
that the current index AM cannot honestly expose `table` as a maintained compact
payload placement in Task 111h.

## Current Code Path

The option resolver accepts the spelling `table`, but rejects it before an index
can be built:

- `src/am/ec_ivf/options.rs`: `Auto` resolves f32 to `source` and compact formats
  to `index`.
- `src/am/ec_ivf/options.rs`: `source` only supports f32; compact source formats
  are rejected except through explicit `source_diagnostic`.
- `src/am/ec_ivf/options.rs`: `table` errors with the message that real
  table-owned persisted rerank payloads are not implemented.

The build and insert payload writers are index-only:

- `src/am/ec_ivf/build.rs`: `build_rerank_group_chain` returns no sidecar unless
  `options.rerank_placement == RerankPlacement::Index`.
- `src/am/ec_ivf/insert.rs`: `append_rerank_group_entry` returns no sidecar unless
  `reloptions.rerank_placement == RerankPlacement::Index`.

The scan path has no table-owned compact fetch:

- `src/am/ec_ivf/scan.rs`: when an index group head exists, scan calls
  `rerank_probe_candidates_index_side` and reads packed index groups.
- `src/am/ec_ivf/scan.rs`: otherwise scan calls
  `rerank_probe_candidates_source_side`, fetches the heap row under the scan
  snapshot, reads the existing f32 source vector, and records
  `dimensions * 4` source bytes per reranked candidate.
- `source_diagnostic` is only a scoring mode over fetched f32 source vectors; it
  is not persisted compact table storage.

## Why A 111h Table Placement Is A Concrete Blocker

A real table-owned compact payload would need one of these storage shapes. None
fits the current AM reloption without new product architecture.

### Heap Column Or Generated/Stored Column

A heap column or generated/stored column would be real PostgreSQL table-owned
storage, but an index reloption cannot silently add or maintain a user-table
column. That route requires explicit DDL, type/expression semantics, storage
accounting, migration behavior, and scan code that reads the payload column under
the same snapshot as the candidate tuple. It also adds storage in the table
while the source vector remains the canonical data.

### Companion Table Keyed By Logical Id

The historical sidecar harness uses tables shaped like:

```sql
CREATE UNLOGGED TABLE sidecar (
  id bigint PRIMARY KEY,
  payload bytea NOT NULL
);
```

That is useful for measurement, but the IVF scan frontier contains heap TIDs.
A logical-id companion table either requires an extra heap fetch to discover the
logical id or requires duplicating that id into index postings. It also needs
triggers or another write path to keep INSERT/UPDATE/DELETE versions consistent.
The current AM has no such relation ownership or maintenance path.

### Companion Table Keyed By Heap TID

A heap-TID companion table avoids the logical-id lookup, but heap TIDs are tuple
versions, not logical rows. UPDATE creates new tuple versions, DELETE/VACUUM
must clean old versions, and snapshot-visible rerank payloads must match the
candidate tuple version. That is a new MVCC and vacuum integration problem, not a
drop-in replacement for the current source or index paths.

### Index AM Writes To A Second Relation

Writing a companion relation from `ambuild`, `aminsert`, and vacuum callbacks
would require a new locking, WAL, transaction, naming, privilege, and cleanup
design. It would also need benchmark counters for table payload bytes/blocks,
plus correctness fixtures across build, insert, update, delete, vacuum, and
rebuild. None of that exists in the current code path.

## Existing Companion-Table Measurements

`crates/ecaz-cli/src/commands/bench/sidecar_rerank.rs` is explicitly scoped as a
measurement harness, not an index feature. It asks a `rerank=off` IVF index for
candidate ids, then reranks client-side with f32, f16, or RaBitQ-8 payloads.

The real-I/O packet `benchmarks/task51-local-ivf-sidecar-real-io/` measured
separate fixed-width `bytea` sidecar tables on a local 50k smoke fixture:

- random-id lookup: about `16.654 ms` to `18.293 ms` p50 sidecar I/O for 50
  candidates;
- TID-sorted lookup: about `0.885 ms` to `1.403 ms` p50 sidecar I/O for 50
  candidates;
- f16 companion table total size: `197 MB`;
- RaBitQ-8 companion table total size: `79 MB`.

The same manifest lists the important caveats: local PG18/WSL2 only, q=100,
static corpus snapshot, sidecar table built in corpus/id order, no concurrent
insert/update/delete churn, sidecar tables fit in OS cache, and no product
heap-TID frontier or table-maintenance design.

This evidence rejects naive random-id companion-table lookup and shows a
TID-sorted companion table could be a future storage project. It does not
implement or validate `rerank_placement = 'table'` for Task 111h.

## Replacement Decision For 111h

Task 111h should keep two implemented product surfaces:

- `source/f32`: exact baseline over the existing source vector. Packet 029 shows
  it remains the warm-cache matched-recall reference at 50k, 100k, and 1M.
- `index/{f16,rabitq4,rabitq8,turboquant}`: compact persisted payloads through
  the common rerank payload codec and packed group/segment layout.

`table` remains reserved to prevent a repeat of the misleading 111g label. A
future table-owned compact payload project should start with a DDL/MVCC design
and its own suite matrix instead of being folded into the 111h reloption.
