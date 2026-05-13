# SPIRE Libpq Security and Operations Runbook

This runbook covers the Phase 12 local production-readiness boundary for SPIRE
remote libpq execution. It is not a full TLS, credential-management, or audit
subsystem design; those remain explicit future work unless named below.

## Connection Security Contract

Remote connection strings are resolved outside the extension through
`conninfo_secret_name`. SPIRE catalogs and SQL diagnostics store and return the
secret name, provider status, sanitized blocker labels, node IDs, and counts.
They must not expose raw libpq conninfo or raw remote error text.

The secret provider is responsible for returning a complete libpq conninfo
string. SPIRE must preserve libpq security parameters from that resolved value,
including `sslmode`, `sslrootcert`, `sslcert`, `sslkey`, `target_session_attrs`,
and any supported future libpq security option. It must not strip, downgrade,
or rewrite `sslmode`. If the provider returns `sslmode=verify-full`, the remote
open must use that mode and fail rather than silently falling back to a weaker
mode.

Operator checklist:

- Use `conninfo_secret_name` for every remote descriptor; do not put raw
  conninfo in SPIRE catalogs.
- Store the real conninfo only in the external secret provider.
- Include the intended `sslmode` and certificate parameters in that provider
  value.
- Treat a descriptor whose conninfo secret cannot be resolved as not ready for
  distributed reads or coordinator-routed writes.
- Confirm diagnostics expose only `conninfo_secret_name`, resolution status,
  and sanitized failure categories.

## Sanitized Failure Handling

Authentication, authorization, certificate, hostname, missing secret, and
conninfo-parse failures are operational failures of the remote node, not data
values. Operator-visible surfaces should classify them with stable sanitized
status labels such as `conninfo_secret_resolution`, `conninfo_parse_failed`,
`connection_failed`, `remote_authentication_failed`, or
`remote_certificate_verification_failed`.

Strict and degraded modes differ only in coordinator behavior:

- strict mode fails the distributed operation when a required remote cannot be
  authenticated, opened, or verified;
- degraded mode may skip the affected remote only when the query path permits
  degraded execution, and it must report the skipped node and sanitized reason.

Neither mode should expose passwords, hosts embedded in secret payloads,
certificate contents, raw conninfo, or raw remote error text through SQL
diagnostics.

If a degraded skip report shows
`first_skip_category = 'tuple_transport_retired'`, the remote endpoint identity
was valid but production tuple delivery could not use the required
`pg_binary_attr_v1` transport. Upgrade the remote `ecaz` extension, refresh the
remote descriptor, and verify the endpoint advertises
`tuple_transport_capabilities` containing `pg_binary_attr_v1`. Do not force the
legacy `json_tuple_payload_v1` transport for production payload dispatch; it is
a compatibility label only.

## Prepared Transaction Readiness

Coordinator-routed writes use remote prepared transactions. Every remote that
can receive SPIRE INSERT or DELETE work must set `max_prepared_transactions`
above zero and reserve enough free slots for peak SPIRE concurrency plus other
application prepared transactions. Changing the value requires a PostgreSQL
restart.

Descriptor registration performs a best-effort readiness preflight when the
secret can be resolved. A warning about unreachable remotes, unreadable
`SHOW max_prepared_transactions`, or a zero value is a write-readiness blocker
even when descriptor registration itself succeeds.

If `PREPARE TRANSACTION` fails because prepared transactions are disabled or
exhausted, fix the remote setting or reduce coordinator write concurrency
before retrying the write workload.

## Orphaned Prepared Transaction Recovery

If a coordinator backend crashes after remote prepare and before the local xact
callback resolves the remote transaction, inspect the remote:

```sql
SELECT gid, prepared, owner, database
  FROM pg_prepared_xacts
 WHERE gid LIKE 'ec_spire_insert_%'
 ORDER BY prepared;
```

SPIRE GIDs use the stable form:

```text
ec_spire_insert_<index_oid>_<node_id>_<served_epoch>_<top_xid>
```

The `ec_spire_insert` prefix is historical and can cover INSERT or DELETE
prepares. Do not infer the operation type from the prefix. Resolve the prepared
transaction only after the affected primary key and coordinator transaction
outcome are known:

- commit an INSERT prepare only when the coordinator transaction committed and
  the expected placement row exists;
- roll back an INSERT prepare when the coordinator transaction aborted or the
  placement row is absent after the outcome is known;
- commit a DELETE prepare only when the coordinator transaction committed and
  the placement row was removed;
- roll back a DELETE prepare when the coordinator transaction aborted and the
  placement row remains.

After `COMMIT PREPARED` or `ROLLBACK PREPARED`, verify the remote row and the
coordinator placement row match the intended outcome. If the coordinator
outcome or affected key cannot be established, leave the prepared transaction
unresolved and escalate with the GID, node ID, and coordinator index OID.

There is intentionally no v1
`ec_spire_recover_orphaned_prepared_xacts(node_id)` helper. Remote
`pg_prepared_xacts` alone does not contain enough information to choose commit
versus rollback safely.

## Credential Rotation

Credential rotation is a Phase 12 deferral. The v1 safe procedure is a manual
maintenance window:

1. Pause coordinator-routed writes and production distributed reads for the
   affected remote descriptors.
2. Update the external secret provider entry referenced by
   `conninfo_secret_name`.
3. Run a descriptor readiness check or remote pipeline dry/live diagnostic
   appropriate for the environment.
4. Resume traffic only after the descriptor reports a resolvable secret and the
   remote can be opened with the intended libpq security parameters.

SPIRE does not yet provide an automatic credential-rotation watcher, pooled
connection invalidation protocol, or dual-secret switchover workflow.

## Audit Logging

The audit-log schema is also deferred. Until it lands, rely on:

- PostgreSQL server logs on the coordinator and remotes;
- review-packet artifacts for validation runs;
- descriptor diagnostics for node IDs, descriptor generation, served epoch,
  status labels, and sanitized blocker categories;
- remote `pg_prepared_xacts` for unresolved prepared transactions.

Do not log raw conninfo or secret payloads in packet artifacts. If an incident
requires proving which secret was used, record the `conninfo_secret_name`,
provider version outside the repository, node ID, descriptor generation, and
timestamp.
