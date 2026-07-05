# Review Request: Task 47 Recall And Cost Gate Entrypoints

## Scope

This packet requests review for commit
`80d0fe0c002edd3ba3466d8fe2694b5dbcb59410`, which adds the first Task 47
recall and cost gate entrypoints:

- `make recall-gate`, `make recall-gate-full`, `make cross-am-gate`, and
  `make cost-gate`;
- suite configs under `fixtures/gates/`;
- `docs/recall-floors.md` with fixture assumptions, current floors, and the
  limitation of the first cost gate.

## Review Focus

- Whether these suite configs are a good first burn-in surface for Task 47.
- Whether the floor values and report-first language are clear enough to avoid
  treating initial IVF/DiskANN values as final release criteria.
- Whether the cost-gate positivity check is acceptable as a first wiring slice,
  with baseline JSON / per-node cost drift left to a follow-up.

## Validation

Packet-local artifacts are in `artifacts/`.

- `make-n-task47-gates.log`: all four new Make targets expand to
  `ecaz bench suite run`.
- `audit-recall-gate-small.log`: small recall suite audit passed.
- `audit-recall-gate-full.log`: full recall suite audit passed.
- `audit-cross-am-gate-small.log`: cross-AM suite audit passed.
- `audit-cost-gate-small.log`: cost suite audit passed.

No live PG18 recall or EXPLAIN runs were executed in this packet. The suite
assumes preloaded `task47_*_real10k` prefixes as documented in
`docs/recall-floors.md`.
