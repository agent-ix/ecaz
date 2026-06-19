# Task 42: On-Disk Format and Cross-Arch / Cross-Version Invariants

Status: **complete** (2026-06-16; closeout `788a074a4`, reviewer-accepted
honest-audit feedback under `reviews/task-42/018-*/`) — fixture, matrix,
layout, qemu, WAL-policy, and PG18 same-binary `pg_upgrade` smoke
infrastructure landed. Broader CI stabilization and richer live-upgrade
coverage are intentionally deferred until the CI surface is steadier and
a second writable format version ships (documented follow-on, not a gap).

## Update — IVF metadata v3 is the first second-writable format (Task 111g, 2026-06-18)

Task 111g (packet `reviews/task-111g/003b-persisted-compact-sidecar/`) bumped the
IVF metadata format from v2 to **v3**, the first time an AM in this repo gained a
*second writable metadata version* (the trigger this task deferred its richer
live-upgrade coverage on). The bump is **additive and backward-readable**:

- `EC_IVF_METADATA_BYTES` widened 80 → 86; new field
  `EC_IVF_METADATA_RERANK_SIDECAR_HEAD_OFFSET = 80` (the compact rerank sidecar
  chain head, tag `0x2A`). `EC_IVF_METADATA_BYTES_V2 = 80` records the legacy
  width that decode still tolerates.
- Reader accepts versions `1..=3`; a v1/v2 index (80-byte special area) decodes
  with `rerank_sidecar_head = INVALID` ("no sidecar"), and rerank falls back to
  the heap/table source path.

Reconciled by 111g (registry-consistency layer of this task):

- `tests/size_of_assertions.rs`: pins `EC_IVF_INDEX_FORMAT_VERSION == 3`,
  `EC_IVF_METADATA_BYTES == 86`, `EC_IVF_METADATA_BYTES_V2 == 80`,
  `EC_IVF_METADATA_RERANK_SIDECAR_HEAD_OFFSET == 80`.
- `fixtures/upgrade/matrix.csv`: ivf rows are now `1` (read-only), `2`
  (read-only), `3` (read+write); `tests/upgrade_matrix.rs` expects ivf writable
  set `{3}`.
- `tests/on_disk_fixtures.rs::ivf_metadata_v1_fixture_decodes`: asserts the
  committed 80-byte **v1** on-disk image decodes under the v3 reader with
  `rerank_sidecar_head == INVALID` — the durable backward-read fixture proof.
- `docs/on-disk-format.md`: version-policy table (IVF `1,2,3`), an IVF metadata
  format-version history table, and the `0x2A` rerank-sidecar tag.

**Remaining EV-3 follow-on (not claimed done here):** NFR-016-EV-3's full *live*
upgrade rehearsal for IVF — build a corpus at the old version, upgrade the
extension, scan with the new reader, and record a recall floor beside a
historical fixture directory — is still a future lane. Because v3 is additive and
backward-readable (proven by the v1-fixture decode above), no in-place data
migration is required to read an old index; the live rehearsal is coverage, not a
correctness gate for this bump.

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
   every entry. While each AM has only one writable format, this is a
   registry-consistency check that pins read/write support and fixture
   presence. Once a second writable format ships, the lane must follow
   NFR-016-EV-3 by building a corpus with format vN, upgrading the extension to
   vN+1, scanning, and verifying a meaningful recall floor. Stored corpora live
   under `fixtures/upgrade/{vN}/`.
5. **WAL record version tags.** Pair with Task 37: current ECAZ writes use
   PostgreSQL GenericXLog page images/deltas and have no extension-owned WAL
   payload body. If Task 37 adds custom ECAZ redo/replay records, each custom
   record carries a version byte at offset 0 and replay rejects missing or
   unknown versions cleanly per ADR-070's default reject-unknown posture.
6. **`pg_upgrade` smoke.** A separate lane that runs `pg_upgrade` from PG18
   to itself (in-place) with ECAZ data present. The current smoke is HNSW-only
   and verifies top-2 ID equality, index presence, heap count, and
   `pg_amcheck` on the upgraded cluster. Richer recall-floor measurement and
   `ec_ivf` / `ec_diskann` / `ec_spire` coverage remain future extensions.
   When PG19 lands, extends to PG18→PG19.
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
  not enabled when reading vN data. Current validation is registry consistency
  until a second writable version exists.
- `pg_upgrade` smoke produces an upgraded cluster with ECAZ indexes that
  pass `pg_amcheck` and preserve the smoke corpus' top-2 IDs. Meaningful recall
  floors activate with a richer corpus.

## Exit Criteria

- `fixtures/on-disk/` covers every on-disk page kind for every AM.
- `make endian-qemu` runs nightly with green status.
- `make upgrade-smoke` runs per-PR with the current matrix; new versions
  add a row.
- `docs/on-disk-format.md` documents the version policy, the endian
  convention, the fixture process, and the upgrade matrix.
- Deferred closeout: broader CI stabilization, richer `pg_upgrade` recall
  coverage, and multi-AM `pg_upgrade` smoke coverage.

## Dependencies

- Independent of Tasks 36–41; can land in parallel.
- The qemu lane needs the CI matrix work from Task 48.
- The `pg_upgrade` smoke can run against same-binary PG18 today; PG18→PG19
  coverage waits for PG19 support.
