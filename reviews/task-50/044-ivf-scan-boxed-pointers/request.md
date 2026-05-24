# Task 50: IVF Scan Boxed Pointer Helpers

## Summary

This packet continues the IVF/RaBitQ P10 scan-opaque cleanup. IVF scan-owned
boxed pointers now go through local helper contracts for:

- immutable scan-owned boxed references;
- mutable scan-owned boxed references; and
- dropping `Box::into_raw` slots after clearing the owning pointer.

The rollout covers prepared queries, PQ fastscan models, the candidate dedup
map, and heap rerank state ownership.

## Code Under Review

- code commit: `9acdc809538e0b511abe492230e8a5c179d42ed8`
- previous packet baseline: `b0d44a22e4544cac1b9a41162061a2726cc8c940`
- touched file: `src/am/ec_ivf/scan.rs`

## Unsafe Movement

Packet-local count artifacts:

- `artifacts/before-counts.log`
- `artifacts/after-counts.log`
- `artifacts/unsafe-ledger-after.jsonl`

Direct unsafe blocks in the touched file:

| File | Before | After |
| --- | ---: | ---: |
| `src/am/ec_ivf/scan.rs` | 56 | 46 |

Current `src/` ledger after this packet:

- `2353` direct unsafe blocks
- `132` files
- ledger check: `ledger covers 2353 current unsafe rows`

## Validation

Packet-local validation artifacts:

- `artifacts/cargo-check-pg18-bench.log`
- `artifacts/unsafe-ledger-generate.log`
- `artifacts/unsafe-ledger-check.log`

Validation run:

```text
cargo check --all-targets --no-default-features --features pg18,bench
```

Result: passed. The log contains the existing unrelated
`src/am/mod.rs` unused-import warning.

Benchmarks were not run. This change does not alter scoring math, selected list
ordering, posting payloads, or heap rerank semantics; it only centralizes
scan-owned boxed pointer access and cleanup.
