# Review request — Task 165: owner-side row-column payload shipping (CustomScan data path)

**Branch:** `task-165-ec-distann-m3`. HEAD `a4ecd5e70`. The row-data half of real
multi-node materialization, building directly on 009 (reviewer 005-P1 / 006-P1).

## Context (honest architecture)

Packet 009 landed `ec_distann_materialize_rows`, which ships a remote-owned hit's
**ctid + tombstone**. But a remote ctid is *not fetchable from the coordinator's
local heap* — so ctid shipping only completes on the co-located/loopback
substrate. A genuinely distributed scan must get the **row column data itself**
from the owning node and reconstruct the tuple on the coordinator. This packet
lands that owner-side data path.

## What landed

`ec_distann_materialize_row_payloads(index_regclass, epoch_fingerprint, vec_ids,
payload_columns text[], payload_send_functions text[])` — for each owned vec_id,
the owning node resolves its heap ctid and ships the requested projection columns
encoded as PostgreSQL binary (`typsend`), returning
`(vec_id, is_tombstone, tuple_payload_missing, payload_nulls bool[],
payload_values bytea[])`.

- **Shared preflight.** The FR-082 epoch fingerprint validation (retriable
  mismatch), FR-078 per-vec_id ownership check (placement error), and
  owned-but-absent → `[EC_RECORD_MISSING]` are now factored into
  `resolve_owned_rows`, reused by both the ctid endpoint (009) and this one, so
  the two materialize paths cannot drift on the fault taxonomy.
- **Owner-side column encoding is pure SQL** — a `WITH ORDINALITY` unnest of the
  candidate ctids LATERAL-joined to the heap relation by ctid, projecting each
  column through its `typsend` function to `bytea`. This mirrors the ec_spire
  tuple-payload pattern (`src/lib.rs` `ec_spire_remote_search_tuple_payload_typed`)
  rather than reaching for unsafe `SendFunctionCall` FFI.
- **Injection-safe.** Send-function names are validated to plain (optionally
  schema-qualified) identifiers before any SQL is built; column names are
  double-quote-escaped; the heap relation is resolved to a schema-qualified,
  quoted name from its oid (a `regclass` value cannot be a FROM-clause target).
- **Order preserving.** Responses zip 1:1 back onto the request order via
  `ORDER BY candidate.ordinality`, and the impl asserts the row count matches.

## Evidence (`artifacts/test-evidence.log`)

`test_ec_distann_materialize_row_payloads_ships_binary_columns`:
- 2 owned rows ship exactly one **8-byte `int8send`** non-null column each, no
  tombstone / no `tuple_payload_missing`;
- the shipped binary **decodes byte-exact** back to the row's `id` (big-endian
  low byte in the fixture's 1..=8 id range);
- wrong epoch fingerprint → retriable epoch-mismatch (fail closed);
- injection-shaped send-function name → rejected before any SQL runs;
- column/send-function arity mismatch → rejected.

**pg18 distann pg_tests: 103 passed, 0 failed (was 102); clippy clean.**

## Honest remaining scope (still open)

This is the owner-side (data-producing) half. The consuming half — the
**CustomScan** that calls this endpoint per remote owner, hex/binary-decodes
`payload_values` via `ReceiveFunctionCall`, and yields reconstructed virtual
tuples instead of routing through `amgettuple`'s local-heap fetch — is the next
slice (B.2/B.3). It will be validated first on the loopback substrate (forcing
the multi-node path through the shipped payload rather than local-directory
resolution), then across the real 3-instance fixture (Slice A). The 3-worker
`ecaz bench suite` recall gate (D) depends on both.

## Ask

Review the owner-side endpoint: the shared `resolve_owned_rows` preflight, the
SQL `typsend` column encoding, the injection guards, and the request/response
ordering contract. **Not closing the request** — the CustomScan consumer and the
3-worker gate remain.
