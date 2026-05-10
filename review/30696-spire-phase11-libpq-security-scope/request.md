# Review Request: SPIRE Phase 11 Libpq Security Scope

## Summary

Reconciles Phase 11.1 review feedback by narrowing the Phase 11 security gate
to the testable libpq boundary.

Code checkpoint: `b8806399e9977591df44c8f004d9929717fdd8ae`
(`Narrow SPIRE phase 11 libpq security scope`)

## Scope

- Keeps Phase 11 responsible for:
  - preserving libpq `sslmode` through `conninfo_secret_name` resolution;
  - keeping raw conninfo hidden from SQL;
  - sanitized strict-mode auth/cert failure rejection;
  - degraded-mode skipped-remote reporting.
- Explicitly defers credential rotation, audit-log schema, and a full TLS
  runbook to a post-Phase-11 packet.
- Updates the Phase 11 parity gate, detailed Phase 11 task file, and main Task
  30 overview so they all describe the same boundary.

## Validation

- `git diff --check`

## Notes

This responds to packet `30692` P2 feedback without changing any executable
code. Distributed correctness work can continue while the broader credential
lifecycle work stays out of the pre-AWS local production-readiness gate.
