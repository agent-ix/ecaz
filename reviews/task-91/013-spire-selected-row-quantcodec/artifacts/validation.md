# Validation Log: Task 91 Packet 013

Head SHA: `3909a8511d2fc369596930a91750ae48e8b454a7`

## Formatting

Command:

```sh
cargo fmt
```

Result: completed. The repository rustfmt configuration emitted the existing stable-toolchain warnings for unstable import grouping settings.

## Focused Tests

Command:

```sh
cargo test --lib am::ec_spire::scan::tests::selected_row_quant_codec_helper_matches_prepared_assignment_scorer --no-default-features --features pg18
```

Result:

```text
running 1 test
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 2034 filtered out; finished in 0.06s
```

Command:

```sh
cargo test --lib am::ec_spire::scan::tests::collect_quantized_routed_probe_candidates_matches_prepared_assignment_scorer --no-default-features --features pg18
```

Result:

```text
running 1 test
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 2034 filtered out; finished in 0.07s
```

Command:

```sh
cargo test --lib am::ec_spire::scan::tests::select_leaf_block_row_ranges --no-default-features --features pg18
```

Result:

```text
running 2 tests
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 2033 filtered out; finished in 0.00s
```

## Diff Check

Command:

```sh
git diff --check
```

Result: passed.
