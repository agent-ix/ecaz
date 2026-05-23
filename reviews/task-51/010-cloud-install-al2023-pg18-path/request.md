# Review Request: Cloud Install AL2023 PG18 Path

Please review the Task 51 cloud install fix:

- code: `crates/ecaz-cloud/src/commands/install.rs`
- validation log: `reviews/task-51/010-cloud-install-al2023-pg18-path/artifacts/cargo-check-ecaz-cloud.log`

## Scope

The AWS final-gate bring-up exposed that `ecaz cloud install` still used the old PGDG-style paths:

- `/usr/pgsql-18/bin/pg_config`
- `postgresql-18`

The AL2023 cloud-init path in this repo installs PostgreSQL 18 packages using the AL2023 layout instead:

- `/usr/bin/pg_config`
- `postgresql`

This packet updates the install command to match cloud-init. It does not change Terraform, benchmark config, table/index layout, or benchmark behavior.

## Validation

```text
cargo check -p ecaz-cloud
Finished `dev` profile [unoptimized + debuginfo] target(s) in 2.77s
```

## Caveat

The running AWS final-gate host was already brought up from the preserved snapshot before this code commit. For that host, I used SSM with the corrected path directly and captured those AWS artifacts under `benchmarks/task51-aws-ivf-rabitq-final-gate/`.
