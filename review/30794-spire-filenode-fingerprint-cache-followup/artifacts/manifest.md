# Artifact Manifest: SPIRE Filenode Fingerprint Cache Follow-up

Head SHA: `bd9341f5ea51326f4de8539ed5ff8c5e5668a212`
Packet: `30794-spire-filenode-fingerprint-cache-followup`
Timestamp: `2026-05-10`

## Fixture

- Lane: SPIRE Stage E lifecycle reviewer follow-up
- Fixture: focused PG18 pgrx identity-cache matrix
- Storage format: `rabitq`
- Rerank mode: production libpq executor identity-cache summary/probe
- Surface: isolated pg_test fixture tables
- Command:

```text
script -q -c "cargo pgrx test pg18 test_ec_spire_remote_search_libpq_executor_loopback_empty" review/30794-spire-filenode-fingerprint-cache-followup/artifacts/identity-cache-pg18.log
```

## Artifacts

### `identity-cache-pg18.log`

Raw focused PG18 pgrx test log.

Key result lines:

```text
test tests::pg_test_ec_spire_remote_search_libpq_executor_loopback_empty ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1589 filtered out; finished in 26.76s
Script done on 2026-05-10 18:46:22-07:00 [COMMAND_EXIT_CODE="0"]
```
