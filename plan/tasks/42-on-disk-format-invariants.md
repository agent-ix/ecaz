# Task 42: On-Disk Format and Cross-Arch / Cross-Version Invariants

Status: **proposed** — locks down ECAZ's on-disk page formats so they survive
endianness differences, version upgrades, and silent layout drift.

## Scope

Every byte that ECAZ writes to disk or that lives in a buffer page:

- `src/storage/page.rs` — generic page header / tuple layout.
- `src/am/ec_hnsw/page.rs` — HNSW page format, tuple kinds, metadata page.
- `src/am/ec_diskann/page.rs` — DiskANN metadata page, neighbor packing,
  payload format versioning (V1, V2, V3).
- `src/am/ec_spire/storage/**` — SPIRE leaf V2 segment / object metadata,
  placement metadata, epoch records.
- Codebook persistence (Lloyd-Max, OPQ rotation matrices).
- Any future WAL record encoding (in tandem with Task 37).

## Why

ECAZ ships multi-arch (currently aarch64-darwin + x86_64-linux, both
little-endian) but the project will be deployed to environments where
operators run base backups across machines and recover on different
hardware. The on-disk format must be:

- **Endian-explicit.** Every integer field encoded with `to_le_bytes` /
  `from_le_bytes` (the convention) — no implicit `transmute` of structs,
  no `unaligned` casts that rely on host endianness.
- **Version-tagged.** Every page header carries an `index_format_version`
  byte / range that the reader checks; mismatched versions ERROR cleanly
  with an upgrade-path message instead of decoding garbage.
- **Size-stable.** `size_of` / `align_of` / field offsets for on-disk types
  are pinned by static assertion; a refactor that reorders fields produces
  a compile error, not silent corruption.
- **Forward-compatible where designed to be.** When a format adds an
  optional region (e.g., SPIRE leaf V2 adding a payload format
  enumeration), older readers must reject cleanly or skip safely per the
  design.

The existing `layout-check` lane covers some of this but not exhaustively.
Endianness is not exercised at all (every CI host is little-endian).
Cross-version replay is not exercised at all (every test starts from a
freshly built index).

## Approach

1. **Endian fixtures.** Add a `fixtures/on-disk/` directory containing
   bit-for-bit on-disk page samples for each AM at the current
   `index_format_version`. Tests decode them and verify the result matches a
   golden in-memory representation. A separate test byte-swaps every
   multi-byte field and asserts the decoder rejects (not silently misreads).
2. **Cross-arch CI.** Add a CI lane (per Task 48) that runs the on-disk
   fixtures under `qemu-user` for an opposite-endian target (e.g., powerpc64
   or s390x). The test compiles, runs decode-only paths under qemu, and
   verifies parity with the native run.
3. **Static layout assertions.** Extend `tests/size_of_assertions.rs` to
   cover every on-disk type, with `const _: () = assert!(size_of::<T>() ==
   N)` and `assert!(offset_of!(T, field) == N)`. Any layout change is a
   compile error.
4. **Version compatibility matrix.** Maintain a table of
   `(format_version, AM, can_read, can_write)` and a test that exercises
   every entry: build a corpus with format vN, upgrade the extension to
   vN+1, scan and verify recall floor. Stored corpora live under
   `fixtures/upgrade/{vN}/`.
5. **WAL record version tags.** Pair with Task 37: each ECAZ WAL record
   carries a version byte; replay rejects unknown versions cleanly.
6. **`pg_upgrade` smoke.** A separate lane that runs `pg_upgrade` from PG18
   to itself (in-place) with ECAZ data present; verifies recall floor and
   `pg_amcheck` parity post-upgrade. When PG19 lands, extends to PG18→PG19.
7. **Make lanes:**
   - `make layout-check` (existing) — extended assertions.
   - `make on-disk-fixtures` — decode golden fixtures, verify parity.
   - `make endian-qemu` — qemu cross-arch decode lane (nightly).
   - `make upgrade-smoke` — version compatibility matrix.
   - `make pg-upgrade-smoke` — `pg_upgrade` end-to-end with ECAZ.

## Validation

- All golden fixtures decode to the expected representation; byte-swapped
  copies are rejected.
- qemu cross-arch lane decodes fixtures correctly.
- A deliberately reordered struct field is caught by the size/offset
  assertions at compile time.
- Upgrade matrix: index built at vN reads correctly at vN+1; vN+1 features
  not enabled when reading vN data.
- `pg_upgrade` smoke produces an upgraded cluster with ECAZ indexes that
  pass `pg_amcheck` and meet recall floors.

## Exit Criteria

- `fixtures/on-disk/` covers every on-disk page kind for every AM.
- `make endian-qemu` runs nightly with green status.
- `make upgrade-smoke` runs per-PR with the current matrix; new versions
  add a row.
- `docs/on-disk-format.md` documents the version policy, the endian
  convention, the fixture process, and the upgrade matrix.

## Dependencies

- Independent of Tasks 36–41; can land in parallel.
- The qemu lane needs the CI matrix work from Task 48.
- The `pg_upgrade` smoke depends on the live-cluster harness from Tasks
  37–38.
