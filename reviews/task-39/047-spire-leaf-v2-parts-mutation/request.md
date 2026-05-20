# Task 39 / 047 — SPIRE leaf_v2_parts.rs mutation campaign

## Goal

Second slice of the reviewer-prescribed SPIRE storage mutation cascade
(`reviews/task-39/044-helpers-expansion/feedback/2026-05-19-02-reviewer.md`).
Drive every mutation in `src/am/ec_spire/storage/leaf_v2_parts.rs` to
**0 missed / 0 timeouts**, adding new killing tests where the existing
careful suite has gaps and documenting the one equivalent mutant.

## Result

**68 mutations enumerated → 67 KILLED, 1 equivalent, 0 timeouts.**

Initial run with the existing careful suite from packets 029 + 044:
**61 KILLED + 7 MISSED.** The seven survivors broke down as:

- 1 in `Meta::validate` (empty-meta + non-zero segment_count guard)
- 2 in `Segment::decode` (prefix-length boundary at `tail.len() <
  LEAF_V2_SEGMENT_PREFIX_BYTES`)
- 2 in `Segment::validate_against_meta` (`||` -> `&&` in the
  header-vs-meta guard chain)
- 2 in `Segment::validate_against_meta` (`|` -> `&` / `^` in the
  `DELTA_INSERT | DELTA_DELETE` flag mask)

This packet ships **5 new killing tests** in
`src/am/ec_spire/storage/tests/leaf.rs`, each targeting one or more of
those survivors. After the re-run, 67/68 are caught; the remaining
`| -> ^` mutation at line 418 is **mathematically equivalent** to the
original (XOR of two disjoint single-bit flags is identical to OR) and
is recorded in `triage.md` as an equivalent-mutant verdict per the
Task 39 docs/hardening triage table.

## Code change

`src/am/ec_spire/storage/tests/leaf.rs` — 5 new tests in the
`miri_leaf_v2_*` family:

- `miri_leaf_v2_meta_rejects_empty_meta_with_nonzero_segment_count`
- `miri_leaf_v2_segment_decode_distinguishes_prefix_boundary_via_error_message`
- `miri_leaf_v2_segment_validate_rejects_pid_mismatch_only`
- `miri_leaf_v2_segment_validate_rejects_object_version_mismatch_only`
- `miri_leaf_v2_segment_validate_rejects_row_with_delta_insert_flag`

Plus a small helper `leaf_v2_segment_with_mismatched_header` that
clones a known-good segment and twists one header field at a time —
the same one-twist-on-known-good pattern packet 029 established and
my packet 044 extended.

No production code change. See `triage.md` for the per-mutation map.

## Methodology

Same as packet 046: the reviewer's prescribed
`cargo mutants --package ecaz-careful-hardening --file ...` invocation
finds 0 mutants because the SPIRE storage children are mounted via
`include!`, which cargo-mutants does not traverse. The
`artifacts/run-leaf-v2-parts-mutations.py` helper:

1. Parses each line of the `--Zmutate-file` enumeration into
   `(line, col, operator-swap-or-body-replacement)`.
2. Applies the mutation textually to the production source file.
3. Runs `cargo test --manifest-path hardening/careful/Cargo.toml --lib
   --quiet -- leaf_v2` (a focused-but-broad filter that captures every
   leaf-V2-touching test).
4. Records KILLED (test failed) or MISSED (all tests passed).
5. Reverts the source from `/tmp/leaf_v2_parts_original.rs`.

Total wall time on this packet's two runs: ~7 minutes each.

## Validation

Artifacts under `reviews/task-39/047-spire-leaf-v2-parts-mutation/artifacts/`:

- `leaf-v2-parts-mutants-enumerated.txt` — full mutation list (68 items).
- `run-leaf-v2-parts-mutations.py` — verification helper.
- `manual-verification.log` — final per-mutation verdict after the new
  killing tests landed: **67 KILLED, 1 MISSED (equivalent), 0
  PATCH-FAIL.**
- `post-verification-tests.log` — `cargo test --manifest-path
  hardening/careful/Cargo.toml --lib`: **534 passed, 0 failed**
  (was 529 after packet 046; +5 new tests).

Production source `src/am/ec_spire/storage/leaf_v2_parts.rs` is
byte-for-byte identical to its pre-packet state
(`diff /tmp/leaf_v2_parts_original.rs ...` returned empty).

## Reviewer Direction

- Confirm the equivalent-mutant verdict on `|` -> `^` at line 418:60.
  Rationale: `0x0008 ^ 0x0010 = 0x0018 = 0x0008 | 0x0010`; no input
  distinguishes the two expressions.
- Confirm the per-file manual verification methodology is acceptable
  for the remaining 11 SPIRE storage files plus `ec_spire/page.rs`.
  Packet 046 raised this question (versus restructuring the careful
  crate to use `mod` declarations); this packet shipped under the
  manual methodology, which works but is per-file mechanical.
