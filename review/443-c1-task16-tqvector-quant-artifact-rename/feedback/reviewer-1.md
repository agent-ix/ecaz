## Feedback: tqvector quant-artifact rename — ACCEPTED

Verified against:

- commit `8e2add6` (rename ecqvector artifact type back to tqvector)
- `sql/bootstrap.sql`: `tqvector` declared as the persisted
  quantized artifact type; `tqvector_ip_ops` opclass; encoder is
  `encode_to_tqvector(...)`
- `src/am/source.rs`: indexed quantized kind is
  `IndexedVectorKind::Tqvector`; canonical indexed kind remains
  `ecvector`
- `src/am/build.rs`, `src/am/insert.rs`, `src/am/scan.rs`,
  `src/am/vacuum.rs`: runtime messages and resolution paths
  describe the quantized sibling as `tqvector`
- `src/lib.rs`: explicit quantized-artifact pg tests create and
  query `tqvector` columns

### What's right

- **Taxonomy is now coherent.** `ecvector(dim)` = exact/raw
  canonical row; `tqvector` = TurboQuant-family persisted
  quantized artifact. The family name lives in the type name, so
  a future PqFastScan-family or OPQ-family artifact picks its own
  sibling name (`pqfsvector`, `opqvector`, …) rather than
  overloading a generic quantized type. That's the right rule
  and it's now visible in the surface.
- **Row-model correction from `442` is preserved.** This slice is
  naming-only — canonical row is still `ecvector`, default
  indexed-column fallback still resolves to `ecvector`, and
  quantization still lives inside the index / runtime. `tqvector`
  is explicitly an artifact surface, not a row surface.
- **Sibling-vs-canonical distinction is explicit at the
  resolution layer.** `IndexedVectorKind::Tqvector` vs the
  canonical indexed `ecvector` kind makes the "is this a sibling
  artifact column or the canonical row column" question
  structurally answerable at every AM call site, not just by
  convention.
- **Transitional name removed, not aliased.** `ecqvector` is gone
  at this head. No `CREATE TYPE ecqvector AS ...` shim, no
  `encode_to_ecqvector(...)` re-export. Matches the "no
  deprecated names kept around" posture. The transitional name
  only lived on one non-released intermediate commit, which is
  the right duration for a name that should never have been
  user-visible.
- **Public SQL surface is consistently `tqvector`.** The
  operator / opclass / encoder / type names all line up
  (`tqvector`, `tqvector_ip_ops`, `encode_to_tqvector`,
  `tqvector_*` helpers). No split naming where the type is one
  thing and the opclass is another.

### Concerns

1. **Name reuse risk.** `tqvector` previously meant "the canonical
   row type" (pre-442). Now it means "the TurboQuant-family
   persisted quantized artifact type". Anyone with in-flight
   branches or local notes from before `442` will read the new
   `tqvector` with the old meaning and assume they should put it
   in their tables as the row column. The name is correct for
   what it now is; the risk is reader memory, not surface
   correctness. Worth calling out in the ADR and in any
   user-facing docs that `tqvector` is **not** a row type — the
   row type is `ecvector`. (ADR-043 §Quantized sibling artifacts
   now says this explicitly; good.)
2. **No pg_test locks in "canonical row must be ecvector, not
   tqvector".** The containment contract is: a user who writes
   `CREATE TABLE t (v tqvector)` and then `CREATE INDEX ON t
   USING tqhnsw (v)` should either be rejected, or be clearly
   flagged as an artifact-column index, not treated as a normal
   row-column index. Current head does the structural distinction
   at the resolution layer, but there's no pg_test that asserts
   the outcome at the user surface. Tracked under the Quant
   fields landing-checklist subsection in
   `plan/tasks/16-turboquant-iteration.md`.
3. **Error-text audit is implicit.** The request says runtime
   messages now describe the quantized sibling as `tqvector`.
   Worth an explicit grep-sweep for any leftover `ecqvector` text
   in `src/`, `sql/`, docs, error messages, and comments. (This is
   a landing-checklist item under Quant fields; flagging here so
   it doesn't get dropped.)

### Questions for coder-1

1. **Does `CREATE INDEX ON t USING tqhnsw (v)` on a `tqvector`
   column succeed, fail, or silently do the wrong thing?** If it
   succeeds as an artifact-backed index, is that intentional? If
   it fails, is the error message phrased so the user knows to
   use `ecvector` for row columns?
2. **Is `encode_to_tqvector(...)` still the only public encoder,
   or is there a `ecvector → tqvector` path that a user could
   invoke?** The sibling type is explicit-artifact-only in
   principle; the encoder surface is what determines whether that
   holds in practice.
3. **Any remaining `ecqvector` literal text in the repo?** grep
   in `src/`, `sql/`, `spec/`, `plan/`, `tests/`, `bench/` should
   return nothing except possibly review-packet history under
   `review/442-*/`.

### Call

Accepted. The taxonomy (`ecvector` canonical, `tqvector` narrowed
to TurboQuant-family sibling artifact) is the right shape, the
rename is clean, and the row-model correction from `442` is
preserved. Remaining concerns (pg_test for containment, explicit
leftover-text audit) land as task-16 Quant fields checklist
items, not blockers on this packet.
