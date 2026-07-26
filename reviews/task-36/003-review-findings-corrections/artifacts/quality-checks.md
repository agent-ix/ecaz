# Quality-check summary

Head: `15e3831c13b65b488fea1c0f1ac1da8d46e321f1`

## Whitespace

Command: `git diff --check`

Result: PASS before each correction commit and after the three correction
commits.

## Rustfmt

Command: `cargo fmt --all -- --check`

Result: FAIL on inherited repository-wide formatting drift. The complete
output is in `cargo-fmt-check.log`. Making this global check green would
rewrite many files and older regions outside Task 36.

## Clippy

Command: `cargo clippy --lib --features bench -- -D warnings`

Result: FAIL on one pre-existing diagnostic outside Task 36:

```text
error: manual checked division
  --> src/am/ec_ivf/quantizer.rs:695:34
  = note: `-D clippy::manual-checked-ops` implied by `-D warnings`
```

No diagnostic referenced `Makefile`, `docs/hardening.md`,
`plan/tasks/36-simd-scalar-differential.md`, `src/quant/prod.rs`, or
`src/quant/int8_approx32/mod.rs`.
