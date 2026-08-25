# Task 235 remote phase inventory

Source checkpoint: `7584c1bf3fc14569b9bfc7928d6a18e2a15728d5`.

## Outcome vocabulary

- `definitely_not_applied`: PostgreSQL returned an explicit error for the
  atomic statement, or the failed phase cannot apply a logical mutation.
- `outcome_unknown`: the request may have crossed a durable boundary before a
  timeout, local interrupt, connection loss, or lost response. The transport
  does not guess. It evicts the session and requires idempotent replay or
  operator recovery.

Every async write failure removes the keyed pooled connection. The outer
transport state also clears all connections before raising a PostgreSQL
interrupt observed while the runtime was parked.

## DML and 2PC phases

| Phase | Call surface | Bound/cancel mechanism | Ambiguous result | Durable/retry contract |
|---|---|---|---|---|
| Connection | `ensure_pooled_connections`, `open_remote_connection` | nonzero connect timeout + PostgreSQL interrupt poll | connection establishment has no logical mutation | failed connection is not pooled |
| Remote statement budget | `configure_remote_statement_timeout` | client deadline + bounded CancelRequest | no DML dispatched yet | connection discarded on setup failure |
| Preplanning intent | `record_physical_insert_intent_row` | `bounded_write_phase`; deadline, interrupt, CancelRequest | upsert acknowledgement may be lost | idempotent GID upsert; session evicted |
| Owner intent | `record_remote_physical_intent` | `bounded_write_phase` | same | idempotent GID upsert; session evicted |
| `BEGIN` | insert/backlink/tombstone | `bounded_write_phase` | logical mutation definitely absent, session state may be open | session evicted on any failure |
| Session `SET` | insert/backlink A/B setting | `bounded_write_phase` | logical mutation absent | session evicted; transaction closes with connection |
| Endpoint mutation | insert/backlink/tombstone endpoint | `bounded_write_phase` | mutation may have executed before response loss | bounded rollback attempt; then unconditional eviction; insert/backlink intent and tombstone source-map retry token remain |
| Rollback cleanup | failed endpoint mutation | `bounded_rollback_after_failure` | cleanup acknowledgement may be lost | primary error retained, cleanup failure appended, session evicted |
| `PREPARE TRANSACTION` | insert/backlink | `bounded_write_phase` | owner may already be durably prepared | no guessed rollback; GID/intent remain for reaper |
| Prepare acknowledgement | intent state `prepare_acked` | `bounded_write_phase` | prepared xact exists; row may be requested or acked | either state is non-decisional; actual coordinator xid decides recovery |
| Precommit decision fence | intent state `commit_intended` | async bounded write in PostgreSQL `PreCommit` callback | row update may commit before response loss | callback aborts local commit on error; reaper ignores the ambiguous row as decision authority and reads coordinator xid status |
| Coordinator commit/abort callback | `resolve_physical_insert_prepared` | fresh blocking TLS client with connect timeout, server statement timeout, and TCP user timeout | `COMMIT/ROLLBACK PREPARED` acknowledgement may be lost | intent remains operator-visible; repeated reaper action uses coordinator xid and prepared-xact presence |
| Terminal intent update | `commit_local` / `rollback_local` | same bounded blocking callback client | terminal audit update may be lost | terminal state is audit only, not recovery decision authority |
| Tombstone `COMMIT` | routed VACUUM tombstone | `bounded_write_phase` | flag may be committed before response loss | source-map row is retained by caller on error; endpoint retry is idempotent; session evicted |
| Reaper scan/action | `ec_distann_reap_orphaned_remote_prepared_xacts` | fresh blocking TLS client with connect, statement, and TCP user timeouts | action acknowledgement may be lost | union of prepared GIDs and nonterminal intent GIDs is reconciled; actual coordinator `pg_xact_status` is authoritative; repeated invocation converges while status is retained |

## Recovery decision table

| Coordinator `pg_xact_status` | Reaper action |
|---|---|
| `in progress` | leave prepared xact fenced |
| `committed` | `COMMIT PREPARED` |
| `aborted` | `ROLLBACK PREPARED` |
| `NULL` / status unavailable | no action; report `operator_required` |

`commit_intended`, `prepare_acked`, and `prepare_requested` remain useful audit
states but cannot override the coordinator's actual transaction outcome.
Scanning nonterminal intents as well as prepared xacts means recovery can
finalize audit state after an earlier `COMMIT PREPARED`/`ROLLBACK PREPARED`
succeeded but its response or terminal update was lost. Conversely, a prepared
GID with a missing intent is explicit in the result and still resolves only
from the coordinator status.

## Build and generation lifecycle phases

All remote lifecycle calls use `lifecycle_query`/`lifecycle_query_one`, which
now route through `bounded_write_phase` and `finalize_write_call`. A timeout,
interrupt, or reset is `outcome_unknown`, evicts the lifecycle connection, and
requires replay through the existing digest/sequence/decision identity.

| Phase | Function | Replay identity |
|---|---|---|
| Begin handoff | `remote_begin_epoch_handoff` | index, epoch, build UUID, build/roster/descriptor digests |
| Stage batch | `remote_stage_epoch_batch` | build UUID, sequence, batch digest |
| Seal handoff | `remote_seal_epoch_handoff` | build UUID, expected count and owner digest |
| Abort handoff | `remote_abort_epoch_handoff` | index + build UUID |
| Publish epoch | `remote_publish_epoch` | build UUID + manifest digest |
| Mark predecessor retired | `remote_mark_epoch_retired` | successor activation + digest |
| Apply retire decision | `remote_apply_epoch_retire` | retire decision + digest |
| Reclaim cancelled generation | `remote_reclaim_cancelled_generation` | cancellation audit + digest |

## Evidence still required

Packet 003 must inject timeout/cancel/reset/backend death before mutation,
during mutation, before and during prepare, after precommit intent, and during
commit/rollback prepared. It must assert source, owner graph/row/directory,
intent, prepared-xact, tombstone, and retry state, including one-owner partial
completion and restart.

Packet 004 must prove duplicate operator recovery, status-unavailable STOP,
prepared-slot saturation/readiness, and bounded callback/reaper behavior. This
inventory does not claim those runtime gates yet.
