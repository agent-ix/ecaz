# SPIRE Typed Tuple Transport

Status: Phase 12.2 design checkpoint, pending reviewer acceptance.

## Goal

Replace the ADR-068 JSON tuple-payload bridge used by
`EcSpireDistributedScan` with a typed transport that can construct remote-origin
virtual tuples without routing scalar values through `serde_json` text input.

This design covers distributed read tuple delivery. The coordinator-routed DML
JSON payloads remain Phase 12.5 schema/type-round-trip hardening unless a later
packet explicitly folds them into the same typed protocol.

## Current Shape

The current read path is:

1. The coordinator asks each remote for requested tuple columns.
2. `ec_spire_remote_search_tuple_payload(...)` returns a JSON object per
   visible remote heap candidate.
3. The CustomScan executor parses that object with `serde_json`.
4. Each JSON value is converted to text and fed through the target column's
   PostgreSQL input function.

That bridge is intentionally narrow. It has already proved the CustomScan
integration point, but it is not the production transport for Phase 12 because:

- arrays and composites are blocked or stringified instead of preserving typed
  PostgreSQL binary values;
- JSON numeric/string rendering can obscure exact type semantics;
- every tuple pays JSON parse/allocation cost in the coordinator executor;
- `serde_json` remains a runtime dependency solely for tuple slot delivery.

## Protocol Choice

Use **per-attribute PostgreSQL binary I/O** rather than one binary
composite/record payload.

The typed endpoint returns the requested projection as column-aligned arrays:

- `payload_attnums int2[]`
- `payload_names text[]`
- `payload_type_oids oid[]`
- `payload_typmods int4[]`
- `payload_collations oid[]`
- `payload_nulls bool[]`
- `payload_values bytea[]`
- `payload_formats text[]`

For each non-null requested attribute, the remote calls the attribute type's
binary send function (`pg_type.typsend`) and returns the bytes. The coordinator
validates that the returned column name, type OID, typmod, and collation match
the coordinator tuple descriptor before calling the corresponding receive path
for the destination slot. For SQL NULL attributes, `payload_nulls[i]` is true
and `payload_values[i]` is a zero-length `bytea` placeholder that the
coordinator must ignore instead of passing to the type receive function.

Reasons to prefer per-attribute binary values:

- the coordinator already owns the requested projection and target tuple
  descriptor, so validating one attribute at a time is direct;
- NULLs stay out-of-band instead of relying on JSON `null` or composite field
  conventions;
- projection subsets do not require manufacturing a remote anonymous record
  type that exactly matches the coordinator's virtual slot;
- a single unsupported column type can produce a precise fallback/error reason;
- the protocol naturally records type/typmod/collation drift per column, which
  Phase 12.5 needs for schema-drift hardening.

Binary composite/record transport remains a later optimization candidate only
if per-attribute framing proves too expensive after P1/P3 measurement.

Empty projections are a supported typed-transport shape. When
`requested_columns` is empty, the endpoint returns aligned empty metadata,
NULL, value, and format arrays with `tuple_transport = 'pg_binary_attr_v1'`.
The coordinator may use this shape to validate row existence without paying
projection conversion cost.

Named composite table columns use the column type's normal `record_send` bytes
and preserve the named composite type OID in `payload_type_oids`. Anonymous
computed composite projections are outside the v1 table-column projection
contract; the v1 endpoint accepts heap column names, not arbitrary SELECT-list
expressions. If a future projection surface admits anonymous `record` values,
that packet must either reconstruct field metadata on the coordinator or reject
anonymous composites with a stable diagnostic.

## Endpoint Shape

Add a sibling read endpoint beside the JSON endpoint:

```sql
ec_spire_remote_search_tuple_payload_typed(
  index_oid oid,
  requested_epoch bigint,
  query real[],
  selected_pids bigint[],
  top_k integer,
  consistency_mode text,
  requested_columns text[]
)
```

