---
id: NFR-016
title: On-Disk Format Evolution Discipline
type: non-functional-requirement
artifact_type: NFR
status: PROPOSED
relationships:
  - target: "ix://agent-ix/ecaz/StR-001"
    type: "constrains"
    cardinality: "N:1"
  - target: "ix://agent-ix/ecaz/FR-007"
    type: "constrains"
    cardinality: "1:N"
  - target: "ix://agent-ix/ecaz/FR-008"
    type: "constrains"
    cardinality: "1:N"
  - target: "ix://agent-ix/ecaz/FR-013"
    type: "constrains"
    cardinality: "1:N"
  - target: "ix://agent-ix/ecaz/FR-015"
    type: "constrains"
    cardinality: "1:N"
  - target: "ix://agent-ix/ecaz/FR-022"
    type: "constrains"
    cardinality: "1:N"
  - target: "ix://agent-ix/ecaz/FR-027"
    type: "constrains"
    cardinality: "1:N"
  - target: "ix://agent-ix/ecaz/FR-034"
    type: "constrains"
    cardinality: "1:N"
  - target: "ix://agent-ix/ecaz/FR-035"
    type: "constrains"
    cardinality: "1:N"
  - target: "ix://agent-ix/ecaz/FR-036"
    type: "constrains"
    cardinality: "1:N"
---
# NFR-016: On-Disk Format Evolution Discipline

## Quality Attribute

Primary quality attribute: **maintainability**. Secondary: **compatibility**.

This NFR governs how every byte ECAZ persists to disk (page payloads,
metadata pages, codebook artifacts, partition objects, manifests) evolves
across releases. It does not define *what* the formats are — that is owned
by the individual FRs (FR-007 HNSW page layout, FR-034 DiskANN build and
storage, etc.). It defines the *discipline* under which those formats may
change.

## Requirement

Every on-disk payload Ecaz writes SHALL be governed by an explicit
format-version tag, an explicit evolution lifecycle, and a fixture-backed
compatibility matrix. Layout drift, silent breakage, and undocumented
deprecation are non-conformant.

## Evolution Rules

### NFR-016-EV-1: Format-Version Tag is Mandatory

Every distinct on-disk payload kind (page header, tuple shape, partition
object header, manifest record, codebook artifact, WAL record) SHALL
carry a format-version field in its on-wire representation. The version
field SHALL appear at a fixed byte offset asserted at compile time, SHALL
be encoded little-endian per the project convention, and SHALL be
inspected before any other field is interpreted.

### NFR-016-EV-2: Version Bump Triggers

A new format version SHALL be allocated and shipped whenever any of the
following changes to a payload kind:

1. **Size change.** The total byte size, any field offset, or any field
   width changes.
2. **Field-meaning change.** A field's interpretation, units, or value
   domain changes, even if its byte size is preserved.
3. **Field addition or removal.** Adding a new field, removing an existing
   field, or reordering fields.
4. **Discriminator change.** Adding a new variant to an enumerated
   discriminator field that an older reader would misinterpret.

Bug fixes that change only the value an encoder writes (e.g., a
miscomputed checksum) without changing the format SHALL NOT bump the
format version; the prior fixture SHALL be regenerated with a comment
identifying the bug fix.

### NFR-016-EV-3: Version Lifecycle States

Each format version SHALL occupy exactly one of the following states at
any time, recorded in the compatibility matrix:

- **Current** (`can_read=true`, `can_write=true`): the version emitted by
  the current encoder and accepted by the current decoder. At most one
  *current* version per payload kind, except when an opt-in transition is
  in flight (see NFR-016-EV-4).
- **Read-only legacy** (`can_read=true`, `can_write=false`): the decoder
  still accepts the version for online reads and `pg_upgrade` carryover,
  but the encoder no longer emits it. Indexes built at this version
  remain queryable; they SHALL NOT be silently rewritten without operator
  action.
- **Removed** (`can_read=false`, `can_write=false`): the decoder rejects
  the version with an ERROR that names the version and the operator
  action required to migrate. Removal SHALL be a deliberate breaking
  change announced in release notes.

### NFR-016-EV-4: Transition Path

A version SHALL move through the lifecycle in order: not-yet-shipped →
Current → Read-only legacy → Removed. Skipping the Read-only legacy
state SHALL require an explicit ADR that records the rationale (security
fix, fundamental redesign, etc.) and the operator-impact statement.

When an opt-in transition is in flight (two writable versions
simultaneously to allow rollback), both versions SHALL be present in the
compatibility matrix as Current with the transition window documented in
the row's notes.

### NFR-016-EV-5: Forward-Compatibility Convention

