---
id: ADR-070
title: "On-Disk Forward-Compat Encoding Convention"
status: PROPOSED
impact: Realises NFR-016 (on-disk format evolution discipline). Affects FR-007 (HNSW page layout), FR-008 (HNSW build), FR-013 (quantization pipeline), FR-015 (ProdQuantizer), FR-022 (vacuum), FR-027 (pgrx-pg18-upgrade), FR-034 (DiskANN build and storage), FR-035 (DiskANN scan/prefilter/rerank), FR-036 (DiskANN insert/vacuum diagnostics), and the SPIRE storage FRs. Sets the baseline convention for any future persisted surface.
date: 2026-05-17
---
# ADR-070: On-Disk Forward-Compat Encoding Convention

## Context

NFR-016 ("On-Disk Format Evolution Discipline") requires every persisted
payload kind to carry a format-version tag and to move through a defined
lifecycle (Current → Read-only legacy → Removed). For payloads that need
to *grow* between versions — adding a new field, embedding an optional
region, reserving space for future use — NFR-016-EV-5 mandates that the
governing ADR pick the encoding convention. This ADR is that decision.

Today the payload surface is heterogeneous:

- **HNSW metadata** uses `payload_flags: u8` plus per-payload sub-records
  to add optional regions (cold rerank, grouped search code). Readers test
  flag bits before consuming each region.
- **DiskANN Vamana metadata** also carries `payload_flags: u8` plus a
  `format_version: u16` header (`INDEX_FORMAT_V3_DISKANN = 3`).
- **SPIRE leaf V2 segments** carry an explicit `format_version` and an
  enumerated payload-format field selected per segment.
- **IVF metadata** carries `format_version` plus a fixed layout with no
  optional regions today.

None of these surfaces have a *project-wide convention* for "the reader is
older than the encoder; what should happen?" The bytes are well-defined
per surface, but the policy is implicit. NFR-016 requires the policy be
explicit, picked once, and documented in this ADR.

There are three plausible conventions, examined below.

## Options Considered

### Option A: No Forward Compat — Reject Unknown Versions Cleanly

The encoder emits version `vN`. The decoder accepts only the
versions it has been compiled to read. An older reader presented
with a `vN+1` payload reads the format-version discriminator, fails
the equality check, and ERRORs with a message that names the
encoder version and the operator action required.

- **Pros:** Simplest. The decoder doesn't need to know what fields it
  doesn't know about. Bugs from forward-compat skip logic
  (mis-skipping a length-prefixed region, mis-interpreting an
  unknown flag bit) are impossible by construction. Matches the
  current Ecaz behaviour everywhere.
- **Cons:** Operationally inconvenient: a rolling upgrade where one
  backend has read the new extension shared object and another has
  not produces noisy ERRORs on the older backend until the binary
  catches up. Every minor format change requires a full restart
  before reads work.

### Option B: Flag-Byte Optional Regions

The encoder emits a `payload_flags: u8` (or u16/u32 as the surface
warrants) where each bit names a known optional region. After the
fixed header, the decoder iterates known flag bits and consumes the
corresponding region. Bits the decoder does not recognise are
ignored; their corresponding bytes are skipped by reading a
length prefix on each optional region.

- **Pros:** Familiar; current HNSW and DiskANN payloads already use
  flag bytes. Fine-grained: a single payload can carry multiple
  independent optional regions added across versions. Older
  readers can ignore unknown flags and consume only what they know.
- **Cons:** The "older reader skips unknown region" path requires
  every optional region to be length-prefixed even when the field
  is fixed-size and self-describing. Flag bits are a limited
  resource (8/16/32 max); growth past that needs a different
  convention. Bit semantics drift between versions are subtle and
  hard to lint statically.

### Option C: Length-Prefixed Trailing Extension Block

The encoder emits the fixed header (with `format_version`) followed
by a length-prefixed "extension block" whose contents are TLV-like
(`tag: u16, length: u16, value: [u8; length]`). The decoder reads
the extension block's outer length, and within it iterates TLVs
consuming tags it recognises and skipping over tags it does not.

- **Pros:** Unbounded growth; no flag-bit budget. Self-describing
  per-field; a new field gets a new tag without any reader
  changes. Clear "skip unknown" semantics that are uniform across
  surfaces.
- **Cons:** Five bytes of header overhead per TLV (or four for
  `tag: u8, length: u8`). All readers must implement the TLV
  loop. The TLV format itself becomes a versioned surface
  (recursive problem) unless frozen forever.

## Decision

**Each persisted payload kind SHALL declare its forward-compat posture in
its governing FR, choosing one of the three options.** The default for
new payload kinds is Option A (reject unknown versions). Option B and
Option C are permitted where operational rolling-upgrade convenience
justifies the implementation cost.

Constraints on each option, normative:

### A. Reject-Unknown Posture

1. The decoder SHALL inspect the format-version discriminator before
   any other field.
