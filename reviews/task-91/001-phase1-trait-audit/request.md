# Task 91 Packet 001: Phase 1 Trait Audit

## Summary

This packet asks for review of the Task 91 Phase 1 design gate. It does not
change Rust code.

The audit is in `artifacts/trait-audit.md`.
The concrete Phase 2 implementation contract is in
`artifacts/dispatch-contract.md`.

Design checkpoint under review:

- `5cdcf38a529ddec50665a4ea44b806f03383897f` - Task 87 merge baseline

## Decisions

- Keep `QuantCodec::score_ip_batch` as the universal batch-kernel dispatch
  method for Task 91/92.
- Use enum dispatch at AM boundaries; avoid hot-loop `dyn QuantCodec` because
  the trait has an associated `PreparedQuery` type.
- Treat grouped-PQ trained model ownership as codec/prepared-query state, not
  `CandidateBatch` metadata.
- Keep QJL residual-sign sidecars in
  `CandidateMeta::GammaAndResidualSigns`.
- Rename storage-binding adapters so unqualified "codec" refers to the common
  `QuantCodec` contract.
- Make IVF Phase 2 the reference slice for the grown trait before SPIRE,
  HNSW, and DiskANN migrations.

## Validation

See `artifacts/manifest.md` for artifact metadata.

No tests were run. This is the design-only Phase 1 packet required before Task
91 implementation starts.

## Review Focus

- Confirm keeping `score_ip_batch` avoids unnecessary churn while still giving
  Task 92 one universal kernel entry point.
- Confirm enum dispatch is the right default before Phase 2.
- Confirm the grouped-PQ model binding plan is sufficient for IVF first and
  later DiskANN/HNSW migrations.
- Confirm the AM path audit covers every scoring path Task 91 needs to own.
