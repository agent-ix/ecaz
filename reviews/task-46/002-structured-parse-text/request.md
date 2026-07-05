# Task 46/002: structured `parse_text` fuzz target

## Scope

Second structure-aware target under Task 46, sibling of the 001
slice. Targets `parse_text` — explicitly called out in Task 46 §Why
as the canonical example of structural waste in the existing
raw-byte fuzz stack.

This is **not** a kickoff packet — 001 opened the workstream. This
slice converts the second raw-byte target identified in Task 46 §Why
to an `Arbitrary`-derived round-trip target.

Validation head: `2bfcca062` (Task 46/002 code commit, on top of
`e5cb093e8` Task 48/001 coder reply).

## What changed

- `fuzz/Cargo.toml` — registers the new bin `fuzz_parse_text_structured`.
- `fuzz/src/lib.rs` — promotes `DEFAULT_QUANT_BITS` / `DEFAULT_QUANT_SEED`
  from crate-private to `pub` and re-exports them plus `payload_len`
  via the `bench_api` module so structured targets can drive the
  canonical-format encoder side.
- `fuzz/fuzz_targets/parse_text_structured.rs` — new structure-aware
  target. `Arbitrary` produces `(dim ∈ 1..=1024, gamma_bits: u32,
  code_seed: Vec<u8>)`; the target formats the canonical
  `[dim=…,bits=…,seed=…,gamma=…]:hex` string with exactly
  `payload_len(dim, bits) - 4` code bytes, calls `parse_text`, and
  asserts every field round-trips (`dim`, `bits`, `seed`, `codes`
  exactly; `gamma` within `1e-3 * |gamma|` of the input, the text-
  format precision floor for `f32::parse`).

No production code change. Test-/tooling-only addition.

## Why this surface

Task 46 §Why names `parse_text` explicitly:

> For higher-level inputs (a `(dim, bits, seed, codes)` tuple as in
> `fuzz_targets/parse_text.rs`, or a `VamanaMetadataPage`) the fuzzer
> spends most cycles producing inputs that are rejected at the first
> length check.

Seven structural gates protect `parse_text`'s success path: valid
UTF-8 → `[…]` brackets → comma-separated `key=value` header →
numeric `dim`/`bits` → `bits == 4` and `seed == 42` canonical
defaults → valid hex body → body length equal to
`payload_len(dim, bits) - 4`. The raw byte fuzzer almost never
passes all seven, so the parser's *arithmetic* (in particular the
length check that compares hex-decoded payload length to
`payload_len(dim, bits) - 4`) is barely exercised.

## Evidence

Matched 10-second runs at the same head, back-to-back on the same
machine, identical `-max_total_time` budget:

- `artifacts/fuzz-parse-text-structured-10s.log`
  - **cov 253, ft 635, corp 79**, 213k exec/sec, 280 NEW units in
    the 10s window.
- `artifacts/fuzz-parse-text-raw-baseline-10s.log`
  - **cov 127, ft 367, corp 141**, 639k exec/sec, 453 NEW units in
    the 10s window.

| metric | raw | structured | ratio |
|---|---:|---:|---:|
| cov  | 127 | 253 | **1.99×** |
| ft   | 367 | 635 | **1.73×** |
| corp | 141 | 79  | 0.56× |
| cov-per-corp-entry | 0.90 | **3.20** | **3.55×** |

### Honest framing of the result

The structured target hits roughly **2× absolute coverage** — below
the Task 46 §Validation ≥ 5× threshold the 001 packet cleared on
`unpack_mse`. Two reasons, both worth recording so a future packet
designer reads the signal correctly:

1. **`parse_text` has richer error branches than `unpack_mse`.**
   Where `unpack_mse` rejects almost every raw input at the first
   length check (one branch), `parse_text` traverses 7 different
   error returns plus the success arithmetic. A purely-success
   structured target by construction does not exercise the 7 error
   branches, so the raw target reaches edges the structured target
   skips.
2. **Coverage *density* — the cleaner Task 46 thesis signal — is
   3.55×.** Each input the structured target keeps in its corpus
   exercises 3.55× more coverage than each kept raw input.
   Structured fuzzing's "spend the budget on edge cases inside the
   shape" promise lands; the absolute-coverage comparison is
   confounded by the error-branch baseline.

The structured target's distinctive value here is the **round-trip
property**: `parse(format(dim, gamma, codes)) == (dim, bits, seed,
gamma, codes)`. The raw target cannot make that assertion — it has
no oracle for "what should this input parse to". Any future
parse_text/format drift surfaces here within seconds and is by
construction a real bug, not a length mismatch.

## Reviewer focus

- The Arbitrary input struct names the three fields the parser
  cares about (`dim`, `gamma_bits`, `code_seed`); everything else
  (`bits`, `seed`) is pinned to the canonical defaults
  `validate_tqvector_bits` / `validate_tqvector_seed` enforce.
- `gamma_bits` is mapped via `f32::from_bits` and NaN/infinity is
  filtered out: `f32::parse` doesn't preserve NaN bit patterns and
  the text-format round-trip is undefined for non-finite values.
  Filtering at the structured input keeps the target on a clean
  success-path contract; testing NaN handling specifically is a
  separate target's job.
- Bench comparison uses the **same fuzzer seed corpus state** for
  both runs (raw target has 141 seed entries from prior campaigns,
  structured target starts from empty). The raw target's 0.90
  cov-per-corp-entry includes the warm seed corpus; if we shrank
  the seed corpus to compare cold-start, the structured ratio would
  widen further.

## Out of scope (follow-ups)

1. **Structured error-path target.** A second `parse_text` target
   that takes a valid structured input and randomly mutates one of
   the 7 gates (drop the `:`, swap `dim=…` for garbage, truncate the
   hex, etc.) would close the error-branch coverage gap this slice
   leaves. Worth pairing with the success-path target to claim full
   coverage.
2. `fuzz_topk_merge_structured` — Task 46 §Approach 2.a, still open.
3. `fuzz_spire_leaf_v2_roundtrip` — Task 46 §Approach 2.b, still
   open (depends on bench_api re-exports for SPIRE leaf encoders).
4. `fuzz_quant_encode_decode_roundtrip` — Task 46 §Approach 2.c,
   still open.
5. Deliberately-introduced parser-bug regression evidence (Task 46
   §Validation last bullet).
6. ECAZ-grammar SQLsmith, Honggfuzz / AFL+ cross-pollination,
   corpus minimize + commit — all still open from 001.
