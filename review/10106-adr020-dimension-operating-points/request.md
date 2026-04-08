# Review Request: ADR-020 Dimension Operating Points

Scope:
- `spec/adr/ADR-020-embedding-dimension-operating-points.md`
- `spec/spec.md`

Summary:
- realign ADR-020 away from broad competitive-positioning language and toward the intended
  `1024` vs `1536` vs `2048` embedding-dimension decision
- document repo-grounded formulas and first-order costs for:
  - `transform_dim`
  - 4-bit payload bytes
  - approximate element-tuple/page density
  - current AVX2 hot query-state size
- make the working hypotheses explicit:
  - `2048` is the quality candidate
  - `1024` is the speed candidate
  - `1536` is the current baseline, not the already-decided winner
- call out the main anti-confusion points that came up in design exploration:
  - FWHT is paid at encode/query-prep time, not per candidate score
  - `1536` and `2048` share the same padded FWHT size but not the same storage or hot-loop cost
  - page-density arithmetic is only a first-order proxy for full HNSW capacity
- update the spec index entry so ADR-020 is listed under its new dimensioning purpose

Out of scope:
- no code changes
- no SIMD implementation changes yet
- no competitive-landscape survey beyond the minimum context needed to support the dimensioning ADR

Validation:
- docs only; no Rust validation commands run

Please review:
- whether ADR-020 now reflects the intended `1024` / `1536` / `2048` design exploration
- whether the distinction between query-prep FWHT cost and per-candidate scoring cost is now clear
- whether the ADR strikes the right balance between current repo facts and still-unmeasured
  hypotheses
