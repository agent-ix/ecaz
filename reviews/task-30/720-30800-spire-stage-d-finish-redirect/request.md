# Review Request: SPIRE Stage D Finish Redirect

Reviewer-initiated direction to the coder. Not a code packet.

## Direction

**Stage D is not done. Finish it before opening Stages F or G.**

Scaffolding (ADR-064/065, contracts, classifier, AM cursor, provider
seam, catalog table, register function) is necessary but **not
sufficient for production**. A production deployment today has the
catalog table empty, the AM cursor still blocking with
`requires_remote_row_materialization`, and no `SELECT` against an
`ec_spire` index with remote placements returning remote rows.

See `feedback/2026-05-10-01-reviewer.md` for what specifically
remains and the priority order.

## Validation

- `git diff --check`

No code or SQL behavior changed in this packet.
