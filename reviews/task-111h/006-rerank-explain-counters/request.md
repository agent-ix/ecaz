# Task 111h / 006 - Rerank EXPLAIN Counter Checkpoint

## Scope

Code commit: `026ad1b12aaaf915eae09181448755ba72844e13`

This checkpoint adds IVF EXPLAIN observability for the packed index-side rerank
path:

- Adds text-valued EXPLAIN properties and reports effective `Rerank Placement`
  and `Rerank Format`.
- Reports packed index-side group/header read accounting:
  - `Rerank Index Group Header Pages Read`
  - `Rerank Index Payload Segment Pages Read`
  - `Rerank Index Group Metadata Bytes Read`
  - `Rerank Index Header Payload Bytes Read`
  - `Rerank Index Segment Payload Bytes Read`
- Reports exact-rerank compact payload work:
  - `Rerank Payload Bytes Scored`
  - `Rerank Payload Slab Bytes Copied`
- Threads the counters through the current `0x2B` / `0x2C` packed group reader.
- Updates stale code comments that described the current index placement as the
  legacy `0x2A` sidecar.

## Non-Claims

This packet is not benchmark evidence and does not close the full Task 111h
EXPLAIN/admin/counter checklist item by itself. It adds the EXPLAIN/counter
coverage needed to make later benchmark packets less ambiguous. Admin inspection
coverage and any decode-stage timing split remain follow-up work.

## Validation

Packet-local logs are under `artifacts/` and summarized in
`artifacts/manifest.md`.

- `cargo-test-ivf-explain.log`: `cargo test --no-default-features --features pg18 ivf_explain --lib`
  passed with 2 tests.
- `cargo-check-pg18.log`: `cargo check --no-default-features --features pg18`
  passed.

## Review Focus

- Verify the new text-valued EXPLAIN property path is safe for PG18 and does not
  affect non-PG18 builds.
- Verify packed group read counters count unique direct survivor groups, not
  candidate rows repeatedly.
- Verify the copy counter reflects only the current batched compact payload
  slab copy and remains zero for non-batched f16.
- Verify the comments no longer mislabel the current packed v5 path as `0x2A`.
