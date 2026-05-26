# Task 46 kickoff slice: first structured-fuzz target (Arbitrary)

## Scope

Opens Task 46 (Structure-Aware Fuzzing and ECAZ-Grammar SQLsmith,
`plan/tasks/46-structure-aware-and-grammar-fuzzing.md`). This packet
lands the first structure-aware target alongside an existing raw-byte
target so reviewers can verify the Task 46 §Validation criterion
("Structured fuzz targets achieve ≥ 5× higher feature/edge coverage
per second than the equivalent raw-byte targets") on the same surface.

Validation head: `a079f1e8bc10d4137ce0e65adf3aef77a04542d1` (main).

## What changed

- `fuzz/Cargo.toml` — adds `arbitrary = { version = "1", features = ["derive"] }`
  and registers the new bin `fuzz_unpack_mse_structured`.
- `fuzz/src/lib.rs` — re-exports `pack_mse_indices` from the
  `bench_api` module so structured targets can drive the encode side.
- `fuzz/fuzz_targets/unpack_mse_structured.rs` — new structure-aware
  target. Uses `arbitrary::Arbitrary` to produce a valid-shape input
  `(dim ∈ 1..=2048, bits ∈ 2..=7, indices: dim values masked to bits)`
  and asserts the round-trip property `unpack(pack(indices)) == indices`.

No production code change. Test-/tooling-only addition.

## Why this surface first

The sibling raw-byte target `fuzz/fuzz_targets/unpack_mse.rs` is the
canonical example of the structural-waste problem Task 46 describes:

```rust
let expected_len = (dim * bits_per_index as usize).div_ceil(8);
if packed.len() != expected_len {
    return;
}
```

Almost every input the fuzzer mutates fails that length check and is
rejected without ever hitting the unpack routine. The structured
sibling encodes the valid-shape relationship into the input type so the
fuzzer always pays its execution budget into the actual unpack path.

Using the same surface (MSE index pack/unpack) keeps the comparison
apples-to-apples for the Task 46 §Validation coverage criterion and
gives the reviewer a single grep target across the artifact logs.

## Evidence

Both runs use `cargo fuzz run <target> -- -max_total_time=10
-print_final_stats=1` on the same machine, back-to-back, at the same
head SHA.

- `artifacts/fuzz-unpack-mse-structured-10s.log`
  - **cov 213, ft 681, corp 85**, exec/s 257k, 57 `NEW` units added in
    the 10s window.
- `artifacts/fuzz-unpack-mse-raw-baseline-10s.log`
  - **cov 40, ft 91, corp 24**, exec/s 816k, **0** `NEW` units added in
    the 10s window (the raw target is already saturated against its
    seed corpus).

Coverage delta vs. the raw-byte sibling at 10s:

| metric | raw | structured | ratio |
|---|---:|---:|---:|
| cov  | 40  | 213 | 5.33× |
| ft   | 91  | 681 | 7.48× |
| corp | 24  | 85  | 3.54× |
| new in 10s | 0 | 57 | n/a (raw saturated) |

The 5.33× coverage and 7.48× feature ratios both clear the Task 46
§Validation ≥ 5× threshold on this surface; exec/sec drops because
each structured input pays for an Arbitrary draw plus a real
encode/decode round-trip, but the structural-rejection waste is
eliminated.

## Build verification

- `cargo check --manifest-path fuzz/Cargo.toml --bin fuzz_unpack_mse_structured`
  passes with no errors (warnings are pre-existing in the `ecaz-fuzz`
  lib's path-included modules and not introduced by this packet).
- `cargo fuzz build fuzz_unpack_mse_structured` succeeds under the
  nightly toolchain.

## Reviewer focus

- The structured target uses `arbitrary::Arbitrary` derive on a small
  input struct; the conversion from raw `arbitrary` u8/u16 fields to
  the valid `(dim, bits, indices)` tuple lives inside the target so
  the property assertion is the only behavioural surface.
- The round-trip assertion (`pack(unpack) == identity`) is the
  cleanest decoder/encoder invariant — any divergence reported by
  libFuzzer is by construction a real bug, not a structural mismatch.
- Coverage numbers above are reproducible from the two artifact logs
  with `grep -E "cov:|ft:|corp:|new_units" *.log`.

## Out of scope (follow-up packets)

- `fuzz_topk_merge_structured` (Task 46 §Approach 2.a) — a second
  structured target on a different surface.
- `fuzz_spire_leaf_v2_roundtrip` and `fuzz_quant_encode_decode_roundtrip`
  (Task 46 §Approach 2.b/2.c).
- ECAZ-grammar SQLsmith (`crates/ecaz-sqlgen`) — separate workstream.
- Honggfuzz / AFL+ integration (Task 46 §Approach 4).
- `make fuzz-corpus-minimize` lane + committed corpora (Task 46
  §Approach 5). The kickoff slice does not commit the corpus
  directory; `fuzz/corpus` remains gitignored until the cmin lane lands.
- Deliberately-introduced parser bug regression evidence (Task 46
  §Validation last bullet) — scheduled for the same follow-up that
  lands the topk and roundtrip targets.
