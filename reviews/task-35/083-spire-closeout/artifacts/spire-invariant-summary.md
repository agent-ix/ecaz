# SPIRE Invariant Summary

## Active Epoch Chain

The recurring SPIRE safety chain is:

1. A live SPIRE `index_relation` is supplied by PostgreSQL AM callbacks, SQL diagnostic wrappers, CustomScan planner/executor callbacks, or relation guards.
2. `page::read_root_control_page(index_relation)` pins and validates the root/control page before decoding `SpireRootControlState`.
3. A nonzero `root_control.active_epoch` names the active epoch manifest, object manifest, placement directory, and local-store config tuple IDs.
4. The manifest loaders decode owned bytes from relation-backed object tuples and validate the active epoch through `SpirePublishedEpochSnapshot` / `SpireValidatedEpochSnapshot`.
5. Placement-directory validation binds placements to local or remote object stores before relation-backed object reads, fanout, heap-rerank, vacuum, or publish paths consume them.

Packets establishing this chain include 048, 054-057, 074-075, 078-082.

## Lock And WAL Summary

- Root/control and page-object reads delegate pin/validation and buffer lifetime to `src/am/ec_spire/page.rs`.
- SPIRE page initialization and metadata updates use the shared metadata initializer / buffer-lock protocol named in packet 078.
- Replacement epoch publication is serialized by the publish lock around root/control reads, placement writes, manifest construction, and `publish_replacement_epoch_to_relation`.
- Vacuum compaction and delete-delta publication hold the publish lock while opening write-capable relation stores and publishing replacement manifests.
- Relation-backed object reads and writes use `HeapRelationGuard`, `IndexRelationGuard`, scan guards, snapshot guards, slot guards, and pinned-buffer guards to scope PostgreSQL resources.

## CustomScan And DML Summary

- Plan-private and custom expression lists are offset-encoded PostgreSQL planner lists. Packets 043, 047, 049, 050, and 080 name the field at each offset as a drift tripwire.
- Planner hook and executor callback comments consistently bind live PostgreSQL planner/executor pointers to the callback duration.
- DML frontdoor primitive-plan decoding uses NodeTag checks, bounded coercion-wrapper recursion, guarded heap/index relation access, and immediate C-string copying.

## Distributed Coordination Summary

- Remote candidate fanout, scan output, dispatch, endpoint identity, operator, pipeline, libpq plan, and receive paths all chain the same active-epoch placement directory to node identity and candidate stream handling.
- Remote placement availability is treated as planner eligibility only; executor paths still validate identity, manifests, and placement ownership before merging result streams.

## RAII Guard Inventory

- `HeapRelationGuard` and `IndexRelationGuard`: heap/index relation opening, relcache field reads, placement catalog scans, heap-rerank relation fallback.
- `ActiveSnapshotGuard`: catalog probes and placement existence checks.
- `IndexScanGuard`: bounded SPIRE placement index probes.
- `TupleTableSlotGuard`: heap tuple slot allocation and SPIRE placement scans.
- `PinnedBufferGuard`: PG18 read-stream buffer handoff in heap-rerank prefetch.
- Publish lock guard: serializes SPIRE replacement epoch publication; packet 070 documents the OID-copy release pattern.

## Deferred Task 50 Candidates

- `ActiveEpochAnchor`: typed wrapper for `(index_relation, root_control, manifest set, placement directory, local_store_config)`.
- `SpireCustomPrivate`: typed round-trip wrapper for CustomScan plan-private offsets.
- Composite heap/index relation guard for DML primary-key resolution.
- NodeTag-dispatched expression decoder helper for plan_private and dml_frontdoor.
- Cross-AM callback wrapper macro for AM callbacks that enter through `pgrx_extern_c_guard`.