2. On a version it does not recognise, the decoder SHALL ERROR with a
   message that names the seen version, the highest version it knows,
   and the operator action ("upgrade the extension to version X or
   later"). The message SHALL NOT include implementation-internal
   addresses or struct names.
3. The decoder SHALL NOT attempt to read any field past the
   discriminator on a rejected payload.

### B. Flag-Byte Posture

1. The flag field SHALL be sized in the header per the payload's
   growth expectations (`u8` is sufficient for ≤ 8 optional regions;
   `u16` for ≤ 16; `u32` thereafter). Bit assignments SHALL be
   documented in the governing FR.
2. Each optional region following the flag field SHALL be encoded
   as `u16 length || [u8; length] payload`. The decoder SHALL
   advance by `length + 2` regardless of whether it recognises the
   corresponding flag bit.
3. A flag bit SHALL NOT be reused: once allocated, a bit retains
   its semantics for the lifetime of the payload kind. Reuse
   requires a format-version bump per NFR-016-EV-2.
4. The decoder SHALL ignore unknown flag bits without ERROR. The
   decoder SHALL ERROR if a recognised flag bit is set but the
   corresponding region's `length` exceeds the remaining payload
   bytes.

### C. TLV Extension-Block Posture

1. The extension block SHALL be appended after all fixed fields and
   SHALL be framed by `u32 total_length || [TLV; ...]`.
2. Each TLV SHALL be encoded as `u16 tag || u16 length || [u8;
   length] value`. Tags are owned by the governing FR; the FR
   SHALL maintain a tag registry.
3. The decoder SHALL skip unknown tags. The decoder SHALL ERROR if
   a TLV's length exceeds the remaining block length, if the block's
   total length exceeds the remaining payload bytes, or if a
   recognised tag's value is malformed for its known schema.
4. Tag values SHALL NOT be reused. A tag retired in version `vN`
   SHALL remain skippable in version `vN+1`; if its semantics
   change, allocate a new tag and bump the format version per
   NFR-016-EV-2.

### Cross-Cutting

1. Posture choice SHALL be recorded in the governing FR's acceptance
   criteria, and SHALL appear in `fixtures/upgrade/matrix.csv` as a
   column or note.
2. A single payload kind SHALL NOT mix postures across versions in
   the same lifecycle row. A change in posture is a format break
   that follows NFR-016-EV-4 (and typically NFR-016-EV-4's "skip
   Read-only legacy" exception path requiring an ADR).
3. The HNSW `payload_flags` and DiskANN `payload_flags` fields
   already in production SHALL be retroactively documented as
   Option B postures in their governing FRs (FR-007, FR-034). No
   code change is required.
4. WAL record version tags (paired with Task 37) SHALL use Option A
   unless a separate ADR justifies otherwise. WAL replay failure
   semantics differ from page-decode failure semantics; conflating
   them via forward-compat skip logic is a recovery hazard.

## Consequences

- **Default posture is conservative.** New payload kinds reject
  unknown versions until someone justifies the rolling-upgrade
  affordance. Authors who want forward compat must declare and
  defend it.
- **Existing flag-byte payloads are blessed.** No retroactive
  rewrite. The governing FRs gain documentation rows tying their
  flag fields to Option B.
- **TLV is available but unused.** Option C is permitted for future
  surfaces that need unbounded growth. No current surface needs it.
- **WAL replay stays strict.** Option A is the default for WAL
  records, preserving recovery semantics. Task 37 may revisit.
- **NFR-016-EV-5 is satisfied** because every payload kind now has a
  named posture, and the posture's semantics are normative.

## Verification

- A reviewer assessing a new persisted payload kind SHALL confirm the
  governing FR names the posture (NFR-016-AC-6).
- A reviewer assessing a change to an existing payload SHALL confirm
  the change does not silently switch postures (this ADR §Cross-Cutting
  rule 2).
- The decode tests for each payload kind SHALL exercise the rejection
  message text under Option A, the unknown-flag skip path under
  Option B, and the unknown-tag skip path under Option C.

## Open Questions

- TLV tag registry: who owns the cross-payload-kind tag-space, if
  any? Current decision: each FR owns its own tag space. Revisit if
  a cross-cutting tag (e.g., "checksum") is wanted.
- Should Option B mandate a `u16` flag field over `u8` from the
  start to defer the bit-budget crisis? Current decision: leave to
  the governing FR; over-provisioning has zero cost on disk only
  when storage is dominated by other fields.

## Dependencies

- **Realises:** NFR-016 (on-disk format evolution discipline).
- **Affects:** every FR listed in the frontmatter.
- **Coordinated with:** ADR-045 (page-layout discipline) — defines the
  *layout* shape that this ADR's posture wraps; ADR-032 (coexisting
  index formats) — describes a transition window across two writable
  formats, an Option-B-shaped flighting case.
- **Pairs with:** Task 37 (WAL records) and Task 42 (fixture and
  matrix infrastructure). The WAL ADR for replay failure semantics
  remains open; it will be a separate ADR coordinated with Task 37.
