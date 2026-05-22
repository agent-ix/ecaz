# Packet 050 Artifacts Manifest

| Artifact | Command | Key result |
| --- | --- | --- |
| `assignment-mutants-enumerated.txt` | `cargo mutants --Zmutate-file /Users/peter/dev/tqvector/src/am/ec_spire/storage/assignment.rs --list` | 54 mutations enumerated |
| `manual-verification.log` | partial run of `run-spire-mutations.py` (killed early due to target/-bloat slowdown — see triage.md) | 9 verdicts (7 KILLED, 2 MISSED — both equivalent) |
| `post-verification-tests.log` | full `cargo test --manifest-path hardening/careful/Cargo.toml --lib` after restoring assignment.rs | `test result: ok. 549 passed; 0 failed` |

Honest provenance:

- The verification is partial (9/54). The 305 GB workspace target/
  has slowed cargo's per-mutation dep check to 7-16 min, making full
  per-file verification impractical.
- 2 survivors identified, both spot-verified as functionally
  equivalent mutants (capacity-hint-only return values).
- Remaining 45 mutations are extrapolated against the methodology
  established in packets 046-049.
- See `triage.md` for the spot-verification details and the
  equivalent-mutant rationale.
- Required follow-up: full re-verification after `target/` cleanup or
  on a fresh build state.
