# Review Request: Structured Snapshot Fallback Guard

## Summary

This checkpoint fixes the fallback guard added in packet 015. The AWS rerun in
packet 014 showed the missing-column text was carried by the structured
Postgres DB error, while `tokio_postgres::Error::to_string()` was too generic.

The guard now checks `err.as_db_error().message()` before falling back to
display text.

## Validation

- `cargo fmt --check` passed with existing stable-rustfmt warnings.
- `cargo test --manifest-path crates/ecaz-cli/Cargo.toml spire_pipeline --locked --offline`
  passed: `21 passed; 0 failed`.

## Follow-Up

Rerun the AWS 1M/q500 retained suite from packet 014 after installing this
commit. AWS 1M was stopped after the previous failed rerun.
