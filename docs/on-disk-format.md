# On-Disk Format Invariants

Task 42 tracks the invariants that keep ECAZ page bytes portable across
endianness, architecture, and extension-version changes. The current encoding
convention is little-endian for every integer and floating-point scalar persisted
inside ECAZ-owned payloads.

## Current Static Coverage

`make layout-check` compiles `tests/size_of_assertions.rs` and pins the first
set of byte-level contracts:

| Area | Covered bytes |
| --- | --- |
| Generic page storage | `ItemPointer` wire size and field offsets |
| HNSW metadata | legacy and current metadata payload sizes, all current field offsets |
| HNSW tuples | element, grouped-hot, turbo-hot, rerank, grouped-codebook, and neighbor tuple fixed offsets |
| DiskANN metadata | Vamana metadata payload size and all current field offsets |
| DiskANN tuples | Vamana node fixed header, dynamic-region offsets, and codebook tuple fixed offsets |
| IVF metadata | metadata payload size, magic, format version, and all current field offsets |
| IVF tuples | block refs, centroid, list-directory, posting, and PQ-codebook fixed offsets |
| SPIRE storage | partition-object headers, assignment rows, leaf V2 meta/segment prefixes, and partition-object V2 chain prefixes |
| SPIRE metadata | local-store configs, placement entries/directories, epoch manifests, and object manifests |

These assertions are intentionally about encoded byte layouts, not host Rust
struct layout. Most persisted structs contain `Vec` fields or are logical views
over encoded slices, so the stable contract is the codec offset table.

## Fixture Process

`make on-disk-fixtures` decodes packetized hex fixtures under
`fixtures/on-disk/` and asserts that they match the expected in-memory
metadata/tuple representations. It also byte-swaps exercised bounded fields
and asserts those mutated fixtures are rejected instead of silently decoded.

Current fixture coverage:

| Fixture | Coverage |
| --- | --- |
| `hnsw_metadata_v3.hex` | HNSW current metadata decode and swapped-version rejection |
| `hnsw_metadata_v4_rabitq.hex` | HNSW RaBitQ metadata decode and swapped-version rejection |
| `hnsw_element_tuple_v3.hex` | HNSW element tuple decode |
| `hnsw_grouped_hot_tuple_v2.hex` | HNSW grouped-hot tuple decode |
| `hnsw_turbo_hot_tuple_v3.hex` | HNSW turbo-hot tuple decode |
| `hnsw_rerank_tuple_v3.hex` | HNSW cold rerank tuple decode |
| `hnsw_neighbor_tuple_v3.hex` | HNSW neighbor tuple decode |
| `hnsw_grouped_codebook_tuple_v3.hex` | HNSW grouped-PQ codebook shard decode |
| `diskann_vamana_metadata_v3.hex` | DiskANN Vamana metadata decode and swapped-version rejection |
| `diskann_vamana_node_tuple_v3.hex` | DiskANN Vamana node tuple decode and swapped-neighbor-count rejection |
| `diskann_vamana_overflow_tuple_v3.hex` | DiskANN duplicate heap-TID overflow tuple decode and swapped-count rejection |
| `diskann_vamana_codebook_tuple_v3.hex` | DiskANN grouped-PQ codebook shard decode |
| `ivf_metadata_v4.hex` | current IVF metadata (92 bytes) decode and swapped-version rejection |
| `ivf_metadata_v3.hex` | legacy 86-byte IVF metadata, now rejected by version |
| `ivf_centroid_tuple_v1.hex` | IVF centroid tuple decode and swapped-dimension rejection |
| `ivf_list_directory_tuple_v1.hex` | IVF list-directory tuple decode |
| `ivf_posting_tuple_v1.hex` | IVF posting tuple decode |
| `ivf_pq_codebook_tuple_v1.hex` | IVF PQ-codebook shard decode |
| `spire_local_store_config_v1.hex` | SPIRE local-store config decode and swapped-version rejection |
| `spire_placement_entry_v1.hex` | SPIRE placement entry decode and swapped-version rejection |
| `spire_placement_directory_v1.hex` | SPIRE placement directory decode and swapped-version rejection |
| `spire_epoch_manifest_v1.hex` | SPIRE epoch manifest decode and swapped-version rejection |
| `spire_manifest_entry_v1.hex` | SPIRE object-manifest entry decode and swapped-version rejection |
| `spire_object_manifest_v1.hex` | SPIRE object manifest decode and swapped-version rejection |
| `spire_leaf_partition_object_v1.hex` | SPIRE leaf partition object body decode and swapped-version rejection |
| `spire_routing_root_partition_object_v1.hex` | SPIRE root routing partition object body decode |
| `spire_delta_partition_object_v1.hex` | SPIRE delta partition object body decode |
| `spire_top_graph_partition_object_v1.hex` | SPIRE top-graph partition object body decode |
| `spire_leaf_v2_meta_v2.hex` | SPIRE leaf V2 partition-object meta decode and swapped-version rejection |
| `spire_leaf_v2_segment_v2.hex` | SPIRE leaf V2 partition-object segment decode |
| `spire_partition_object_v2_chain_meta.hex` | SPIRE generic V2 chain meta decode and swapped-version rejection |
| `spire_partition_object_v2_chain_segment.hex` | SPIRE generic V2 chain segment decode |

