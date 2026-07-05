# Review: Scan Candidate Provenance

Commit: `17c6e6f`

Scope:
- `src/am/scan.rs`
- `src/lib.rs`

Summary:
- `ScanCandidate` now carries `source_tid` as explicit discovery provenance.
- Entry-seeded frontier candidates use `INVALID` source provenance because they are seeded directly from metadata, not discovered from another element.
- Adjacency-discovered candidates now record the element tid they were expanded from.
- Frontier debug helpers now expose provenance alongside frontier slots so pg regressions can validate source tracking.
- Regression coverage now verifies:
  - bootstrap seeded successor candidates record the entry candidate as their discovery source
  - consume/refill-discovered candidates record the consumed frontier head as their discovery source when refill adds a new candidate
  - the existing consume/refill regression no longer assumes a fixed three-slot frontier when the current entry point exposes fewer live neighbors

Validation:
- `cargo test`
- `PGRX_HOME=/tmp/tqvector_pgrx_home cargo pgrx test pg17`
- `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`

Review focus:
- Does `source_tid` capture the right current provenance contract for traversal groundwork, or is there a better boundary before multi-hop expansion starts?
- Are the entry-seeded `INVALID` source semantics and adjacency-discovered source semantics clear and consistent?
- Do the new debug/provenance assertions cover the important parent-tracking edge cases without overfitting to the current bootstrap graph shape?
