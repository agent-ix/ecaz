# Task 94 Review Request: SVE Vector-Lane Warning Cleanup

## Scope

This checkpoint removes a local PG18 feature-build warning observed in packet
014. The grouped-PQ SVE runtime vector-lane helper is only called by the
test-only Graviton evidence helper today, so the private helper is now gated
behind `#[cfg(test)]`.

This keeps the Graviton 4 test evidence path intact while preventing non-test
builds from warning on an otherwise unused private helper.

## Code

- `5289ae91d` - `Gate grouped-PQ SVE vector lane helper to tests`

## Validation

- `cargo fmt --check`: passed; see `artifacts/cargo-fmt-check.log`.
- `cargo test grouped_pq --lib`: passed; see `artifacts/cargo-test-grouped-pq-lib.log`.

Key result:

```text
test result: ok. 35 passed; 0 failed; 0 ignored; 0 measured; 2018 filtered out
```

The packet-local grouped-PQ test log includes the PG18 feature build and no
longer reports the previous `runtime_vector_lanes` dead-code warning.

No CI, AWS, or benchmark runs were used for this packet.