## Version Policy

Every current metadata page carries a format-version field that readers check
before interpreting the rest of the payload:

| AM | Current tags | Reader behavior |
| --- | --- | --- |
| HNSW | `1`, `2`, `3`, `4` | accepts known tags, rejects unknown tags |
| DiskANN | `3` | accepts the DiskANN tag, rejects foreign tags |
| IVF | `4` | accepts the IVF metadata tag, rejects all other versions |
| SPIRE partition objects | `1`, `2` | accepts known object versions, rejects unknown versions |

Any incompatible field addition or reinterpretation must add a new format tag
and update the layout assertions, fixture golden files, and upgrade matrix.

## IVF Metadata Format

IVF writes and reads metadata format version `4` only; any other version
(including the legacy 86-byte v3 layout) is rejected. This is a research project
with no backward compatibility — an index written by an older format is simply
rebuilt.

The v4 metadata struct is `EC_IVF_METADATA_BYTES = 92` bytes wide.
`EC_IVF_METADATA_RERANK_SIDECAR_HEAD_OFFSET = 80` holds the head `ItemPointer` of
the compact rerank sidecar chain (tag `0x2A`). A head of `ItemPointer::INVALID`
is the legitimate "no sidecar" state for `rerank_placement = 'table'` and for
f32 storage; the scan then reranks from the heap/table source path. That is a
runtime placement state, not a compatibility mode.

`EC_IVF_METADATA_RERANK_SIDECAR_DIRECTORY_HEAD_OFFSET = 86` holds the head
`ItemPointer` of the survivor-directed rerank sidecar directory chain (ADR-079).
The directory maps the first heap TID of each `0x2A` sidecar block to that
block's TID, sorted by heap TID, so an index-side rerank reads only the sidecar
blocks that actually hold survivor payloads instead of the whole chain. A
directory head of `ItemPointer::INVALID` (or any index with rows inserted since
the last full build) falls back to the full-chain sidecar read; the directory is
written once per build over the build-sorted sidecar chain.

## IVF Posting Tuple Tags

Per-tuple tags inside IVF data pages. The surviving Task 111 dense formats are
page-local dense blocks and aligned dense blocks; Task 111g adds the compact
rerank sidecar block:

| Tag | Tuple kind | Status |
| --- | --- | --- |
| `0x21` | centroid tuple | current |
| `0x22` | list-directory tuple | current |
| `0x23` | row posting tuple | current, default mutable/delta format |
| `0x24` | PQ codebook tuple | current |
| `0x25` | dense posting block | current dense block format |
| `0x28` | aligned dense posting block | current typed-view dense block format |
| `0x2A` | rerank sidecar block | current; compact rerank rep keyed by heap TID (Task 111g) |

The `0x2A` rerank sidecar block stores a tid-keyed run of compact rerank
payloads (f16 = `dims * 2` bytes, rabitq4 = the rabitq4 payload length) for
`rerank_placement = 'index'`, chained via a per-block `next_tid`. Its on-disk
shape is
`[tag:u8][rerank_format:u8][payload_len:u16][entry_count:u16][next_tid:6]` then
`entry_count` heap TIDs and `entry_count * payload_len` payload bytes. Build
writes it globally tid-sorted; insert prepends a single-entry block (O(1)) and
repoints the metadata head; vacuum tombstones dead entries in place
(`heap_tid = INVALID`, same byte length) and REINDEX regenerates the chain.
`rerank_placement = 'table'` (and any v1/v2 index) writes no sidecar.

