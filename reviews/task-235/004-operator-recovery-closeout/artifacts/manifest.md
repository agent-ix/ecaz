# Task 235 operator-recovery closeout artifact manifest

Date: 2026-08-25 (America/Los_Angeles)

## Scope and provenance

- Task bucket: `reviews/task-235/`
- Packet: `004-operator-recovery-closeout/`
- Validated candidate: `b871d5481376df87c60ae486d68bb78519944c21`
- Owning runtime source:
  `reviews/task-235/003-2pc-lifecycle-fault-matrix/artifacts/task235-write-lifecycle-fault-matrix.log`
- Runtime command, fixture, TLS posture, head preflight, and complete artifact
  hashes are recorded in packet 003's `artifacts/manifest.md`. This packet is a
  task-local closeout view over that same clean release run, not a second run.

## Operator results

- Status unavailable: two identical reaper calls returned
  `xid_status_unknown:operator_required`, retained `prepare_requested`, created
  no prepared transaction, and performed no guessed commit or rollback.
  Restoring authoritative coordinator status converged the intent to
  `commit_local`; the duplicate call emitted zero actions.
- Lost decision acknowledgement: both commit-prepared and rollback-prepared
  acknowledgement-loss cells reconciled eleven nonterminal intent rows in one
  operator recovery attempt and emitted zero duplicate actions.
- Partial/missing evidence: the one-owner partial-commit cell reconciled ten
  remaining prepared transactions, while the missing-intent cell explicitly
  recovered the prepared GID from the coordinator's committed full XID.
- Readiness: filling all 32 owner prepared slots produced the stable
  `prepared_slots_exhausted_hint_increase_max_prepared_transactions` category,
  exposed one nonterminal recovery fence, mutated neither source nor owner,
  and converged on one operator reaper attempt.
- Routed deletion: killing the owner backend during physical tombstone
  application made the first VACUUM fail; explicit retry converged with zero
  source/source-map rows, one owner tombstone, and zero prepared/nonterminal
  residue.

These results cover Task 235's NFR-014 operational slice: recovery remains
operator-driven, status uncertainty stops without guessing, readiness reports
the prepared-slot requirement, and recovery records identify coordinator and
target node without embedding conninfo or secrets. Task 236's accepted secure
transport packet remains the governing evidence for the broader NFR-014 TLS,
secret-resolution, privilege, and redaction controls.

## Artifact

- `operator-recovery-summary.log` — exact selected records copied from the
  owning compact matrix at the validated head; SHA-256
  `f2fa67c665f152dfba020057664e81d10f95f88c45c95082c552356ce33d9ad9`.
