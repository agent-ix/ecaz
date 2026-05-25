# Task 59 Packet 004: Cloud Install Retained Tables

## Summary

Code commit under review:

- `dbd1a0c01` (`Allow retained-table cloud installs`)

This checkpoint adds `ecaz cloud install --skip-extension-recreate`.

The AWS Graviton tuning lane keeps benchmark tables live between optimization
cycles. The existing install command successfully built and installed the
extension files, installed the CLI, and restarted PostgreSQL, but then failed at:

```text
DROP EXTENSION IF EXISTS ecaz;
```

because retained benchmark tables depend on extension-owned types such as
`ecvector`.

The new flag preserves the normal build/copy/restart behavior but replaces the
drop/create sequence with an extension presence/version query. This is suitable
for Rust-only scan-path changes where the PostgreSQL catalog SQL does not need
to be recreated.

## Evidence

The failure mode is recorded in the benchmark packet:

- `benchmarks/task59-aws-diskann-duplicate-expansion-fast-path/artifacts/cloud-install-diskann-aws-optimization.log`
- `benchmarks/task59-aws-diskann-duplicate-expansion-fast-path/artifacts/cloud-install-frontier-heap.log`

Both logs show the remote host completed:

- `cargo pgrx install --sudo --release --pg-config /usr/bin/pg_config`
- `cargo build --release -p ecaz-cli`
- `sudo install -Dm755 ... /usr/local/bin/ecaz`
- `sudo systemctl restart postgresql`

and then failed only at extension drop because Task 55 benchmark tables depended
on `ecvector`.

## Validation

- `cargo check -p ecaz-cloud` passed; see `artifacts/cargo-check-ecaz-cloud.log`.
