# Review Request: NFR-016 + ADR-070 (On-Disk Format Evolution Discipline)

## Summary

Two new spec artifacts under review:

- `spec/non-functional/NFR-016-on-disk-format-evolution-discipline.md` —
  the format-evolution discipline (maintainability + compatibility) that
  Task 42's work has been implicitly delivering against. Pins
  version-tag, bump-trigger, lifecycle, fixture, and size-assertion
  rules.
- `spec/adr/ADR-070-on-disk-forward-compat-encoding-convention.md` —
  the design decision that realises NFR-016-EV-5 (forward-compat
  encoding convention). Picks three permitted postures (reject-unknown
  / flag-byte / TLV) with reject-unknown as the default.

`spec/adr/index.md` is updated to list ADR-070.

## Why now

Task 42 (on-disk format and cross-arch / cross-version invariants)
has shipped the *mechanisms* (golden fixtures, byte-swap rejection,
qemu cross-arch CI, version-compat matrix, compile-time layout
pinning) but the *policy* layer was implicit. Without a normative
NFR, a future contributor adding a new field to a v3 struct has no
authoritative source for the bump-or-not decision, the deprecation
flow, or the forward-compat encoding shape. NFR-016 + ADR-070
provide that source.

This pair was identified during the Task 42 review cycle (see
`reviews/task-42/015-9056-task42-qemu-cross-build-fix/feedback.md`) and the user
direction: "all of that belongs in the spec, not in docs or task
record".

## Scope of this review

- The two new artifacts and the index update.
- Cross-references to existing FRs (FR-007, FR-008, FR-013, FR-015,
  FR-022, FR-027, FR-034, FR-035, FR-036) and NFRs (NFR-005,
  StR-001).
- The retroactive blessing in ADR-070 §Cross-Cutting rule 3 (HNSW +
  DiskANN `payload_flags` retroactively documented as Option B).

## Out of scope (deferred)

- Per-FR documentation updates that name each FR's chosen posture
  (Option A / B / C). ADR-070 §Cross-Cutting rule 1 mandates this but
  it is a follow-up edit across multiple FRs.
- The WAL ADR for replay failure semantics — coordinated with Task 37.
- A `cargo run --bin capture-on-disk-fixture` tooling lane. NFR-016-EV-10
  permits but does not require it.

## Validation

- `id` format unique and in sequence: NFR-016 (next after NFR-015),
  ADR-070 (next after ADR-069). Verified by `grep -n 'id: NFR-016' …`.
- Frontmatter `relationships:` array structured per the project
  pattern (compared against NFR-004 which has the longest
  `constrains` list).
- ADR-070 frontmatter `impact:` line names each affected FR, per
  ADR-045 pattern.
- Index entry added to `spec/adr/index.md` under "Current Optional
  or Deferred Decisions".

## Reviewer Focus

1. Whether NFR-016-AC-4 and NFR-016-AC-6 (governance criteria
   verified by review-time inspection rather than automated test)
   are an acceptable shape, or whether they should be reshaped into
   automated checks (e.g., a linter that diff-checks the matrix CSV
   against the encoded format versions).
2. Whether ADR-070's three-option menu is the right granularity, or
   whether Option C (TLV) should be dropped from this ADR and
   deferred to a separate decision when a payload kind actually
   needs it.
3. Whether the retroactive blessing of HNSW/DiskANN `payload_flags`
   as Option B is correctly framed in ADR-070 §Cross-Cutting rule 3.
4. The cross-reference set in NFR-016's `relationships:` — whether
   any FR is missing (e.g., FR-047 cloud loader, FR-049+ SPIRE
   storage FRs) or over-listed.
