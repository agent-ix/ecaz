# Task 50: IVF Scan-Owned Slices

## Summary

This packet advances the P10 scan-opaque ownership cleanup in the IVF/RaBitQ
lane. `EcIvfScanOpaque` now exposes its scan-owned arrays through one local
`scan_owned_slice` helper tied to the opaque borrow instead of repeating raw
slice construction and pointer arithmetic in each accessor.

Covered accessors:

- query values;
- test/debug query values;
- selected probe lists; and
- posting candidate iteration.

## Code Under Review

- code commit: `f49a76bc38f8d90592cdef6c030a32f7e97d56c3`
- previous packet baseline: `95e51547d99bd91d9b71bf7654287dddcacb207c`
- touched file: `src/am/ec_ivf/scan.rs`

## Unsafe Movement

Packet-local count artifacts:

- `artifacts/before-counts.log`
- `artifacts/after-counts.log`
- `artifacts/unsafe-ledger-after.jsonl`

Direct unsafe blocks in the touched file:

| File | Before | After |
| --- | ---: | ---: |
| `src/am/ec_ivf/scan.rs` | 59 | 56 |

Current `src/` ledger after this packet:

- `2363` direct unsafe blocks
- `132` files
- ledger check: `ledger covers 2363 current unsafe rows`

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

Benchmarks were not run. This change only centralizes scan-owned array borrows;
it does not change candidate scoring, selected list order, or posting payload
bytes.
