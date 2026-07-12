# Review request: cancelled-generation recovery

Please review code commit `1b531ce9fcb97268cb027de29dd52dd2441b72a7` as
the follow-up to packet 033's explicitly open cancelled-orphan cleanup path.

## Contract and wire format

- `DistannCancelPublishAuditV1` canonically binds coordinator UUID, build id,
  epoch, fingerprint/manifest, caller, reason, and timestamp under
  `ec_distann_cancel_epoch_publish_v1\0`.
- The new format has a production round trip, independent TC-050 decoder,
  endian/version rejection, golden fixture, pinned offsets, and writable
  upgrade-matrix row.
- FR-082 and Task 179 now specify explicit replayable cancellation cleanup.

## Participant safety

- `ec_distann_reclaim_cancelled_generation` accepts only the exact canonical
  audit/digest and the matching coordinator/build/epoch identity.
- Only non-active `Ready` or `Published` generations are eligible. A Published
  generation must match the audit fingerprint and manifest exactly.
- One transaction inserts `ec_distann_cancelled_generation_reclaim` before
  dropping hidden relations and deleting the live generation row. Exact replay
  succeeds from the tombstone; conflicting replay fails closed.
- The endpoint is SECURITY DEFINER with a pinned search path and no PUBLIC
  execute privilege.

## Coordinator recovery

- `ec_distann_recover_cancelled_publish` locks and re-verifies the durable
  Cancelled decision/audit, refuses an active target, and replays the immutable
  private participant bindings across local and remote owners.
- Partial remote success is safe because each participant tombstone is
  idempotent. The coordinator writes `cancellation_reclaimed_at` only after all
  participant calls acknowledge.
- DROP/REINDEX catalog cleanup includes the new tombstone table, and participant
  status reports `CancelledReclaimed` with the audit digest.

## Validation

See `artifacts/manifest.md` and its raw packet-local logs. All focused checks
pass, including a committed `Published`-before-cancel participant window.

## Review questions

1. Does the canonical cancellation audit provide sufficient authorization and
   identity binding for participant deletion?
2. Is the participant transaction ordered safely (tombstone before physical
   deletion, all rolled back together)?
3. Is coordinator replay correct after any subset of remote participants has
   already committed cleanup?
4. Together with packet 033, does this close the packet-006 Pending-decision
   availability wedge and its Published-but-never-active orphan follow-up?

## Still open for Task 179

- Real three-instance fault injection should exercise a remote participant
  committed Published/cleanup acknowledgement before another owner fails.
- Remote RPC timeout/interrupt behavior, physical hot-path caching/cost
  decomposition, persisted-head sensitivity/cap evidence, and final accepted
  performance evidence remain open.
