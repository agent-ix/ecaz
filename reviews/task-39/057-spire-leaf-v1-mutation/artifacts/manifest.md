# Packet 057 Artifacts Manifest

| Artifact | Command | Key result |
| --- | --- | --- |
| `leaf-v1-mutants-enumerated.txt` | `cargo mutants --Zmutate-file /Users/peter/dev/tqvector/src/am/ec_spire/storage/leaf_v1.rs --list` | 10 mutations enumerated |
| `spot-verify-encode-body-replacement.log` | manual apply of `SpireLeafPartitionObject::encode -> Ok(vec![])` + `cargo test --manifest-path hardening/careful/Cargo.toml --lib` | mutation killed |
| `post-verification-tests.log` | `cargo test --manifest-path hardening/careful/Cargo.toml --lib` after revert | `test result: ok` |

Source `src/am/ec_spire/storage/leaf_v1.rs` byte-for-byte identical
pre/post packet (verified by `diff` against
`/tmp/leaf_v1_original.rs`).
