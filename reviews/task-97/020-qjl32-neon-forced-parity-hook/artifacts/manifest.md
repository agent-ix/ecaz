# Task 97 Packet 020 Artifact Manifest

- head SHA: `cc9df4553ddff2eab5a8b2a8a4b317e8e9b25517`
- task bucket: `reviews/task-97/020-qjl32-neon-forced-parity-hook/`
- timestamp: `2026-06-10T06:21:08Z`
- lane: coder-1 LUT lane
- quant: `turboquant_qjl`
- storage format / AM surface: not applicable; unit-test hook only
- isolated one-index-per-table or shared-table surface: not applicable
- CI: not run
- AWS: not run

## Artifacts

| Artifact | Command | Result |
| --- | --- | --- |
| `cargo-fmt-check.log` | `cargo fmt --check` | passed; stable rustfmt emitted the existing unstable-option warnings |
| `cargo-test-qjl32.log` | `cargo test qjl32 --lib -- --color never` | passed; 12 passed, 0 failed |
| `git-diff-check.log` | `git diff --check` | passed |

## Key Result Lines

From `cargo-test-qjl32.log`:

```text
test quant::qjl32::tests::qjl32_neon_block32_matches_pre_slice_scorer_tolerance_when_available ... ok
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 2061 filtered out; finished in 0.07s
```

The NEON test is target-gated. On this Intel host it compiles and returns early,
which preserves local coverage. On an AArch64 NEON host it calls the NEON scorer
directly and asserts `Isa::Neon`, giving the future Graviton 4 packet a concrete
hook for the Packet 007 carry-forward requirement that NEON parity execute for
real.
