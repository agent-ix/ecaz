# Validation

- `cargo check --offline --all-targets --no-default-features --features pg18`: passed at code head `a6289dddf`.
- Focused pgrx test harness invocation for the new union and BW=256 tests: stopped after several minutes with no emitted result; no failure was reported.
- `git diff --check`: passed before the code checkpoint.
