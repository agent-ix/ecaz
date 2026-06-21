# Task 111h / 008 - Packed Rerank Group Lookup Test Checkpoint

## Scope

Code commit: `84a2694580e50650ef6dbe3861efa98c26d73fc8`

This checkpoint adds low-level coverage for packed rerank group payload lookup:

- Direct posting-carried group header TID lookup returns the matching payload.
- Fallback lookup with an invalid direct TID scans loaded groups and returns the
  matching payload.
- Deleted group slots are skipped and do not return stale payloads.

## Non-Claims

This packet is not benchmark evidence and does not close the full PG18 lifecycle
fixture requirement. It covers the private packed-group lookup branch in unit
tests; create/insert/update/delete/vacuum and snapshot-visible end-to-end PG18
fixtures still remain open.

## Validation

Packet-local logs are under `artifacts/` and summarized in
`artifacts/manifest.md`.

- `cargo-test-rerank-group-lookup.log`: `cargo test --no-default-features --features pg18 rerank_group_payload_lookup --lib`
  passed with 2 tests.
- `cargo-check-pg18.log`: `cargo check --no-default-features --features pg18`
  passed.

## Review Focus

- Verify direct lookup and fallback scan semantics match the current packed
  group reader.
- Verify deleted group slots cannot leak stale payloads into index-side rerank.
- Verify this packet is correctly scoped as low-level coverage, not a substitute
  for the remaining PG18 lifecycle fixtures.
