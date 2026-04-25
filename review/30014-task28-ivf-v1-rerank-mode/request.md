# Review Request: Task 28 IVF V1 Rerank Mode

Scope: Phase 4 rerank-mode checkpoint. IVF v1 now makes its compressed-only
scoring contract explicit: `auto` resolves to persisted `off`, `off` is
accepted, and heap/source rerank modes are rejected until their storage
contracts exist.

Task: `plan/tasks/28-ivf-access-method.md` Phase 4

Branch: `task28-ivf`

Head SHA: `417d7c6a8299c04282c61dc12b0106885e5063eb`

Owner: coder2

Files:

- `src/am/ec_ivf/options.rs`
- `src/am/ec_ivf/page.rs`
- `src/am/ec_ivf/build.rs`
- `src/am/ec_ivf/scan.rs`
- `src/am/ec_ivf/mod.rs`
- `src/am/mod.rs`
- `src/lib.rs`
- `plan/tasks/28-ivf-access-method.md`

Validation:

- `cargo check --no-default-features --features pg18 --tests`
- `git diff --check`

Validation notes:

- Validation was PG18-only per the current AGENTS policy.
- No test suite was run. The new PG tests were compiled only.
- No measurement claim is made in this packet.

## Summary

This slice closes the Phase 4 rerank-mode decision:

- Adds explicit v1 rerank helpers on `RerankMode`.
- Persists `rerank = 'auto'` as effective `off` metadata.
- Rejects `heap_f32` and `source_column` at build/empty-build time with a
  clear unsupported-mode error.
- Revalidates persisted rerank mode during scan setup so stale or incompatible
  metadata cannot silently use the compressed-only path.
- Adds PG debug coverage for auto-to-off metadata and unsupported heap-f32
  rerank requests.
- Updates the task plan to move Phase 4 status to recall tests next.

## Review Focus

Please review for:

- Whether rejecting heap/source rerank at build time is preferable to allowing
  index creation and failing only at scan time.
- Whether persisting `auto` as `off` is the right v1 metadata contract, or
  whether metadata should preserve the user-specified reloption.
- Whether the scan-time validation is sufficient for old or externally-created
  metadata.
- Whether the error text is clear enough for users trying unsupported rerank
  modes.

## Non-Goals

This packet does not implement heap/source rerank, storage-format-specific
scoring, live insert, vacuum, planner costing, recall gates, or any measurement
claim.
