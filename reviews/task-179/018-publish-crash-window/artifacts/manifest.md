# Packet 018 — publish transaction/crash boundary

Task bucket: `reviews/task-179/`; packet `018-publish-crash-window/`.
Head SHA: `b87523e716172e16e6e6355acf1d9c130955b92f`.
Surface: T3/T4 transaction boundary and single-node T4a crash recovery.

| File | Command | Key result |
|---|---|---|
| `validation.log` | commands listed in the artifact | Two focused PG18 tests and strict clippy pass |

The positive lifecycle test uses a real loopback PostgreSQL backend with one
autocommit transaction per lifecycle endpoint. This is correctness evidence, not
benchmark evidence; no corpus or measurement surface is involved.
