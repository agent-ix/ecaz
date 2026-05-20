# Packet 049 Artifacts Manifest

| Artifact | Command | Key result |
| --- | --- | --- |
| `helpers-mutants-enumerated.txt` | `cargo mutants --Zmutate-file /Users/peter/dev/tqvector/src/am/ec_spire/storage/helpers.rs --list` | 190 mutations enumerated |
| `helpers-survivors-revalidated.txt` | first-pass survivors that the second-pass run re-tested against the new killing tests | 41 survivors from the initial cumulative-test run |
| `run-spire-mutations.py` | generic per-file verification helper (consolidated from packets 046/047/048; this packet fixed the `apply_body` bug — one-line-body mutations now correctly target the function whose body they belong to) | (script body) |
| `manual-verification-survivor-revalidation.log` | re-verify of the 41 survivors against the cumulative test surface (including the 15 new killing tests in this packet) | **24 KILLED, 17 MISSED — all 17 documented equivalent mutants** |
| `post-verification-tests.log` | full `cargo test --manifest-path hardening/careful/Cargo.toml --lib` after restoring helpers.rs | `test result: ok. 549 passed; 0 failed` |

Provenance:

- Task bucket: `reviews/task-39/`.
- Packet path: `reviews/task-39/049-spire-helpers-mutation/`.
- Surface: pure-Rust mutation evaluation of
  `src/am/ec_spire/storage/helpers.rs` against the existing
  shadow-careful test suite plus 15 new killing tests in
  `src/am/ec_spire/storage/tests/helpers.rs`.
- No production code change; mutations applied transiently to the
  production source and reverted from a backup before commit.
- 2 additional body-replacement mutations (`73:5
  is_delete_delta_assignment -> true` and `80:5 validate_leaf_v2_header
  -> Ok(())`) were uncovered after the `apply_body` fix; both verified
  killed by the corresponding new tests via manual apply + revert
  cycles outside the survivor-revalidation log.

See `triage.md` for the per-mutation map (killing tests + equivalent
mutant rationale).
