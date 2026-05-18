---
topic: spire-review-followups
agent: coder1
role: coder
model: gpt-5
date: 2026-05-12
seq: 30925
stage: phase-12.4
status: open
---

# Review Request: SPIRE Review Follow-Ups

## Scope

Please review commit `28f304fb` (`Address SPIRE review follow-ups`).

This responds to P3 feedback from `30920`, `30921`, and `30923`.

## What Changed

- Added a short code comment documenting the DELETE helper's
  `node_id = -1` no-routing sentinel as a result-row sentinel that is never
  written to `ec_spire_placement`.
- Changed the remote `PREPARE TRANSACTION` capacity classifier to match
  SQLSTATE `55000` directly before message matching, while keeping
  message-gated matching for broader resource SQLSTATEs.
- Added unit coverage for the SQLSTATE-only `55000` path.
- Changed descriptor-registration prepared-capacity preflight messaging from
  `WARNING` to NOTICE-level output.
- Updated ADR-069 and `docs/SPIRE_DIAGNOSTICS.md` to say the registration
  preflight emits a NOTICE-level operator message visible to the client/logs.

## Evidence

See `artifacts/manifest.md`.

Validation run against `28f304fb2467c2ad5fc3b9d63fef68c7b0a4385f`:

- `git diff --check HEAD^ HEAD`
- `cargo fmt --check`
- `cargo test --features pg18 --no-default-features prepare_transaction_capacity_classifier_matches_postgres_errors`
- `cargo pgrx test pg18 test_ec_spire_remote_node_descriptor_registration_contract`

## Review Focus

- Confirm SQLSTATE `55000` direct matching is acceptable in this
  PREPARE-only wrapper context.
- Confirm NOTICE-level registration preflight output is the right operator
  surface.
- Confirm the DELETE sentinel comment is sufficient for the `30920` P3.
