# Packet 051 Artifacts Manifest

| Artifact | Command | Key result |
| --- | --- | --- |
| `local-store-mutants-enumerated.txt` | `cargo mutants --Zmutate-file /Users/peter/dev/tqvector/src/am/ec_spire/storage/local_store.rs --list` | 87 mutations enumerated |
| `manual-verification.log` | `python3 /tmp/run_spire_mutations_v2.py src/am/ec_spire/storage/local_store.rs 051-spire-local-store-mutation 0` | 75 KILLED, 12 MISSED (10 killed by new test, 2 equivalent) |
| `post-verification-tests.log` | full `cargo test --manifest-path hardening/careful/Cargo.toml --lib` after restoring local_store.rs | `test result: ok. 550 passed; 0 failed` |
