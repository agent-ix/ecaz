# spec-review Checklist Application

Applied 2026-05-17 against NFR-016 and ADR-070 + the index update.

## ID Format and Uniqueness

| Check | Result | Notes |
|---|---|---|
| ID format `NFR-XXX` | ✅ | `NFR-016` (3-digit) |
| ID format `ADR-XXX` | ✅ | `ADR-070` (3-digit) |
| `{PARENT}-AC-N` for acceptance criteria | ✅ | NFR-016-AC-1..6 |
| `{PARENT}-EV-N` for evolution rules (project convention) | ✅ | NFR-016-EV-1..10 — matches NFR-015 "Reporting Rules" / NFR-004 sub-section style. Not in the generic checklist but consistent with the repo. |
| `{PARENT}-VR-N` for verification | ✅ | NFR-016-VR-1..5 — matches the NFR template |
| No duplicate IDs | ✅ | `grep id: NFR-016` returns one match; `grep id: ADR-070` returns one match |
| Sequential | ✅ | NFR-016 follows NFR-015; ADR-070 follows ADR-069 |

## NFR Quality (adapted from FR Quality checklist)

| Check | Result | Notes |
|---|---|---|
| Description clear/specific | ✅ | Quality Attribute + Requirement statement |
| Related US/StR linked | ✅ | StR-001 upstream |
| Outputs (acceptance shape) defined | ✅ | 6 ACs, each with at least one VR |
| Behavior detailed | ✅ | 10 evolution rules cover bump triggers, lifecycle states, transition path, forward-compat, matrix authority, fixture coverage, size pinning, add-field procedure, fixture provenance |
| Constraints document rationale | ✅ | §Notes (Non-Normative) explains the "bump if unsure" bias, the emergency carve-out, and the fixture-tool optionality |
| Error conditions documented | ✅ | NFR-016-EV-1 mandates reject-before-decode; NFR-016-EV-3 names the lifecycle ERROR states |
| Verifiable criteria | ✅ | Every AC has at least one VR. AC-4 and AC-6 are reviewer-process (intentional — see §Open Questions below) |
| Dependencies documented | ✅ | §Dependencies table covers upstream/design/downstream/coordinated |

## ADR Quality (project convention, no generic checklist)

| Check | Result | Notes |
|---|---|---|
| Frontmatter present (id, title, status, impact, date) | ✅ | Mirrors ADR-045 |
| §Context establishes the problem | ✅ | Describes heterogeneous current state + cites payload_flags / format_version mechanisms already in tree |
| §Options Considered enumerates alternatives | ✅ | Three options with pros/cons each |
| §Decision is normative | ✅ | Each option has normative constraints under "A.", "B.", "C." |
| §Consequences names downstream effects | ✅ | Default conservative; existing flags blessed; TLV available but unused; WAL strict |
| §Verification names reviewer + test checks | ✅ |
| §Open Questions present | ✅ | TLV tag-space ownership; flag-field sizing default |
| §Dependencies links spec artifacts | ✅ | NFR-016 (realises), ADR-045 / ADR-032 (coordinated), Tasks 37 + 42 (paired) |
| Index entry added | ✅ | `spec/adr/index.md` updated under Current Optional or Deferred Decisions |

## Cross-Referencing

| Check | Result | Notes |
|---|---|---|
| NFR-016 cites ADR-070 | ✅ | §Dependencies "Upstream-design" row |
| ADR-070 cites NFR-016 | ✅ | Frontmatter `impact:` + §Decision opening + §Dependencies "Realises:" |
| Both cite affected FRs | ✅ | Same FR set across both (FR-007, FR-008, FR-013, FR-015, FR-022, FR-027, FR-034, FR-035, FR-036) |
| Full IDs used (no shorthand) | ✅ |
| Terminology consistent | ✅ | "format version", "payload kind", "posture" used uniformly across both artifacts |

## Common Issues Audit

