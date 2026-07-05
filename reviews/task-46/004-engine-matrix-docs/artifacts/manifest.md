# Packet 004 — Task 46: engine matrix docs

## Head

- Task bucket: `reviews/task-46/`
- Packet path: `reviews/task-46/004-engine-matrix-docs/`
- Validation head SHA: `ddb51741e`
- Branch: `main`
- Surface under validation: `docs/hardening.md` `## Fuzzing →
  Engine Matrix` subsection. Documentation-only.

## Diff summary

- `docs/hardening.md`: +71 lines (new `### Engine Matrix`
  subsection appended under the existing `## Fuzzing` section).

No code change, no fuzz target change, no Make target change. This
is a pure docs slice; the surfaces the docs describe were
established by packets 001 / 002 / 003.

## Key result lines cited by request.md

- `## Fuzzing → ### Engine Matrix` section now exists at
  `docs/hardening.md:182` (after the existing SQLsmith reference).
- The matrix names: libFuzzer, Honggfuzz, AFL+, ECAZ-grammar
  SQLsmith, cross-pollinate — five lanes covering Task 46 §Approach
  3 + §Approach 4 + §Approach 5.
- Target shape taxonomy distinguishes decoder targets (raw bytes,
  retained intentionally per §Why) vs structured-input targets
  (Arbitrary derive, success-path round-trip property).

## Task 46 progress after this slice

| # | §Exit Criterion | Status |
|---|---|---|
| 1 | Every structured-input fuzz target uses Arbitrary | partial (2 of N) |
| 2 | `make sqlsmith-ecaz` nightly with seed corpus | 0% |
| 3 | Honggfuzz + AFL+ weekly with `make fuzz-cross-pollinate` | 0% |
| 4 | `fuzz/corpus/` minimized + committed | ✓ done (003) |
| 5 | `docs/hardening.md` documents engine matrix | **✓ done (this)** |

Task 46 ≈ 35% complete (2 of 5 gates closed; one partial).
