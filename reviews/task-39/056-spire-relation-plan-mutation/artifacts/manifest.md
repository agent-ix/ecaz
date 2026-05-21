# Packet 056 Artifacts Manifest

| Artifact | Command | Key result |
| --- | --- | --- |
| `relation-plan-mutants-enumerated.txt` | `cargo mutants --Zmutate-file /Users/peter/dev/tqvector/src/am/ec_spire/storage/relation_plan.rs --list` | 13 mutations enumerated |
| `spot-verify-plan-local-store-relations-empty.log` | manual apply of `plan_local_store_relations -> Ok(vec![])` + `cargo test --manifest-path hardening/careful/Cargo.toml --lib` | mutation killed |
| `post-verification-tests.log` | `cargo test --manifest-path hardening/careful/Cargo.toml --lib` after revert | `test result: ok` |

Head SHA at packet authoring: see `git rev-parse HEAD` of commit
landing this packet.

Source `src/am/ec_spire/storage/relation_plan.rs` byte-for-byte
identical pre/post packet (verified by `diff` against
`/tmp/relation_plan_original.rs`).
