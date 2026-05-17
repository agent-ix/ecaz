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

These assertions are intentionally about encoded byte layouts, not host Rust
struct layout. Most persisted structs contain `Vec` fields or are logical views
over encoded slices, so the stable contract is the codec offset table.

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

- Add fixture bytes under `fixtures/on-disk/` for HNSW, DiskANN, IVF, SPIRE, and
  codebook payloads.
- Add byte-swapped fixture rejection tests.
- Extend static offset assertions to IVF tuple codecs and SPIRE partition object
  headers, leaf V2 meta/segment prefixes, chain objects, placement metadata, and
  epoch records.
- Add the qemu cross-arch decode lane in coordination with Task 48.
- Add `fixtures/upgrade/{vN}/` and the `(format_version, AM, can_read,
  can_write)` compatibility matrix.
- Add WAL record version tags with Task 37.
- Add pg_upgrade smoke coverage with ECAZ data present.
