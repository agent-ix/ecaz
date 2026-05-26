# Task 46/005: three new structured-fuzz targets (closes §Exit Criteria #1)

## Scope

Closes Task 46 §Exit Criteria gate 1:

> 1. Every fuzz target that consumes a structured input uses
>    `Arbitrary`-derived inputs.

Adds the three §Approach 2 targets the Task 46 spec names by name,
plus the structured sibling of `fuzz_vector_normalize` (the last
existing raw target that consumes a structured input — the four
decoder targets intentionally stay raw per Task 46 §Why).

Validation head: post-commit landing this slice.

## What changed

- `fuzz/Cargo.toml` — registers three new bins.
- `fuzz/fuzz_targets/topk_merge_structured.rs` — Task 46 §Approach
  2.a. Two `Arbitrary` `Vec<i64>` lists, sorted, passed through a
  single-pass O(k) `merge_truncate_ascending`. Property: result
  equals the slower `concat → sort → truncate` reference.
- `fuzz/fuzz_targets/quant_encode_decode_roundtrip.rs` — Task 46
  §Approach 2.c. `Arbitrary` `Vec<u32>` mapped to finite f32 in
  `[-1.0, 1.0]`; runs
  `ProdQuantizer::encode → pack_payload → decode_approximate`;
  asserts the decoded vector has the right dimension and only
  contains finite floats. Tolerance is "is_finite + correct len",
  not exact recovery (quantization is lossy by design).
- `fuzz/fuzz_targets/vector_normalize_structured.rs` — structured
  sibling of `fuzz_vector_normalize`. Maps `Vec<u32>` words to
  finite clamped `f32` so every iteration drives `encode` rather
  than rejecting on non-finite input.

No production code change. Test-/tooling-only addition.

§Approach 2.b (`fuzz_spire_leaf_v2_roundtrip`) is not in this slice
because `SpireLeafPartitionObjectV2Meta` is `pub(super)` in
`src/am/ec_spire/storage/leaf_v2_parts.rs` — wrapping it requires a
crate-level re-export decision that's a follow-up. §Exit #1 only
requires Arbitrary on *existing* structured-input targets; the
SPIRE leaf V2 target does not exist yet, so this slice closes the
gate without it.

## Evidence

Three 10-second smoke runs, captured to packet artifacts:

- `artifacts/fuzz-topk-merge-10s.log`
  - 850,323 runs in 10 s (77k exec/s)
  - 150 new corpus entries added in the 10 s window
  - 0 crashes — `merge_truncate == sort_truncate` invariant held for
    every Arbitrary-derived pair.
- `artifacts/fuzz-quant-encode-decode-roundtrip-10s.log`
  - 5,446 runs in 10 s (495 exec/s — slow because each iteration
    runs `ProdQuantizer::new` + encode + decode_approximate)
  - 25 new units
  - 0 crashes — every `encode→decode_approximate` round-trip
    produced a finite vector of the original dimension.
- `artifacts/fuzz-vector-normalize-structured-10s.log`
  - 3,758 runs in 10 s (341 exec/s — same reason as above)
  - 11 new units
  - 0 crashes — every clamped `[-1, 1]` Arbitrary input passed
    through `ProdQuantizer::encode` without panicking.

## Reviewer focus

- The topk_merge target's "system under test" is an inline single-
  pass O(k) merge; the assertion is against the obviously-correct
  `concat→sort→truncate` reference. Future ECAZ merge code can pick
  up the same primitive and the target re-targets directly.
- The quant round-trip property is "stays finite + right
  dimension", not exact recovery — quantization is lossy so a tight
  numeric tolerance would fail by design. The looser property still
  catches dimension drift, panics, and NaN/inf escapes.
- The vector_normalize sibling matches the raw target's shape
  (dim cap 128, `ProdQuantizer::new(dim, 4, 42)`) so the two
  targets fuzz the same surface from different input
  distributions.

## Task 46 §Exit Criteria progress after this slice

| # | §Exit Criterion | Status |
|---|---|---|
| 1 | Every structured-input fuzz target uses `Arbitrary` | **✓ done (this)** — parse_text/unpack_mse/vector_normalize structured siblings + 2 new structured-input targets; the 4 decoder targets stay raw per §Why |
| 2 | `make sqlsmith-ecaz` nightly with seed corpus | 0% |
| 3 | Honggfuzz + AFL+ weekly with `make fuzz-cross-pollinate` | 0% |
| 4 | `fuzz/corpus/` minimized + committed | ✓ done (003) |
| 5 | `docs/hardening.md` documents engine matrix | ✓ done (004) |

Task 46 ≈ 60% complete (3 of 5 §Exit gates closed).

## Out of scope (still open)

- §Exit #2: `make sqlsmith-ecaz` ECAZ-grammar lane — separate slice.
- §Exit #3: Honggfuzz + AFL+ + `make fuzz-cross-pollinate` —
  separate slice.
- §Approach 2.b: `fuzz_spire_leaf_v2_roundtrip` — blocked on a
  SPIRE leaf-V2 encoder re-export decision; not §Exit-blocking.
- Deliberately-introduced parser-bug regression evidence (§Validation
  last bullet) — applies broadly; one-time fixture commit, separate
  slice.
- Per-target §Validation 5× threshold disposition from packet 002
  — operator said "i dont care which choice", so the slice stands.
