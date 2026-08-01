# Review request — Task 214 P0: spec-vs-code drift inventory

- Task: `plan/tasks/214-ec-distann-spec-remediation.md`, phase P0
- Packet: `reviews/task-214/001-drift-inventory/`
- Head: `baf81d498`, branch `task-203-ec-distann-conformance`
- Date: 2026-08-01. Coder: fable (Claude Fable 5)

## What this packet contains

The full spec-vs-code examination Task 214 P0 requires: every distann FR
(075–084), NFR (017–022), and ADR (085/086) walked against
`src/am/ec_distann/`, the `sql/bootstrap.sql` + `src/lib.rs` DDL surface, and
the suite/fixture conformance machinery in `crates/ecaz-cli/`, itemising
behavior shipped-but-unspecified, specified-but-changed, and
specified-but-removed.

- `inventory.md` — the consolidated inventory (sections A–H): the work list
  for P1–P5 and the base for the Task 211/212/213 P0 spec rounds.
- `artifacts/audit-*.md` — eight per-cluster audit reports, every finding with
  file:line evidence.
- `artifacts/catalog-inventory.md` — the complete 20-table catalog + SQL API
  map (P3 input).
- `artifacts/manifest.md` — provenance.

## Headlines (details in `inventory.md`)

1. **A1** — the sharded, membership-only head (the shipped default since Task
   210) is normatively undocumented; FR-080 describes its inverse. The §4.1
   replica/attestation subsystem has no owning requirement.
2. **A2** — ADR-086's ACCEPTED decision was reversed in code (replica demoted
   to opt-in nonconforming; "rejected" TRAV-30 gateway copies shipped).
   Superseding ADR needed; gateway copies need an owning FR.
3. **A3** — the spec describes only the v5 physical lane, but the legacy v4
   lane (16-byte fingerprint, GUC roster with raw conninfo, second lifecycle
   SQL surface with name collisions) still ships and drives multi-node scans.
4. **A4** — FR-083 DML is written as shipping but is unimplemented on the
   distributed lane; v5 deletes are a silent noop (also flagged as a candidate
   code bug, not just doc drift).
5. 78 itemized findings (22 high), 17 of 20 catalog tables unmentioned in any
   spec, and a [CODE?] list (inventory §H) of candidate code bugs surfaced by
   the audit that Task 214 will not fix (documentation-only task).

## Questions for the reviewer

1. **A3 posture**: given the research-no-backward-compat convention, is the
   right spec statement "v5 is the design; the v4 lane is fixture substrate
   only" (and if so, should retiring the v4 multi-node lane become a filed
   task), or does the legacy lane deserve normative coverage?
2. **[CODE?] list disposition** (inventory §H): agree these route to a coder
   task rather than being absorbed into P2 spec text? The silent v5 delete
   noop (A4) and the silent dropped-row materialization path (FR-079 F4) look
   like real hazards.
3. Any objection to the P1-before-P2 sequencing and to landing Task 211–213
   P0 specs against the elevated structure (inventory §H)?

## Status

P0 complete pending review. P1 (elevation to `spec/functional/distann/`)
starts next; the inventory's per-phase work lists (C/D → P2, E → P3, F → P4)
drive the remaining phases.
