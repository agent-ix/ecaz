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
| IVF tuples | block refs, centroid, list-directory, posting, PQ-codebook, rerank group header, and rerank group payload segment fixed offsets |
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
| `ivf_metadata_v9.hex` | current IVF metadata (92 bytes) decode and swapped-version rejection |
| `ivf_metadata_v8.hex` | legacy 92-byte IVF metadata with two-value RaBitQ score flag before exact-dequant mode, now rejected by version |
| `ivf_metadata_v7.hex` | legacy 92-byte IVF metadata lacking metadata-backed RaBitQ rerank score/clip fields, now rejected by version |
| `ivf_metadata_v6.hex` | legacy 92-byte IVF metadata using packed `0x2B` groups with whole-vector TurboQuant rerank payloads, now rejected by version |
| `ivf_metadata_v5.hex` | legacy 92-byte IVF metadata using packed `0x2B` groups with non-residual RaBitQ rerank payloads, now rejected by version |
| `ivf_metadata_v4.hex` | legacy 92-byte IVF metadata using the old `0x2A` sidecar, now rejected by version |
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
| IVF | `9` | accepts the IVF metadata tag, rejects all other versions |
| SPIRE partition objects | `1`, `2` | accepts known object versions, rejects unknown versions |

Any incompatible field addition or reinterpretation must add a new format tag
and update the layout assertions, fixture golden files, and upgrade matrix.

## IVF Metadata Format

IVF writes and reads metadata format version `9` only; any other version
(including the legacy 86-byte v3 layout, the 92-byte v4 `0x2A` sidecar layout,
the v5 packed-group layout with non-residual RaBitQ rerank payload bytes, and
the v6 packed-group layout with whole-vector TurboQuant rerank payload bytes,
the v7 layout that kept RaBitQ rerank score/clip in mutable reloptions, and the
v8 layout that stored byte 22 as a two-value RaBitQ score flag) is rejected.
This is a research project with no backward compatibility - an index written by
an older format is simply rebuilt.

The v9 metadata struct is `EC_IVF_METADATA_BYTES = 92` bytes wide.
`EC_IVF_METADATA_RABITQ_RERANK_SCORE_MODE_OFFSET = 22` stores the compact
rerank score mode (`0 = estimator/default`, `1 = rabitq least_squares`,
`2 = exact_dequant`) and
`EC_IVF_METADATA_RABITQ_RERANK_CLIP_OFFSET = 23` stores the RaBitQ rerank clip
(`1..=8`). These are build-time payload interpretation knobs; scan and insert
read them from metadata rather than mutable live reloptions.
`EC_IVF_METADATA_RERANK_SIDECAR_HEAD_OFFSET = 80` holds the head `ItemPointer` of
the packed compact rerank group-header chain (tag `0x2B`). A head of
`ItemPointer::INVALID` is the legitimate "no sidecar" state for
`rerank_placement = 'source'` and for f32 storage; the scan then reranks from
the heap/source-vector path. That is a runtime placement state, not a
compatibility mode.

`EC_IVF_METADATA_RERANK_SIDECAR_DIRECTORY_HEAD_OFFSET = 86` is retained as a
legacy field-width slot from ADR-079, but v9 packed rerank groups write
`ItemPointer::INVALID` there. The old v4 directory mapped `0x2A` sidecar blocks;
v9 follows the `next_group_tid` field stored in each `0x2B` group header instead.

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
| `0x2A` | rerank sidecar block | legacy v4; compact rerank rep keyed by heap TID (Task 111g) |
| `0x2B` | rerank group header | current v9; scorer-width compact rerank group metadata |
| `0x2C` | rerank group payload segment | current v9; payload-only continuation bytes |

The legacy v4 `0x2A` rerank sidecar block stored a tid-keyed run of compact
rerank payload bytes, chained via a per-block `next_tid`. It remains only as a
benchmark/code baseline; v9 readers reject v4 metadata rather than reading
`0x2A` as a current format.

The v9 `0x2B` rerank group header stores a logical scorer-width group once:
`[tag:u8=0x2B][rerank_format:u8][list_id:u32][scorer_width:u16][valid_count:u16]`
`[payload_len:u16][total_heap_tids:u32][total_payload_bytes:u32]`
`[header_payload_bytes:u16][next_segment_tid:6][next_group_tid:6][reserved:u16]`,
followed by the deleted bitmap for `scorer_width`, `valid_count` gammas,
`valid_count` heap-TID counts, `valid_count` heap-TID offsets, `valid_count`
payload offsets, `total_heap_tids` heap TIDs, and the first payload fragment.
Build writes groups per list and flushes at scorer-width completion or list
boundary. Each posting stores its direct group-header TID in the existing
`rerank_tid` slot.

The v9 `0x2C` rerank group payload segment stores continuation bytes only:
`[tag:u8=0x2C][payload_bytes:u16][next_segment_tid:6]` followed by
`payload_bytes` payload bytes. Payload segments do not repeat group metadata.
Group headers chain through `next_group_tid` for fallback scans, vacuum, and
inspection. Vacuum tombstones dead group entries by setting the header deleted
bitmap; REINDEX regenerates compact groups.

`rerank_placement = 'source'` writes no compact payload. `rerank_placement =
'table'` is reserved for a future real table-owned persisted payload design and
is not the heap-source fallback.

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
