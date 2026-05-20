# Task 50: SPIRE DML Relation Catalog Helpers

## Summary

This packet advances the SPIRE production target in P2/P11 by making the DML
frontdoor relation-catalog helper layer safe at its call sites. The helpers
already validate/null-check and copy PostgreSQL relcache/catalog data into
owned Rust values; callers no longer need direct unsafe wrappers around that
contract.

Covered helper surfaces:

- relation-context cache fill from an open heap relation;
- tuple descriptor access;
- relation column-name extraction;
- ec_spire index and primary-key inspection;
- index-key column extraction; and
- type-name formatting.

## Code Under Review

- code commit: `14a7276dd9f7c550d1ff73a26bc6dd573701abbf`
- previous packet baseline: `099e0bbcafa37e658ec18b2e18b26d510451e138`
- touched file: `src/am/ec_spire/dml_frontdoor/mod.rs`

## Unsafe Movement

Packet-local count artifacts:

- `artifacts/before-counts.log`
- `artifacts/after-counts.log`
- `artifacts/unsafe-ledger-after.jsonl`

Direct unsafe blocks in the touched file:

| File | Before | After |
| --- | ---: | ---: |
| `src/am/ec_spire/dml_frontdoor/mod.rs` | 100 | 91 |

Current `src/` ledger after this packet:

- `2344` direct unsafe blocks
- `132` files
- ledger check: `ledger covers 2344 current unsafe rows`

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

Benchmarks were not run. This change only removes redundant unsafe wrappers
around relation-catalog helper contracts and preserves the copied catalog facts.
