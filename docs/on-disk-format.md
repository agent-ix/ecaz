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
| `hnsw_element_tuple_v3.hex` | HNSW element tuple decode |
| `hnsw_neighbor_tuple_v3.hex` | HNSW neighbor tuple decode |
| `hnsw_grouped_codebook_tuple_v3.hex` | HNSW grouped-PQ codebook shard decode |
| `diskann_vamana_metadata_v3.hex` | DiskANN Vamana metadata decode and swapped-version rejection |
| `diskann_vamana_node_tuple_v3.hex` | DiskANN Vamana node tuple decode and swapped-neighbor-count rejection |
| `diskann_vamana_codebook_tuple_v3.hex` | DiskANN grouped-PQ codebook shard decode |
| `ivf_metadata_v1.hex` | IVF metadata decode and swapped-version rejection |
| `ivf_centroid_tuple_v1.hex` | IVF centroid tuple decode and swapped-dimension rejection |
| `ivf_list_directory_tuple_v1.hex` | IVF list-directory tuple decode |
| `ivf_posting_tuple_v1.hex` | IVF posting tuple decode |
| `ivf_pq_codebook_tuple_v1.hex` | IVF PQ-codebook shard decode |

## Version Policy

Every current metadata page carries a format-version field that readers check
before interpreting the rest of the payload:

| AM | Current tags | Reader behavior |
| --- | --- | --- |
| HNSW | `1`, `2`, `3` | accepts known tags, rejects unknown tags |
| DiskANN | `3` | accepts the DiskANN tag, rejects foreign tags |
| IVF | `1` | accepts the current tag, rejects unknown tags |
| SPIRE partition objects | `1`, `2` | accepts known object versions, rejects unknown versions |

Any incompatible field addition or reinterpretation must add a new format tag
and update the layout assertions, fixture golden files, and upgrade matrix.

## Remaining Task 42 Gaps

- Extend fixture bytes under `fixtures/on-disk/` to SPIRE and remaining
  HNSW/DiskANN/IVF page kinds.
- Extend byte-swapped fixture rejection tests to additional bounded multi-byte
  fields where the current decoder can reject malformed values.
- Extend static offset assertions to additional SPIRE routing/top-graph object
  body prefixes if they become durable page-buffer contracts beyond the current
  partition-object and metadata codecs.
- Add the qemu cross-arch decode lane in coordination with Task 48.
- Add `fixtures/upgrade/{vN}/` and the `(format_version, AM, can_read,
  can_write)` compatibility matrix.
- Add WAL record version tags with Task 37.
- Add pg_upgrade smoke coverage with ECAZ data present.
