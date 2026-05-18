# Manifest: Task 41 Invariant #2 Memory Lifetime Strategy

- head SHA: `7fef354f16ebd44eca63700ff666ee4b479ac189`
- task bucket and packet path:
  `reviews/task-41/121-invariant2-memory-lifetime-strategy/`
- timestamp: `2026-05-18T02:59:01Z`
- lane / fixture / storage format / rerank mode: source inventory and strategy;
  no SQL fixture, storage-format matrix, or rerank-mode execution.
- isolated one-index-per-table or shared-table surfaces: not applicable; this
  is a strategy packet.

## Artifacts

### strategy.md

- command used: synthesized from Task 41 objective, current packets 114-120,
  and the inventories below.
- key result lines:
  - high-level model covers Datum/varlena, tuple-slot Datum, buffer/page bytes,
    palloc scan-state arrays, and C strings.
  - ground-level plan sequences Phase A through Phase E with local packet
    boundaries and stop conditions.

### detoast-inventory.log

- command used:
  `rg -n "pg_detoast_datum(_packed)?\\(|varlena_to_byte_slice" src/am src/lib.rs -g '*.rs'`
- key result lines:
  - `13` lines after packets 114-120.
  - Remaining non-guard candidate is `src/lib.rs:848` `ecvector_typmod_in`;
    AM entries are now guard-internal calls in local detoast wrappers.

### raw-slice-inventory.log

- command used:
  `rg -n "from_raw_parts\\(|from_raw_parts_mut\\(" src/am src/lib.rs src/storage -g '*.rs'`
- key result lines:
  - `74` lines.
  - Dominant clusters are page/buffer byte views, scan opaque palloc arrays,
    DSM/build-parallel arrays, and already-guarded HNSW source views.

### slot-datum-inventory.log

- command used:
  `rg -n "tts_values|tts_isnull|slot_getsomeattrs_int|ExecClearTuple|TupleTableSlot" src/am src/storage -g '*.rs'`
- key result lines:
  - `111` lines.
  - Main review clusters are HNSW source helpers, DiskANN scan state, SPIRE scan
    relation, and CustomScan tuple output writers.

### palloc-inventory.log

- command used:
  `rg -n "palloc\\(|palloc0\\(|pfree\\(|PgMemoryContexts|CurrentMemoryContext|MemoryContext" src/am src/lib.rs src/storage -g '*.rs'`
- key result lines:
  - `89` lines.
  - Main review clusters are scan opaque allocations, option/C-string frees,
    AM build/vacuum stats, and CustomScan state.
