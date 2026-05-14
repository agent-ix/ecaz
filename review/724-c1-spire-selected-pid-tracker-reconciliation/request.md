# Review Request: SPIRE Selected-PID Tracker Reconciliation

- agent: coder1
- date: 2026-05-14
- code commit: `756f84a3634cb403fe1d5d29eedf363a0558526f`
- task rows: closes `12c.7.b`, `12c.16.a`

## Summary

Tracker-only reconciliation for the current split Phase 12c task file.

The selected-PID CustomScan payload test already exists from packet `692`,
and batch-1 reviewer feedback accepted it as closing both `12c.7.b` and
`12c.16.a`. This checkpoint updates
`plan/tasks/task30-phase12c-spire-test-coverage.md` so the live tracker matches
that accepted coverage.

## Evidence

- `src/tests/custom_scan.rs:408`
  - `test_ec_spire_customscan_selected_pid_payloads`
  - Builds matching coordinator and loopback remote SPIRE indexes over eight
    known rows.
- `src/tests/custom_scan.rs:419`
  - Remote rows use stable `id` and `title` payloads for IDs 101 through 108.
- `src/tests/custom_scan.rs:460`
  - Coordinator rows use matching vectors and stable payloads for IDs 1 through
    8.
- `src/tests/custom_scan.rs:490`
  - Asserts coordinator and remote active epochs match, then captures the
    selected coordinator leaf PIDs.
- `src/tests/custom_scan.rs:506`
  - Rewrites all selected coordinator PIDs to remote node 2, making the
    selected-PID mapping explicit.
- `src/tests/custom_scan.rs:524`
  - `payload_rows_for_pids` calls
    `ec_spire_remote_search_tuple_payload(... selected_pids ..., 8, 'strict',
    ARRAY['id','title'])`.
- `src/tests/custom_scan.rs:555`
  - Per-PID probes assert every one-PID request returns rows only for that PID.
- `src/tests/custom_scan.rs:568`
  - The all-PID probe asserts observed PIDs exactly equal the selected PID set,
    and the all-PID rows equal the union of the per-PID probes.
- `src/tests/custom_scan.rs:579`
  - Forces the CustomScan plan with `LIMIT 8`.
- `src/tests/custom_scan.rs:605`
  - Executes the CustomScan `LIMIT 8`, sorts `(id, title)`, and compares it to
    the selected remote payload rows.

Reviewer feedback in
`review/31080-spire-phase12c-batch1-feedback/feedback/2026-05-14-001-reviewer.md`
also records packet `692` as closing `12c.7.b` and `12c.16.a`.

## Changes

- Checked the four `12c.7.b` bullets in the current split task file.
- Checked the four duplicated semantic-tightening bullets under `12c.16.a`.
- No test code changed in this checkpoint.

## Validation

- `git diff --check -- plan/tasks/task30-phase12c-spire-test-coverage.md`
  - Passed.
- No compile or runtime test was run for this tracker-only checkpoint; the
  request points to already-reviewed test evidence only.

## Review Focus

- Confirm this reconciliation is against the current split task file, not the
  older pre-split tracker.
- Confirm packet `692` plus the cited source-test evidence satisfy both
  `12c.7.b` and the duplicated `12c.16.a` semantic row.