The endpoint keeps the same candidate identity columns as
`ec_spire_remote_search_tuple_payload(...)`, including `node_id`, `vec_id`,
`row_locator`, score, status, and `tuple_payload_missing`. It replaces the
`tuple_payload jsonb` field with the typed arrays above and adds:

- `tuple_transport text`, initially `pg_binary_attr_v1`;
- `tuple_transport_status text`, with stable labels such as `ready`,
  `unsupported_type_binary_io`, `schema_drift`, and
  `remote_tuple_payload_missing`.

The JSON endpoint remains available during the migration window and for
diagnostics, but the production CustomScan path prefers typed payloads once all
target remotes and requested columns support them.

## Coordinator Receive

`EcSpireDistributedScan` will add a typed payload receive path that:

1. requests the typed endpoint when the descriptor advertises
   `pg_binary_attr_v1`;
2. validates returned projection metadata against the coordinator tuple
   descriptor;
3. stores SQL NULLs directly from `payload_nulls`, ignoring the corresponding
   zero-length `payload_values` placeholder;
4. calls the column type's receive function for non-null binary values;
5. marks the slot virtual only after every requested attribute is validated and
   converted;
6. reports fail-closed status on metadata mismatch instead of silently falling
   back after the remote has produced typed rows.

The existing JSON slot path remains the explicit fallback path while mixed
version deployments are supported. It should not be used after a remote has
advertised typed support for a requested projection and then returns malformed
typed metadata.

## Negotiation

Add typed tuple transport capability to the remote descriptor/identity surface:

- `tuple_transport_capabilities text[]`, containing `pg_binary_attr_v1`;
- `tuple_transport_default text`, initially either `json_v1` or
  `pg_binary_attr_v1`;
- `tuple_transport_status text`, so strict/degraded diagnostics can name why a
  remote was skipped or why JSON fallback was selected.

Coordinator selection rules:

- Strict mode requires every selected remote to support the chosen production
  transport for every requested column.
- Degraded mode may skip a remote whose selected projection lacks typed
  support, reporting the node id and first unsupported column/type.
- JSON fallback is allowed only when at least one selected remote is an older
  extension version that does not advertise typed support.
- JSON fallback is not allowed for a new remote that advertises
  `pg_binary_attr_v1` but fails typed metadata validation.

The advertised capability is protocol-level, not a guarantee that every future
projection on that remote has binary I/O support. A typed-capable remote may
still reject a specific requested column with `unsupported_type_binary_io` if
that column's type lacks a usable binary send function. For v1,
`tuple_transport_default` is the build's hardcoded preferred default rather
than a per-descriptor operator setting; a future debugging or version-skew
packet can add a descriptor writer surface if forcing JSON per remote becomes
necessary.

## JSON Fallback Window

The JSON read path may remain production-reachable for one minor-version window
after `pg_binary_attr_v1` ships.

Removal criteria:

- typed endpoint fixtures cover scalar, array, composite, NULL, and domain
  values where PostgreSQL binary I/O supports the type;
- unsupported type classes have stable diagnostic labels and reviewer-accepted
  deferrals;
- mixed-version negotiation fixtures prove typed-preferred and JSON-fallback
  behavior;
- tuple-heavy read throughput is measured before and after typed transport with
  packet-local logs;
- no non-diagnostic runtime path still requires `serde_json` for read tuple
  slot delivery.

After those criteria pass, remove JSON from the production read path and keep
the JSON endpoint only if a diagnostic or compatibility packet explicitly
accepts it.

## First Implementation Slices

1. Add the typed endpoint with metadata arrays and a JSON-parity scalar fixture.
2. Add binary I/O coverage for NULL, arrays, composites, and domains.
3. Add descriptor negotiation and strict/degraded fallback diagnostics.
4. Switch CustomScan receive to typed-first with JSON fallback for older
   remotes.
5. Measure tuple-heavy read throughput and retire production JSON fallback
   after reviewer acceptance.
