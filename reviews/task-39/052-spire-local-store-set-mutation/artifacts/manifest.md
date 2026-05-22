# Packet 052 Artifacts Manifest

| Artifact | Command | Key result |
| --- | --- | --- |
| `local-store-set-mutants-enumerated.txt` | `cargo mutants --Zmutate-file /Users/peter/dev/tqvector/src/am/ec_spire/storage/local_store_set.rs --list` | 19 mutations enumerated |
| `manual-verification.log` | `python3 /tmp/run_spire_mutations_v2.py src/am/ec_spire/storage/local_store_set.rs 052-spire-local-store-set-mutation 0` | **19 KILLED, 0 MISSED** |
| `post-verification-tests.log` | full `cargo test --manifest-path hardening/careful/Cargo.toml --lib` after revert | `test result: ok. 550 passed; 0 failed` |
