# Review request: recovery-state remediation and conditional closeout

## Status

The three conditions in packet 059's outside `ACCEPT-WITH-CONDITIONS` decision
are closed. Task 179 is marked done by this packet, as authorized by that
decision; no merge to `main` is requested or performed.

Final production/test source SHA: `87ea7a42753df547a11eec96d8bced90738ac66c`.

## Packet-017 P2 remediation

Code checkpoint `772728b23` closes the durable registration skew identified in
packet 017 and carried into packet 059:

- `ec_distann_recover_epoch_publish` locks the registration before any
  participant RPC;
- a new recovery accepts only `Pending + Decided`;
- exact replay accepts only `Activated|Applied + Published`;
- every `Pending -> Applied`, `Pending -> Activated`, and
  `Decided -> Published` update uses `RETURNING 1` and requires exactly one
  affected row; and
- the active-pointer, decision, disposition, and registration changes remain
  in one transaction, so any failed count rolls the entire transition back.

The real-backend multi-epoch test now injects `Pending + Ready` skew, proves
`EC_EPOCH_STATE` before activation, restores `Decided`, and then continues
through the existing post-ack fault, recovery, replay, retirement, reclaim,
force-retire, abandonment, and cancellation sequence.

## Aggregate test maintenance

The reviewer-requested aggregate run exposed stale Task 179 test fixtures rather
than production defects. Checkpoints `6f0439845` and `87ea7a427` update them to:

- declare typed `ecvector(4)` columns for distributed-control fixtures;
- test an unknown abort before the pg_test transaction retains another build's
  session lock;
- revoke the temporary role's schema grant before dropping it;
- assert scan-registry behavior according to the actual preload state; and
- expect the current pre-publish CustomScan error,
  `EC_GENERATION_MISSING: logical index has no active epoch`.

## Validation

- Focused recovery test at `772728b23`: 1 passed, 0 failed, 2507 filtered.
- Final aggregate Task 179 PG18 run at `87ea7a427`:
  238 passed, 0 failed, 3 explicitly ignored, 2267 filtered; the same command
  also ran 21/21 DistANN on-disk fixtures successfully.
- Crate-wide PG18 clippy at `87ea7a427`: pass with `-D warnings`.

The aggregate pgrx command is intentionally the complete `distann`-named
surface and is serial because this repository's PG fixtures and global GUC
tests are not parallel-safe. A broader full-crate diagnostic found three old,
unrelated TurboQuant test failures before the PG lane; they are recorded in
`artifacts/full-crate-diagnostic.md` and are not reclassified as Task 179
failures.

## Housekeeping

- `plan/tasks/README.md` now records Task 179 as done and preserves Task 172's
  broader open scope.
- The task header now cites the outside decisions and packet 060.
- The stale nine-packet forecast is reconciled to the actual packet 001–060
  history.

## Requested verification

Please verify that this packet faithfully closes the three conditions in
packet 059. The packet remains open for feedback; this request does not
self-approve a merge.
