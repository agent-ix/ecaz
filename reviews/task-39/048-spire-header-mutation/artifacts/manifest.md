# Packet 048 Artifacts Manifest

| Artifact | Command | Key result |
| --- | --- | --- |
| `header-mutants-enumerated.txt` | `cargo mutants --Zmutate-file /Users/peter/dev/tqvector/src/am/ec_spire/storage/header.rs --list` | 35 mutations enumerated |
| `run-spire-mutations.py` | generic per-file verification helper (parses enumeration, applies mutation textually, runs careful suite, records KILLED/MISSED, reverts) | (script body) |
| `manual-verification.log` | output of running the helper above | **35 KILLED, 0 MISSED, 0 PATCH-FAIL** |
| `post-verification-tests.log` | full `cargo test --manifest-path hardening/careful/Cargo.toml --lib` after restoring header.rs | `test result: ok. 534 passed; 0 failed` |
