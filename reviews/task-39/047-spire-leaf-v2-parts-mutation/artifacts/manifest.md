# Packet 047 Artifacts Manifest

| Artifact | Command | Key result |
| --- | --- | --- |
| `leaf-v2-parts-mutants-enumerated.txt` | `cargo mutants --Zmutate-file /Users/peter/dev/tqvector/src/am/ec_spire/storage/leaf_v2_parts.rs --list` | 68 mutations enumerated |
| `run-leaf-v2-parts-mutations.py` | hand-written helper that parses each enumeration line, applies the mutation textually, runs the careful suite with a `leaf_v2`-filtered `cargo test`, records KILLED/MISSED, reverts from `/tmp/leaf_v2_parts_original.rs` | (script body) |
| `manual-verification.log` | output of two runs of the helper above, before and after the new killing tests landed | **67 KILLED, 1 MISSED (equivalent), 0 PATCH-FAIL** |
| `post-verification-tests.log` | full `cargo test --manifest-path hardening/careful/Cargo.toml --lib` after restoring leaf_v2_parts.rs | `test result: ok. 534 passed; 0 failed` |

Provenance:

- Task bucket: `reviews/task-39/`.
- Packet path: `reviews/task-39/047-spire-leaf-v2-parts-mutation/`.
- Surface: pure-Rust mutation evaluation of
  `src/am/ec_spire/storage/leaf_v2_parts.rs` against the existing
  shadow-careful test suite plus 5 new killing tests in
  `src/am/ec_spire/storage/tests/leaf.rs`.
- No production code change; mutations applied transiently to the
  production source and reverted from a backup before commit.
