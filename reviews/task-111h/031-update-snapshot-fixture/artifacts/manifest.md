# Artifact Manifest

Packet: `reviews/task-111h/031-update-snapshot-fixture`

Task bucket: `reviews/task-111h`

Code commit under review:
`ad518f14a13ed0239f039b13e551173813147764`

Created: `2026-06-20`

## Scope

This packet covers one Task 111h correctness fixture slice:

- update-path compact rerank payload maintenance,
- snapshot-visible old/new tuple behavior through SQL index scans,
- post-update index-side f16 payload byte/source-byte counters.

Storage format / rerank mode:

- `storage_format = 'coarse_rerank'`
- `rerank_placement = 'index'`
- `rerank_format = 'f16'`
- `rerank_width = 8`

The fixture uses an executor SQL query path for MVCC visibility assertions
because the lower-level `debug_ec_ivf_gettuple_counter_snapshot` helper drives
the AM directly and does not represent executor heap visibility for invisible
index entries.

## Artifacts

| Artifact | Description |
| --- | --- |
| `cargo-pgrx-test-pg18-update-snapshot.log` | Initial focused PG18 run; failed at compile time because the original test name exceeded PostgreSQL's identifier limit. |
| `cargo-pgrx-test-pg18-update-snapshot-pass.md` | Successful focused PG18 rerun output summary. |

## Commands

Initial failed attempt:

```sh
script -q -e -c "cargo pgrx test pg18 test_ec_ivf_index_placement_update_uses_snapshot_visible_payload" reviews/task-111h/031-update-snapshot-fixture/artifacts/cargo-pgrx-test-pg18-update-snapshot.log
```

Successful focused run:

```sh
cargo pgrx test pg18 test_ec_ivf_index_placement_update_snapshot_payload
```

## Key Result Lines

```text
test tests::pg_test_ec_ivf_index_placement_update_snapshot_payload ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 2206 filtered out; finished in 86.25s
```

## Non-Claims

This is not a final Task 111h closeout packet. It does not provide legacy
`0x2A` benchmark evidence, table-owned persisted compact storage evidence,
copy/slab cleanup or benchmark-away evidence, cold/remote benchmark evidence, or
the final promote/iterate/abandon decision.
