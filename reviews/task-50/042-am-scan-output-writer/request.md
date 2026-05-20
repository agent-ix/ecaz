# Task 50: AM Scan Output Writer

## Summary

This packet advances the P2/P10 PostgreSQL scan descriptor contract across the
production AM scan output path. IVF, SPIRE, and HNSW all had local unsafe
helpers that wrote the same PostgreSQL `IndexScanDesc` output fields:

- `xs_heaptid`
- `xs_orderbyvals`
- `xs_orderbynulls`

Those writes now go through `src/am/common/scan_output.rs`. The per-AM wrappers
remain only to preserve local call names and AM-specific fault-injection
context strings.

## Code Under Review

- code commit: `bffe6f84bf5e9622a0040c0ff606a7808ef832d7`
- previous packet baseline: `d2dd0ecb3f868da9be814c1170f9390db45ca732`
- touched files:
  - `src/am/common/mod.rs`
  - `src/am/common/scan_output.rs`
  - `src/am/ec_hnsw/scan.rs`
  - `src/am/ec_ivf/scan.rs`
  - `src/am/ec_spire/scan.rs`
  - `src/am/ec_spire/scan/relation.rs`

## Unsafe Movement

Packet-local count artifacts:

- `artifacts/before-counts.log`
- `artifacts/after-counts.log`
- `artifacts/unsafe-ledger-after.jsonl`

Direct unsafe blocks in the touched scan-output files:

| File | Before | After |
| --- | ---: | ---: |
| `src/am/ec_hnsw/scan.rs` | 149 | 146 |
| `src/am/ec_ivf/scan.rs` | 62 | 59 |
| `src/am/ec_spire/scan/relation.rs` | 18 | 15 |
| `src/am/common/scan_output.rs` | 0 | 3 |

Net current `src/` ledger after this packet:

- `2366` direct unsafe blocks
- `132` files
- ledger check: `ledger covers 2366 current unsafe rows`

The new common file owns the remaining three PostgreSQL scan-output boundary
operations. The three AM call sites no longer directly mutate the scan output
arrays or heap TID field.

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

Benchmarks were not run. This is a scan descriptor output writer consolidation;
it preserves the same score values, null flags, heap TID fields, and
fault-injection allocation labels.
