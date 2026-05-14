# Review Request: SPIRE Phase 12b Audit Decisions

## Summary

Packet 31017 processes the midphase audit feedback from
`review/31050-spire-phase12b-midphase-audit/feedback/2026-05-13-001-reviewer.md`.

The tracker now makes two decisions explicit before more 12b.2 fixture
evacuation lands:

- The hard 2,500-line cap applies to production files under
  `src/am/ec_spire/`, not to the temporary `src/tests/` fixture sink.
- The old `src/lib.rs <2,000 lines` target is replaced by a fixture-body
  absence requirement, because pgrx registration, re-exports, pg_extern
  wrappers, and pg_test scaffold remain in `src/lib.rs`.

This packet also adds a one-line comment clarifying that
`tuple_transport_status: ready` in `ExplainCustomScan` is a stable shape
marker, not a live transport probe.

Code checkpoint: `05598eb538bd7883b845596e6716205a0e94e1f3`

## Review Focus

- Confirm the tracker decisions address the midphase audit's two
  non-blocking flags.
- Confirm the revised exit criteria are explicit enough for future
  closeout review.
- Confirm the `ExplainCustomScan` comment does not imply a behavior
  change.

## Validation

- `cargo fmt --check`
- `git diff --check`
- `wc -l src/tests/mod.rs src/tests/remote_search.rs src/tests/dml_frontdoor.rs src/lib.rs src/am/ec_spire/custom_scan/explain.rs`
- `rg -n 'Midphase audit decision|tuple_transport_status|stable shape marker|src/tests/.*2,500|src/lib.rs.*fixture' plan/tasks/task30-phase12b-spire-cleanup.md src/am/ec_spire/custom_scan/explain.rs`

No PG18 fixture was run for this packet because the code change is a
comment-only clarification and the rest is tracker text.

Artifacts and key result lines are recorded in
`review/31017-spire-phase12b-audit-decisions/artifacts/manifest.md`.
