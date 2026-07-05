# Review Request: IVF Page Relation Boundary

## Summary

This checkpoint addresses the soundness-audit concern that IVF page helpers exposed safe APIs over raw `pg_sys::Relation` page access.

Code commit: `a8daba4c968164637f4c55b68e500c6e72bfa3e7`

The reviewer was correct about the anti-pattern: `IvfPageRelation::new` was a safe constructor for a raw PostgreSQL relation view, and several page helper APIs accepted raw relations without making the caller acknowledge the live-IVF-index contract.

## Scope

- Made `IvfPageRelation::new` unsafe.
- Made the exported IVF page helpers that take `pg_sys::Relation` unsafe:
  - centroid/directory/codebook/posting readers
  - posting-list visitors and rewriters
  - posting append and directory metadata updates
  - debug posting block summaries
  - metadata page initialize/read/update
- Updated IVF admin, insert, quantizer, scan, and vacuum callers with explicit unsafe acknowledgments at the relation-owner boundary.

## Completion Audit Note

This packet improves contract visibility and closes part of the IVF page/helper cluster called out by the soundness audit. It does not claim a raw unsafe-block count reduction; in fact, some local unsafe acknowledgments moved back to call sites because the previous safe facade was the reviewed anti-pattern.

Task 50 remains open. Follow-up IVF work should continue propagating unsafe boundaries outward to PostgreSQL callback edges where possible, and later passes can replace some explicit boundaries with lifetime-bearing relation/page wrappers.

## Validation

- `cargo check --all-targets --no-default-features --features pg18,bench`: passed; existing unused-import warning remains in `src/am/mod.rs`.
- `git diff --check`: passed.
- `make unsafe-block-count`: passed.

See `artifacts/manifest.md` for packet-local command provenance and key output lines.