Optional regions, extension blocks, and reserved-for-future-use fields
in an on-disk payload SHALL follow the convention selected in the
governing ADR (see Dependencies). Each payload kind SHALL document
whether forward compatibility is intentional ("older reader skips the
extension block safely") or unsupported ("older reader rejects the
version cleanly"). Silent decode of unknown extension content SHALL NOT
occur.

### NFR-016-EV-6: Compatibility Matrix is Source of Truth

`fixtures/upgrade/matrix.csv` SHALL list every format version Ecaz can
read or has ever shipped as a writable version, along with its
lifecycle state and the golden fixture file that pins its bytes. A
version that does not appear in the matrix SHALL NOT be readable by
production code. The matrix SHALL be enforced by an automated test
(currently `tests/upgrade_matrix.rs`) that runs on every PR.

### NFR-016-EV-7: Fixture Coverage is Mandatory

Every entry in the compatibility matrix SHALL be backed by a golden
fixture under `fixtures/on-disk/` whose bytes are checked into the
repository. The fixture SHALL be exercised by a decode test asserting
each field, and by a byte-swap rejection test for at least the
format-version discriminator. Endianness coverage SHALL include a
big-endian cross-arch decode lane.

### NFR-016-EV-8: Compile-Time Size Pinning

Every persisted struct or layout SHALL be size-pinned and
offset-pinned by static assertions in `tests/size_of_assertions.rs`.
A change that alters the size or any documented field offset SHALL
produce a compile error, forcing the contributor to bump the format
version per NFR-016-EV-2 and follow NFR-016-EV-3 / NFR-016-EV-4.

### NFR-016-EV-9: Add-Field Procedure

A contributor who adds a field to a persisted struct SHALL:

1. Allocate a new format version constant.
2. Add a row to the compatibility matrix for the new version as
   Current and demote the previous Current row to Read-only legacy
   (or document a flighting window per NFR-016-EV-4).
3. Add a golden fixture for the new version exercising the new field.
4. Update the size and offset assertions to reflect the new layout.
5. Update the decoder so it can read both the new version and the
   previous Read-only-legacy versions.

### NFR-016-EV-10: Fixture Provenance

Golden fixtures MAY be hand-crafted with documented magic values for
small surfaces or captured from a real index for surfaces too large
to hand-construct safely. Each fixture file SHALL carry a comment
identifying its provenance ("captured from PG18 corpus X" or
"hand-crafted to exercise fields A, B, C") so a future maintainer
can regenerate it. A tooling lane that captures fixtures from a live
cluster MAY be added later; absence of such tooling does not relax
any other rule in this NFR.

## Acceptance Criteria

### NFR-016-AC-1

Every entry in `fixtures/upgrade/matrix.csv` has a corresponding fixture
file under `fixtures/on-disk/`, a decode test in
`tests/on_disk_fixtures.rs`, and a byte-swap rejection test for its
format-version discriminator.

### NFR-016-AC-2

Every persisted struct named in FR-007, FR-008, FR-013, FR-015,
FR-022, FR-034, FR-035, FR-036, and the SPIRE storage FRs has at least
one static size or offset assertion in `tests/size_of_assertions.rs`.
A pull request that changes any pinned size or offset without bumping
the format version fails CI.

### NFR-016-AC-3

`fixtures/upgrade/matrix.csv` records the lifecycle state for every
version Ecaz can read. A version that is removed from the encoder
without a corresponding matrix update fails the upgrade-matrix test.

### NFR-016-AC-4

A new format version landed under this NFR ships with: (a) a release
note entry, (b) a matrix row transition for the prior version, (c) a
new fixture, (d) a new decode test, (e) updated static assertions.
A new format version that lacks any of these artifacts fails review.

### NFR-016-AC-5

`make endian-qemu` runs on the schedule defined by NFR-005
(build-and-CI) and decodes every fixture in `fixtures/on-disk/` on a
big-endian target. Any fixture that decodes correctly on
little-endian but fails on big-endian (or vice versa) is a bug under
this NFR, not a fixture defect.

### NFR-016-AC-6

Forward-compatible extension blocks are encoded per the convention
selected in the governing ADR. A payload kind whose forward-compat
behavior is not documented in its FR is in violation of NFR-016-EV-5.

## Verification

| ID | Verification Activity | Evidence |
|----|-----------------------|----------|
| NFR-016-VR-1 | `make on-disk-fixtures` decodes every fixture and rejects byte-swapped discriminators. | `tests/on_disk_fixtures.rs` test log |
| NFR-016-VR-2 | `make upgrade-smoke` enforces matrix invariants and the current writable set. | `tests/upgrade_matrix.rs` test log |
| NFR-016-VR-3 | `make layout-check` enforces compile-time size and offset pinning. | `tests/size_of_assertions.rs` test log |
| NFR-016-VR-4 | `make endian-qemu` cross-decodes fixtures under big-endian qemu. | scheduled CI run, packet-local |
| NFR-016-VR-5 | Reviewer checks new format-version rows against NFR-016-EV-2 through NFR-016-EV-4 during PR review. | review packet `request.md` + matrix diff |

## Dependencies

| Type | Reference | Description |
|------|-----------|-------------|
| Upstream | StR-001 | Native compressed vector storage is meaningless if the on-disk bytes drift silently between releases. |
| Upstream-design | ADR for on-disk forward-compatibility encoding | Selects the encoding convention for optional regions and reserved fields. To be ADR-070 or successor. |
| Downstream | FR-007, FR-008, FR-013, FR-015, FR-022, FR-027, FR-034, FR-035, FR-036 | Every FR that describes a persisted surface is constrained by this NFR. |
| Coordinated | Task 37 (crash recovery + amcheck) | WAL record version tags are governed by the WAL portion of this NFR; per-record contracts live in the Task 37 FR work. |
| Coordinated | Task 42 (on-disk format invariants) | Provides the fixture, matrix, and qemu lane infrastructure this NFR mandates. |

## Notes (Non-Normative)

- This NFR does not pick the *value* of the format-version field for any
  payload — that is owned by each surface's FR (e.g., `INDEX_FORMAT_V3_DISKANN
  = 3` is owned by FR-034). This NFR only requires the field exist, be
  bumped per the rules, and appear in the matrix.
- The bump-trigger rules are intentionally strict: a contributor who is
  unsure whether a change requires a bump SHALL bump. Over-bumping is
  cheap (a new matrix row, a new fixture); under-bumping silently
  corrupts production indexes.
- The "skip the Read-only legacy state" carve-out in NFR-016-EV-4 is for
  emergencies (security fixes, demonstrated data corruption). It is not
  a normal release path.
- A future automated tool that captures fixtures from a running PG18
  cluster is permitted by NFR-016-EV-10 but not required. Hand-crafted
  fixtures with documented magic constants are acceptable forever for
  surfaces small enough to hand-construct reliably.
