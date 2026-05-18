# Task 41 Invariant 2 Action Closeout

## Scope

This packet closes out the promoted reviewer action list from
`reviews/task-41/147-invariant2-completion-audit/feedback/2026-05-17-02-reviewer.md`.

Code commit under direct review:

- `52a6e1a5e68ff9ac7357a76ae1d829bdc375a7bf` - `Fix HNSW read helper all-targets compile`

The commit fixes all-targets debug/test-only fallout from the HNSW read-helper result unification:

- `src/am/ec_hnsw/shared.rs`: the debug collect path now unwraps the shared helper result before pushing `Vec<u8>` values.
- `src/am/ec_hnsw/vacuum.rs`: the debug/test-only null pointer uses a fully qualified `std::ptr::null_mut()` path.

## Reviewer Action Mapping

- A, B: writable page tuple helpers were split/consolidated in packet `150-writable-page-tuple-helper-split`, commit `9b7ec742`.
- C: new closure helpers now use higher-ranked closure bounds in packets `148`, `149`, `150`, and `151`.
- D: HNSW page tuple read helpers now return `Result<Option<R>, String>` from the shared helper in packet `151`, with this packet fixing all-targets compile fallout.
- E: detoast guards were consolidated into `src/am/common/detoast.rs` in packet `148`, commit `268d4f63`.
- F: `scan_debug` now delegates to the shared HNSW page tuple helper in packet `151`.
- G: HNSW DSM init mutable-slice borrows now stay inside scoped HRTB callbacks in packet `149`, commit `705b2c94`.
- H: `uuid_source_identity_payload` now exposes its fixed-size byte view through a scoped HRTB callback in packet `149`.
- I: the detoast ERROR contract wording was narrowed in packet `148`.

## Validation

- `cargo fmt --all --check` passed. See `artifacts/cargo-fmt-check.log`.
- `cargo check --all-targets --no-default-features --features pg18` passed with pre-existing warnings. See `artifacts/cargo-check-all-targets-pg18.log`.
- `git diff --check 52a6e1a5^ 52a6e1a5` passed. See `artifacts/git-diff-check-code-commit.log`.
- `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings` was attempted and failed on broad existing lint debt outside this slice, including unused reexports in `src/am/mod.rs`, existing `let_and_return`, `type_complexity`, `too_many_arguments`, and test lint issues. See `artifacts/cargo-clippy-all-targets-pg18.log`.
- A final memory/lifetime inventory was recorded in `artifacts/final-memory-lifetime-inventory.log`.

## Reviewer Focus

Please confirm the promoted A-I feedback items are covered by packets `148` through `152`, and that the clippy limitation is acceptable as unrelated existing lint debt rather than scope for Task 41 invariant 2.