Task 111f removed the abandoned page-spanning packed and columnar frozen-list
formats (`0x26`, `0x27`, `0x29`) before they shipped on `main`. Their former tag
values remain reserved and must not be silently reused; any future incompatible
IVF posting shape needs a new explicit format decision, layout assertions, and
upgrade fixtures.

## Upgrade Matrix

`fixtures/upgrade/matrix.csv` is the current `(format_version, AM, can_read,
can_write)` table. `make upgrade-smoke` validates that the matrix has unique
rows, that writable formats are readable, that each row points at a committed
fixture, and that the current writable set is explicit.

HNSW currently has two writable format tags because TurboQuant/PqFastScan and
RaBitQ are selected by `storage_format` rather than by an in-place migration.
For any future incompatible replacement of an existing writable tag, the matrix
must grow a live upgrade rehearsal for the old writable version per
NFR-016-EV-3: build the old corpus, upgrade the extension, scan it with the new
reader, and record the recall floor beside the historical fixture directory.

## Cross-Arch Decode

`make endian-qemu` runs the on-disk fixture suite for the big-endian
`s390x-unknown-linux-gnu` target through `qemu-s390x`. The GitHub Actions
`endian-qemu` job installs the target, qemu user emulator, and cross linker,
then runs this make target on `main`, manual dispatch, and the nightly schedule.

The qemu lane is decode-only. It links the extension test binary but does not
execute PostgreSQL callbacks under s390x; the unresolved pgrx FFI symbols are
therefore allowed only for this target.

## PG Upgrade Smoke

`make pg-upgrade-smoke` runs the PG18 same-binary `pg_upgrade` lane through
`ecaz dev pg-upgrade-smoke`. The fixture creates an old cluster with ECAZ
installed, inserts a small `ecvector` corpus, builds an `ec_hnsw` index, checks
the pre-upgrade nearest-neighbor result, runs `pg_upgrade`, starts the upgraded
cluster, verifies the same top-2 IDs, index presence, and heap count, then runs
`pg_amcheck` against the upgraded database.

This is intentionally a narrow HNSW-only smoke today. The four-row corpus makes
the top-2 equality check a trivial recall@2 proxy, not a substantive recall
floor. Richer recall measurement and `ec_ivf` / `ec_diskann` / `ec_spire`
coverage should be added when those AMs have corpus sizing that makes the
upgrade lane load-bearing.

## WAL Format Policy

Current ECAZ page changes use PostgreSQL GenericXLog. Those WAL records carry
PostgreSQL-managed page images/deltas, not extension-owned ECAZ record bodies,
so there is no current custom WAL payload that can carry its own version byte.
The durable version contract for replayed bytes is therefore the page payload
format tag that the on-disk fixture suite and layout assertions cover.

If Task 37 adds extension-owned WAL redo/replay payloads, byte 0 is reserved as
the custom WAL record format tag. `src/storage/wal.rs` owns
`ECAZ_CUSTOM_WAL_RECORD_FORMAT_VERSION`, the byte-0 offset constant, and the
validator that rejects missing or unknown custom WAL record versions before
replay reads the body. ADR-070 keeps custom WAL records on the conservative
reject-unknown posture unless a later WAL-specific ADR justifies a different
encoding.

## Future Conditional Extensions

- Extend fixture bytes under `fixtures/on-disk/` to any raw generic page
  encoding that becomes a durable external byte contract.
- Extend byte-swapped fixture rejection tests to additional bounded multi-byte
  fields where the current decoder can reject malformed values.
- Extend static offset assertions to additional SPIRE routing/top-graph object
  body prefixes if they become durable page-buffer contracts beyond the current
  partition-object and metadata codecs.
- Extend `fixtures/upgrade/` from the current matrix into historical corpus
  directories when a new incompatible format version ships.
- Extend `pg_upgrade` smoke from the current HNSW-only top-2 equality probe to
  richer recall-floor coverage and the other AMs when corpus sizing supports it.
