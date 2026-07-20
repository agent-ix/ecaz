# Review request: require a committed cancelled decision before recovery

## Scope

Please review implementation commit `13905ef14` as the narrow remediation for
packet 034 P2-1.

`ec_distann_recover_cancelled_publish` now reads the locked decision row's
`xmin`, validates its encoding, and rejects it with
`EC_TRANSACTION_BOUNDARY` when `TransactionIdIsCurrentTransactionId` reports
that cancellation was written by the current transaction. This mirrors normal
publish and retire recovery: cancellation must commit before cleanup recovery
begins.

FR-082 now states that boundary explicitly.

## Live transaction proof

The existing real-client multi-epoch PG18 test now adds this sequence before
its successful cancellation path:

1. `BEGIN`;
2. cancel the Pending publish decision;
3. attempt cancelled-generation recovery in the same transaction;
4. require `EC_TRANSACTION_BOUNDARY`; and
5. `ROLLBACK`.

The remainder of the test then commits cancellation in autocommit mode and
proves ordinary audited reclaim, exact replay, and the next epoch build still
succeed. This covers both the negative boundary and rollback/no-poison behavior.

## Validation

See `artifacts/manifest.md` and its exact-SHA PG18 logs.

## Requested decision

Please close packet 034 P2-1 if the xmin guard and real transaction-boundary
regression match the established recovery contract.
