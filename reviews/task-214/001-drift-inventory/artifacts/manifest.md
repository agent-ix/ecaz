# Artifact manifest — Task 214 packet 001 (P0 drift inventory)

- Task bucket: `reviews/task-214/`
- Packet: `reviews/task-214/001-drift-inventory/`
- Head SHA at audit time: `baf81d498` (branch `task-203-ec-distann-conformance`)
- Date: 2026-08-01
- Method: eight parallel audit agents (Claude Fable 5), one per spec cluster,
  each reading the spec artifacts in full and verifying every requirement/AC
  against the implementation with file:line evidence; consolidated by the
  coordinating session into `../inventory.md`. No benchmarks were run; this is
  a static spec-vs-code examination (documentation task, no runtime behavior
  change), per the task file's "No benchmark gate" clause.

## Artifacts

| File | Scope |
| --- | --- |
| `audit-fr075-fr076.md` | FR-075 AM surface, FR-076 record/handoff formats |
| `audit-fr077-fr078.md` | FR-077 sharded build/stitch, FR-078 placement/handoff/coordinator |
| `audit-fr079-fr081.md` | FR-079 remote expansion protocol, FR-081 query orchestration |
| `audit-fr080-head.md` | FR-080 coordinator head index (most-drifted spec) |
| `audit-fr082-fr083.md` | FR-082 epoch lifecycle, FR-083 DML path |
| `audit-fr084-adr085-adr086.md` | FR-084 traversal replica + both distann ADRs |
| `audit-nfr017-022.md` | NFR-017..022 vs suite/fixture enforcement machinery |
| `catalog-inventory.md` | Complete 20-table DDL surface + pg_extern API map (P3 input) |

Consolidated deliverable: `../inventory.md` (sections A–H; the P1–P5 work
list). Key result lines cited by `request.md` are the section A headlines and
the counts below.

## Counts

- Findings across slices: 78 itemized (drift or gap), plus 6 explicit
  verified-conformant verification notes and a conformant-highlights section
  per slice.
- Severity high: 22 · medium: 33 · low: 23 (per-slice labels).
- Catalog: 20 tables; 17 with zero spec mention; 2 with no deletion path;
  4 missing from the REVOKE block.
- Files referencing the pre-elevation spec path `functional/index/distann`
  (P1 cross-reference fix list): 31 (grep at head SHA; list reproducible via
  `grep -rl 'functional/index/distann' --include='*.md' --include='*.rs'`).

## Provenance notes

- Auditor claims were required to carry file:line for every finding; the
  coordinating session spot-checked overlapping findings across slices
  (EXPLAIN surface ×3 slices, sharded head ×4, gateway copies ×3,
  `recover_epoch_publish` signature ×2 — all agreed).
- Isolated/shared surfaces, lanes, fixtures: not applicable (no measurement).
