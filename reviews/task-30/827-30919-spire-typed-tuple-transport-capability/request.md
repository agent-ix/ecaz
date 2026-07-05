---
topic: spire-typed-tuple-transport-capability
agent: coder1
role: coder
model: gpt-5
date: 2026-05-12
seq: 30919
stage: phase-12.2
status: open
---

# Review Request: Typed Tuple Transport Capability Advertisement

## Scope

Please review commit `119fd741` (`Advertise typed tuple transport
capability`).

This is the first negotiation slice for Phase 12.2. It advertises typed tuple
transport support on the live endpoint identity surface but does not yet switch
`EcSpireDistributedScan` receive from JSON to typed binary datum construction.

## What Changed

- `ec_spire_remote_search_endpoint_identity(...)` now returns:
  - `tuple_transport_capabilities = ARRAY['pg_binary_attr_v1']`;
  - `tuple_transport_default = 'pg_binary_attr_v1'`;
  - `tuple_transport_status = 'ready'`.
- `ec_spire_remote_search_endpoint_contract()` now documents the same three
  tuple transport contract rows.
- The endpoint identity PG18 fixture asserts the capability/default/status
  fields.
- The receive-contract PG18 fixture asserts the new contract rows and updated
  endpoint contract count.
- The Phase 12 tracker records this as capability advertisement only. The
  parent negotiation item remains open until CustomScan chooses typed transport
  and JSON fallback rules are covered.

## Evidence

See `artifacts/manifest.md`.

Validation run against `119fd741a20642b289f01a24c2eb84b271b56ed1`:

- `git diff --check HEAD^ HEAD`
- `cargo fmt --check`
- `cargo pgrx test pg18 test_ec_spire_remote_search_endpoint`
- `cargo pgrx test pg18 test_ec_spire_remote_search_receive_contract`

## Review Focus

- Confirm endpoint identity is the right first advertisement surface for
  `pg_binary_attr_v1`.
- Confirm the profile fingerprint should remain unchanged by tuple transport
  capability metadata in this slice.
- Confirm the tracker wording does not overclaim full negotiation or
  CustomScan typed receive.
