# Validation

Code checkpoint: `8eea5f965` (`Capture physical head membership and scan rounds`).

- `PGRX_PG_CONFIG_PATH=/home/peter/.pgrx/18.3/pgrx-install/bin/pg_config cargo check -p ecaz-cli --offline --all-targets` — passed.
- `PGRX_PG_CONFIG_PATH=/home/peter/.pgrx/18.3/pgrx-install/bin/pg_config cargo test -p ecaz-cli --offline --bin ecaz distann_physical_scan_round_notice_is_structured_from_fixture_summary` — passed.
- Focused PG18 lifecycle test — result recorded below after completion.

The parser regression covers the fixture summary form that previously caused
the structured scan-round rows to be dropped. No benchmark matrix was rerun.

The captured round evidence already in the Task 206 correction packet reports
approximately 1.24 ms transport wait and 1.02 ms straggler spread for a 512 B
request and 112 KiB response. Even allowing eight rounds, that is roughly
10–20 ms of transport time against the approximately 190 ms physical p50;
the remaining latency gap is therefore owner-side compute/serialization, not
the transport wait itself.
