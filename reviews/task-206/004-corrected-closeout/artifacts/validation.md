# Validation

- `cargo check --offline --all-targets --no-default-features --features pg18`: passed.
- Focused pgrx test harness invocation for the new BW=256 and partition-union tests: stopped after several minutes with no emitted result; no failure was reported.
- `git diff --check`: passed before the code checkpoint.
