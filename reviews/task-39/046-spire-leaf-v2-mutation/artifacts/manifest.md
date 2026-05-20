# Packet 046 Artifacts Manifest

| Artifact | Command | Key result |
| --- | --- | --- |
| `leaf-v2-mutants-enumerated.txt` | `cargo mutants --Zmutate-file /Users/peter/dev/tqvector/src/am/ec_spire/storage/leaf_v2.rs --list` | 14 mutations enumerated |
| `initial-mutants-run.log` | `cargo mutants --package ecaz-careful-hardening --file hardening/careful/src/../../../src/am/ec_spire/storage/leaf_v2.rs --output reviews/task-39/046-spire-leaf-v2-mutation/artifacts/initial -j 4` | `Found 0 mutants to test  WARN No mutants found under the active filters` — automation blocked because `include!` content is invisible to cargo-mutants' module discovery |
| `manual-verification.log` | per-mutation `Edit` / `sed` / Python apply + `cargo test --manifest-path hardening/careful/Cargo.toml --lib` per focused filter, then revert from `/tmp/leaf_v2_original.rs` | **14 KILLED, 0 MISSED, 0 TIMEOUTS** |
| `post-verification-tests.log` | full `cargo test --manifest-path hardening/careful/Cargo.toml --lib` after restoring leaf_v2.rs | `test result: ok. 529 passed; 0 failed` |

Provenance:

- Task bucket: `reviews/task-39/`.
- Packet path: `reviews/task-39/046-spire-leaf-v2-mutation/`.
- Head SHA at packet commit time.
- Surface: pure-Rust mutation evaluation of
  `src/am/ec_spire/storage/leaf_v2.rs` against the existing
  shadow-careful test suite (packets 029 + 044 supplied the
  killing tests).
- No production code change; mutations applied transiently and
  reverted from a backup before commit.
