# Review request: close the DistANN endpoint privilege set

## Scope

Please review implementation commit `7c3bc1485` as the class-level remediation
for packet 033's recurring endpoint privilege finding and packet 039's partial
fix.

The prior checkpoint secured a hand-maintained list of remote signatures. It
left sibling read, write, legacy-lifecycle, and debug functions outside that
list, while its ACL test only inspected metadata under a hardcoded name filter.

This checkpoint replaces that posture with a set policy:

- a finalize-time dependency scan applies `SECURITY DEFINER`, the fixed safe
  search path, and `REVOKE ... FROM PUBLIC` to every extension-owned
  `ec_distann_*` SQL function except the access-method handler and the two
  explicitly benign public surfaces (`ec_distann_owning_node` and
  `ec_distann_epoch_status`);
- the set is derived from `pg_depend` extension membership, so a new sibling is
  protected without editing another signature list;
- `ec_distann_debug_tombstone`, `ec_distann_debug_set_in_flight`, and
  `ec_distann_debug_expand_search` are compiled only with `pg_test` and are
  absent from production extension SQL;
- `ec_distann_fold_delta_into_graph` rejects non-READ-COMMITTED isolation before
  relation access or mutation; and
- FR-079/FR-083 now state the class-wide policy and fold isolation contract.

## Real caller proof

The PG18 ACL regression no longer relies on `has_function_privilege` or a
literal endpoint-name list. It:

1. enumerates every protected extension-owned function from `pg_proc` and
   `pg_depend`;
2. creates a fresh NOLOGIN role and grants it extension-schema usage so schema
   denial cannot mask a function ACL failure;
3. derives a non-NULL, type-correct inert call for every installed signature;
4. executes every call after `SET ROLE`; and
5. requires SQLSTATE `42501` and `permission denied for function` for every
   overload, while also checking definer and search-path metadata.

The test requires at least 40 protected overloads and fails if any argument
type lacks a non-NULL fixture. The separate fold test uses a real loopback
Repeatable Read transaction and invalid OID 0, proving isolation rejection
precedes relation access.

## Validation

See `artifacts/manifest.md` and its packet-local logs for exact-SHA commands and
results. The production PG18 package check also requires zero occurrences of
all three debug endpoint names in the packaged extension SQL.

## Requested decision

Please confirm that the dependency-derived set policy plus real unprivileged
invocations closes packet 033's recurring privilege class, packet 039's
remaining P1/P2 endpoint findings, and the fold-maintenance isolation gap.
