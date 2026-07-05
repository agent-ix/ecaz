# Review Request: SPIRE Scan/Cost/Vacuum Relation Boundaries

## Summary

This checkpoint continues the soundness-audit response for SPIRE helpers that accepted raw PostgreSQL relation or scan pointers through safe APIs.

Code commit: `dd418a66441e76b53224e72c9f034135f8c47a1e`

The reviewer’s finding remains correct for these helpers: they rely on PostgreSQL callback-owned `Relation` and `IndexScanDesc` pointers, so the functions must not be safe Rust APIs.

## Scope

- Marked SPIRE scan heap-relation and snapshot resolution helpers unsafe.
- Marked SPIRE heap-rerank prefetch and heap-slot allocation helpers unsafe where they consume raw relation descriptors.
- Marked SPIRE cost and tree-height snapshot helpers unsafe where they consume raw index relations.
- Marked SPIRE vacuum live-assignment count helper unsafe.
- Marked the remote-candidate relation OID helper unsafe and added call-site acknowledgments in libpq planning/executor paths.

## Validation

- `cargo check --all-targets --no-default-features --features pg18,bench`: passed; existing unused-import warning remains in `src/am/mod.rs`.
- `git diff --check`: passed.
- `make unsafe-block-count`: passed; count increase is expected for this explicit-boundary pass.

See `artifacts/manifest.md` for packet-local command provenance and key output lines.
