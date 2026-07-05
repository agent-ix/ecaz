# Request: Unify Entry Seeding On Layer-0 Beam Trace

Commit: `c4a9df3`

Summary:
- remove the remaining legacy `ef_search == 1` entry-seeding path in `scan.rs`
- make bootstrap entry seeding always flow through the graph-owned layer-0 beam runner
- keep the existing seeded-frontier width contract by truncating the discovered beam trace to the bootstrap frontier limit

Please review:
- whether removing the last non-beam entry seeding branch is the right next A2 handoff
- whether preserving the current width-1 frontier behavior through trace truncation is the right compatibility contract
- whether the runtime now has a clean enough entry-seeding seam to move more bootstrap frontier progression behind graph-owned traversal next
