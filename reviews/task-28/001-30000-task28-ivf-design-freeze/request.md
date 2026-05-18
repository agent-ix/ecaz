# Review Request: Task 28 IVF Design Freeze

Scope: Phase 0 only. Activates IVF as an optional access method and freezes
the first implementation contract before the AM scaffold starts.

Task: `plan/tasks/28-ivf-access-method.md` Phase 0

Branch: `task28-ivf`

Head SHA: `d28d13d9630f55946f50355c2d9a611fc832a04c`

Owner: coder2

Files:

- `spec/adr/ADR-048-ivf-access-method.md`
- `spec/adr/ADR-017-hnsw-over-ivf.md`
- `spec/adr/ADR-035-spann-billion-scale.md`
- `spec/adr/ADR-041-module-structure-for-multi-am-multi-quantizer-growth.md`
- `spec/spec.md`
- `plan/tasks/28-ivf-access-method.md`

Validation:

- `git diff --cached --check` before commit
- No cargo or pgrx tests run; this is a docs/spec planning checkpoint.

## Summary

This slice freezes the Phase 0 IVF contract:

- New optional AM name: `ec_ivf`.
- HNSW remains the default.
- ADR-048 activates IVF and amends ADR-017 rather than replacing HNSW.
- ADR-035 is marked dropped; SPANN is no longer on the active roadmap.
- ADR-041 now names `ec_ivf` as the posting-list AM under the multi-AM
  module layout.
- IVF is explicitly a posting-list AM over the existing quantizer profiles:
  TurboQuant, PqFastScan, and RaBitQ.

## Design Contract

The first implementation target is plain IVFFlat-style:

1. Train centroids during `CREATE INDEX`.
2. Assign each vector to exactly one posting list.
3. Route queries by scoring normalized centroids.
4. Scan the nearest `nprobe` lists.
5. Score candidates with the selected posting-list quantizer profile.
6. Emit ordered results through the normal index-scan lifecycle.

## Review Focus

Please review for:

- Whether `ec_ivf` is the right SQL AM name.
- Whether the operator-class naming fallback is coherent:
  reuse `tqvector_ip_ops` / `ecvector_ip_ops` for `USING ec_ivf` if
  PostgreSQL catalog uniqueness permits it; otherwise use explicit
  `*_ivf_ip_ops` names.
- Whether spherical k-means is an acceptable first router for the current
  inner-product surface.
- Whether `storage_format = turboquant | pq_fastscan | rabitq | auto`
  is the right reloption shape.
- Whether the full-probe exactness gate is strong enough before ANN recall
  and planner claims.
- Whether marking ADR-035 as dropped is sufficient cleanup for removing
  SPANN from the active roadmap.

## Non-Goals

This packet does not request review of code, page layout, or callback
implementation. Those start in the next packet.
