# Task 39 RaBitQ mutation checkpoint

## Summary

Runs the first RaBitQ mutation campaign and adds focused tests for the main
survivor classes in `src/quant/rabitq.rs`.

This is a checkpoint packet, not final Task 39 closeout: the final mutation run
still has 9 missed mutants and 2 timeouts, all recorded in `triage.md`.

## Code under review

- Commit: `ce378e208dfc151ac82acb3e7f8d3982ce3090cd`
- Changed file: `src/quant/rabitq.rs`

## Mutation results

Initial run:

- Command: `cargo mutants --in-place --package ecaz-careful-hardening --file hardening/careful/src/../../../src/quant/rabitq.rs --output reviews/task-39/026-rabitq-mutation/artifacts/initial/rabitq.rs.mutants`
- Result: 456 mutants tested in 38m: 118 missed, 317 caught, 21 unviable.

Intermediate rerun:

- Command: `cargo mutants --in-place --package ecaz-careful-hardening --file hardening/careful/src/../../../src/quant/rabitq.rs --output reviews/task-39/026-rabitq-mutation/artifacts/rerun/rabitq.rs.mutants`
- Result: 456 mutants tested in 44m: 27 missed, 408 caught, 21 unviable.

Final run:

- Command: `cargo mutants --in-place --package ecaz-careful-hardening --file hardening/careful/src/../../../src/quant/rabitq.rs --output reviews/task-39/026-rabitq-mutation/artifacts/final/rabitq.rs.mutants`
- Result: 455 mutants tested in 74m: 9 missed, 423 caught, 21 unviable, 2 timeouts.

## Validation

- `cargo test --manifest-path hardening/careful/Cargo.toml --lib rabitq -- --nocapture`
  passed: 25 RaBitQ tests.
- `cargo check --all-targets --no-default-features --features pg18,bench`
  passed with pre-existing warnings.
- `git diff --check` passed.

## Notes

- `--in-place` was used because this checkout has large unrelated untracked
  fixture directories; scratch-copy mutation would copy those into each mutant
  workspace.
- The remaining survivors are carried as explicit follow-up work in
  `triage.md` so this can land as the requested main checkpoint.
