# Task 234 remote transport call-site inventory

Inventory baseline: `73842c413b8022dfaa9cd79ad9c3f3603ede091a`.

Scope: `src/am/ec_distann/remote_transport.rs`. The acceptance scan is over
async `tokio-postgres` connect, setup, prepare, query, query-one, execute, and
batch-execute awaits. Synchronous `postgres::Client` callbacks are listed
separately; they do not satisfy an async-wrapper check and are not silently
treated as covered.

## Existing bounded async surfaces

| Surface | Await path | Disposition |
| --- | --- | --- |
| connection establishment | `open_remote_connection` -> `await_remote(config.connect)` | bounded connect timeout; retained |
| remote `statement_timeout` setup/refresh | `configure_remote_statement_timeout` -> `await_remote(query_one)` | bounded, but Task 234 must define pool eviction on ambiguous completion |
| scan session identity setup | `configure_scan_identity` -> `await_remote(query)` | bounded, same eviction requirement |
| prepared physical statements | `prepare_physical_statement` -> `await_remote(prepare)` | bounded, same eviction requirement |
| lifecycle query/query-one | `lifecycle_query`, `lifecycle_query_one` | bounded; lifecycle/write semantics otherwise remain Task 235 |
| physical seed, expansion, and materialization | `physical_query` | bounded read path |
| logical expansion and row materialization | `scan_query` | bounded read path |
| traversal-replica chunk reads | `physical_query` | bounded read path |

`await_remote` currently sends a remote cancel only for a locally observed
PostgreSQL interrupt. A client deadline drops the query future without a cancel
delivery or mandatory connection eviction, and the remote-error variants
collapse timeout/cancel/transport classes into strings or `EC_INTERNAL`.
Those are wrapper-contract gaps even for the already-routed read callers.

## Task 234 bare async read/control gaps

| RPC | Function | Bare operation | Required result |
| --- | --- | --- | --- |
| sharded physical head search | `run_one_physical_head_search` | `Client::query(...).await` | bounded typed read await |
| crown-code export | `run_one_crown_code` | `Client::query(...).await` | bounded typed read await |
| gateway-routing export | `run_one_gateway_routing` | `Client::query(...).await` | bounded typed read await |
| head-shard export | `remote_head_shard_export` | `Client::query(...).await` | bounded typed read await |
| head-shard import | `remote_head_shard_import` | `Client::query_one(...).await` | bounded typed control await |

The crown-code row is the correction to the original four-RPC plan. It is a
distributed read, uses the same pooled connection state, and has no defensible
allowlist reason to remain bare.

All multi-owner read batches must be normalized fail-closed: after any owner
fails, callers receive no successful sibling result to consume. Error choice is
deterministic in request order. A local interrupt prevents later undispatched
owner futures from being polled.

## Task 235 async write/transaction allowlist

The following bare awaits are real gaps, but changing their ambiguous-commit
semantics in Task 234 would cross the explicit task boundary:

- `record_remote_physical_intent` and `mark_remote_physical_intent` executes;
- physical insert `BEGIN`, owner debug setup, insert call, `PREPARE
  TRANSACTION`, rollback, and intent acknowledgement;
- tombstone `BEGIN`, mutation, `COMMIT`, and rollback;
- backlink `BEGIN`, owner debug setup, mutation, `PREPARE TRANSACTION`,
  rollback, and intent acknowledgement.

Task 234 may reuse non-transactional connection/setup helpers without claiming
these write calls are safe. They remain structural-scan allowlist entries owned
by Task 235.

## Synchronous libpq / local SPI allowlist

The xact callbacks, pre-commit intent updates, prepared-transaction reaper,
node-registry inspection, and local SPI operations use synchronous
`postgres::Client`, the shared SPI surface, or PostgreSQL-local calls. They are
not async bare-await findings. Task 235 owns remote write/recovery semantics;
Task 236 owns TLS/secret/conninfo productionization for both async and sync
transports.

## Structural closeout scan

At Task 234 closeout, the five named functions above must contain no direct
`.query(...).await` or `.query_one(...).await`. New async read/control awaits
must either route through the typed bounded read wrapper or be added to this
inventory with an explicit task owner and reason. The Task 235 transaction
allowlist must not expand.
