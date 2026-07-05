# Packet 053 Artifacts Manifest

| Artifact | Command | Key result |
| --- | --- | --- |
| `vec-id-mutants-enumerated.txt` | `cargo mutants --Zmutate-file /Users/peter/dev/tqvector/src/am/ec_spire/storage/vec_id.rs --list` | 148 mutations enumerated |
| `manual-verification.log` | partial run of `run-spire-mutations.py` (killed early due to target/-bloat slowdown) | 31 verdicts (26 KILLED, 5 MISSED — all equivalent constant-arithmetic) |
| `post-verification-tests.log` | `cargo test` after revert | `test result: ok. 550 passed; 0 failed` |
