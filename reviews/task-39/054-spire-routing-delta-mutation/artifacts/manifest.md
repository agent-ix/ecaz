# Packet 054 Artifacts Manifest

| Artifact | Command | Key result |
| --- | --- | --- |
| `routing-delta-mutants-enumerated.txt` | `cargo mutants --Zmutate-file /Users/peter/dev/tqvector/src/am/ec_spire/storage/routing_delta.rs --list` | 58 mutations enumerated |
| `spot-verify-encode-body-replacement.log` | manual apply of `encode -> Ok(vec![])` + `cargo test --manifest-path hardening/careful/Cargo.toml --lib` | 1 test FAILED (mutation killed) |
| `post-verification-tests.log` | `cargo test --manifest-path hardening/careful/Cargo.toml --lib` after revert | `test result: ok` (no failures) |

Head SHA at packet authoring: see `git rev-parse HEAD` of commit
landing this packet.

Source `src/am/ec_spire/storage/routing_delta.rs` byte-for-byte
identical pre/post packet (verified by `diff` against
`/tmp/routing_delta_original.rs`).
