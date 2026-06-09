# Validation Log: Task 91 Packet 014

Head SHA: `528f3d2cd90e98c8f6d0d54f2947d0a294418419`

## Formatting

Command:

```sh
cargo fmt
```

Result: completed. The repository rustfmt configuration emitted the existing stable-toolchain warnings for unstable import grouping settings.

## Focused Tests

Command:

```sh
cargo test --lib am::ec_spire::scan::tests::column_payload_quant_codec_batch_helper_matches_prepared_assignment_scorer --no-default-features --features pg18
```

Result:

```text
running 1 test
test am::ec_spire::scan::tests::column_payload_quant_codec_batch_helper_matches_prepared_assignment_scorer ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 2035 filtered out; finished in 0.07s
```

Command:

```sh
cargo test --lib am::ec_spire::scan::tests::selected_row_quant_codec_helper_matches_prepared_assignment_scorer --no-default-features --features pg18
```

Result:

```text
running 1 test
test am::ec_spire::scan::tests::selected_row_quant_codec_helper_matches_prepared_assignment_scorer ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 2035 filtered out; finished in 0.06s
```

Command:

```sh
cargo test --lib am::ec_spire::scan::tests::collect_quantized_routed_probe_candidates_matches_prepared_assignment_scorer --no-default-features --features pg18
```

Result:

```text
running 1 test
test am::ec_spire::scan::tests::collect_quantized_routed_probe_candidates_matches_prepared_assignment_scorer ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 2035 filtered out; finished in 0.06s
```

## Diff Check

Command:

```sh
git diff --check
```

Result: passed.