| Issue | Found? | Notes |
|---|---|---|
| Vague criteria ("fast", "user-friendly") | No | All criteria are concrete (byte-offset, version-equality, file-existence) |
| Only happy path | No | Each option in ADR-070 names reject conditions; NFR-016-EV-1 + EV-3 + EV-4 are explicit about ERROR / Removed states |
| AC without test | Partial | NFR-016-AC-4 and AC-6 verified by reviewer process, not automation. Documented as Open Question 1 in the request packet. |
| Inconsistent IDs | No | All 3-digit |

## Findings

### Strengths

- **Clean separation of policy from mechanism.** NFR-016 owns the
  "what discipline" question; ADR-070 owns the "which encoding shape"
  decision. Neither overreaches.
- **Retroactively blesses existing in-tree posture.** ADR-070
  §Cross-Cutting rule 3 names HNSW + DiskANN `payload_flags` as
  Option B without requiring code changes. Avoids the "ship policy,
  break tree" trap.
- **Default is conservative.** Option A (reject unknown) is the
  default; B and C must be justified per surface. This pushes the
  cost of forward-compat onto the contributor who wants it.

### Items worth flagging

1. **AC-4 and AC-6 are governance criteria, not automatable as-is.**
   AC-4 ("a new version ships with: matrix row + fixture + test +
   assertions + release note") and AC-6 ("forward-compat behavior is
   documented in the FR") are checked at PR-review time by a human.
   That's honest, and NFR-015 follows the same pattern. If you want
   them automated, the path is a small linter that parses
   `fixtures/upgrade/matrix.csv`, walks each FR for a posture
   declaration, and fails when either is missing. Out of scope for
   this packet.

2. **NFR-016 omits SPIRE storage FRs from its `relationships:` list.**
   FRs that describe `src/am/ec_spire/storage/**` (leaf V2, partition
   objects, manifests) are covered in spirit but not enumerated. I
   left them out because I wasn't sure which FR numbers own SPIRE
   storage today. Reviewer SHOULD add the missing IDs or confirm
   they aren't yet specified.

3. **ADR-070 §Open Question 1 (TLV tag registry ownership)** is left
   to per-FR ownership. If a cross-cutting tag emerges later (e.g.,
   a per-payload checksum field), this would need revisiting. Not
   blocking.

4. **WAL replay failure-semantics ADR is not in this packet.** NFR-016
   §Dependencies and ADR-070 §Dependencies both point at "WAL ADR…
   coordinated with Task 37" but it is not yet written. That ADR
   will land when Task 37 has a concrete WAL record contract to
   constrain.

5. **Index entry status is PROPOSED.** Consistent with ADR-045 and
   ADR-046; will move to ACCEPTED after this review packet is acted
   upon and any per-FR posture declarations land.

## Analysis sub-skills

The spec-review skill lists six analysis sub-skills. Applied
selectively:

- **spec-analysis-integrity** (completeness, consistency, atomicity):
  Each EV rule is atomic (single normative claim). Each AC has at
  least one VR. Terminology is consistent across NFR-016 and ADR-070.
  No conflicting requirements detected. ✅
- **spec-analysis-dependency** (separate enablement from feature
  work): NFR-016 is enablement (it constrains FRs without prescribing
  feature behaviour). ADR-070 is design (it picks the encoding
  shape). Both sit upstream of feature FRs that consume on-disk
  format work. ✅
- **spec-analysis-evidence** (verification methods): NFR-016-VR-1..5
  name concrete test lanes (`make on-disk-fixtures`,
  `make upgrade-smoke`, `make layout-check`, `make endian-qemu`).
  VR-5 is reviewer process; flagged in finding #1. ✅
- **spec-analysis-failure-domain**, **spec-analysis-risk-complexity**,
  **spec-analysis-scope-boundary**: deferred. These are most
  productive for feature FRs; for an evolution-discipline NFR and
  an encoding-convention ADR they have less leverage.

## Verdict

Both artifacts pass the checklist. Recommended for ACCEPTED status
after:

- Reviewer confirms or extends the FR list in NFR-016's
  `relationships:` (see finding #2).
- Reviewer answers Reviewer Focus question 2 about whether Option C
  belongs in ADR-070 today or should be deferred.

The two follow-ups (per-FR posture declarations + WAL ADR) can be
filed as separate packets.
