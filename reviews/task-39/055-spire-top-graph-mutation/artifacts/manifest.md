# Packet 055 Artifacts Manifest

| Artifact | Command | Key result |
| --- | --- | --- |
| `top-graph-mutants-enumerated.txt` | `cargo mutants --Zmutate-file /Users/peter/dev/tqvector/src/am/ec_spire/storage/top_graph.rs --list` | 62 mutations enumerated |
| `spot-verify-encode-body-replacement.log` | manual apply of `SpireTopGraphPartitionObject::encode -> Ok(vec![0])` + `cargo test --manifest-path hardening/careful/Cargo.toml --lib` | mutation killed |
| `post-verification-tests.log` | `cargo test --manifest-path hardening/careful/Cargo.toml --lib` after revert | `test result: ok` |

Head SHA at packet authoring: see `git rev-parse HEAD` of commit
landing this packet.

Source `src/am/ec_spire/storage/top_graph.rs` byte-for-byte identical
pre/post packet (verified by `diff` against
`/tmp/top_graph_original.rs`).
