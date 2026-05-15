# Artifact Manifest: SPIRE CustomScan Cost Calibration

- head SHA before this code checkpoint: `45e95bded96e3ecd77f063c9a49197709389a830`
- packet/topic: `30976-spire-cost-calibration`
- timestamp: `2026-05-13T07:41:52Z`
- fixture: reused local PG18 pgrx database `spire_phase12_measure` from packet
  `30975`, with coordinator index `phase12_tuple_measure_coord_idx` and
  loopback remote index `phase12_tuple_measure_remote_idx`.
- lane / storage / rerank: local PG18 loopback, `rabitq`, `nprobe=1/4/8/16`,
  default rerank width from the relation, `pg_binary_attr_v1` tuple transport.
- surface shape: isolated two-index loopback fixture with one available remote
  node and 16 remote placements. This is not a shared-table remote surface.

## Code Constants

Updated in `src/am/ec_spire/custom_scan.rs`:

- `CUSTOM_SCAN_REMOTE_DISPATCH_CPU_UNITS`: `32.0` -> `1024.0`
- `CUSTOM_SCAN_MERGE_CPU_UNITS`: `4.0` -> `0.5`
- added `CUSTOM_SCAN_TUPLE_BYTE_CPU_UNITS = 0.001`

The local calibration showed fixed remote dispatch dominates this small
loopback workload, while output row count and projected tuple width add a
shallower marginal slope. The final modeled-cost rows keep startup dominated by
remote dispatch (`2.56` of `2.60`) but still make output rows, remote fanout,
remote placement count, and projected width monotonic in unit coverage.

## Setup And Validation Artifacts

### `install-ecaz-pg18-pg-test-final.log`

- command:
  `target/debug/ecaz dev install ecaz-pg-test --pg 18 --log-file review/30976-spire-cost-calibration/artifacts/install-ecaz-pg18-pg-test-final.log`
- purpose: install the final calibrated PG18 pg_test extension before
  post-change SQL capture.
- key result: installed `/home/peter/.pgrx/18.3/pgrx-install/lib/postgresql/ecaz.so`
  with sha256 `4c1f6302cd628236ad75c74888cbf04ca669e56163e637b7de380fbb5d6ec9d1`.

### Focused Rust test

- command: `cargo test custom_scan_cost`
- result: passed 3 focused tests:
  - `custom_scan_cost_accounts_for_projected_tuple_width`
  - `custom_scan_cost_increases_with_output_rows`
  - `custom_scan_cost_increases_with_remote_fanout`

## Measurement Artifacts

### `calibrate-spire-cost.sql`

- command source for the latency calibration matrix.
- shape: for each `nprobe` in `1,4,8,16`, run 20 dynamic SQL coordinator KNN
  reads at `k=10,50,100` for both `id_only` and `title_body` projections.
- payload widths:
  - `id_only`: no text payload bytes counted
  - `title_body`: materializes `title` and `body`, with 31,510 bytes for
    `k=10`, about 158k bytes for `k=50`, and about 317k bytes for `k=100`
    across the 20-query run

### `calibrate-spire-cost-final.log`

- command:
  `target/debug/ecaz dev sql --host /home/peter/.pgrx --port 28818 --database spire_phase12_measure --file review/30976-spire-cost-calibration/artifacts/calibrate-spire-cost.sql --log-output review/30976-spire-cost-calibration/artifacts/calibrate-spire-cost-final.log`
- key result lines:
  - `id_only 1 10 20 200 0 32.273 31.674 33.533 39.801 30.986`
  - `id_only 16 100 20 2000 0 42.719 42.521 45.220 45.653 23.409`
  - `title_body 1 10 20 200 31510 32.049 31.775 33.158 34.481 31.202`
  - `title_body 16 100 20 2000 317466 42.503 42.454 43.714 43.879 23.528`
- columns:
  `projection nprobe k query_count rows_returned payload_bytes avg_ms p50_ms p95_ms p99_ms qps`.
- interpretation: one-remote loopback reads stay near 32-34 ms at `k=10` and
  41-43 ms at `k=100`; projected text payload has a small marginal effect
  relative to fixed dispatch.

### `customscan-cost-model-after.sql`

- command source for a SQL mirror of the post-change CustomScan cost formula
  over the measured fixture's eligibility row.
- shape: computes startup and total modeled cost for `id_only` width `8` and
  `title_body` width `175` at `k=10,50,100`.

### `customscan-cost-model-after.log`

- command:
  `target/debug/ecaz dev sql --host /home/peter/.pgrx --port 28818 --database spire_phase12_measure --file review/30976-spire-cost-calibration/artifacts/customscan-cost-model-after.sql --log-output review/30976-spire-cost-calibration/artifacts/customscan-cost-model-after.log`
- key result lines:
  - `id_only 10 1 16 0.040000 2.560000 0.125000 0.012500 0.100000 0.000200 2.600000 2.837700`
  - `id_only 100 1 16 0.040000 2.560000 1.250000 0.125000 1.000000 0.002000 2.600000 4.977000`
  - `title_body 10 1 16 0.040000 2.560000 0.125000 0.012500 0.100000 0.004375 2.600000 2.841875`
  - `title_body 100 1 16 0.040000 2.560000 1.250000 0.125000 1.000000 0.043750 2.600000 5.018750`
- columns:
  `projection output_rows remote_fanout remote_placements routing_traversal_cost remote_dispatch_cost heap_rerank_cost merge_cost tuple_delivery_cost tuple_width_cost modeled_startup_cost modeled_total_cost`.

## Notes

- A packet-local `EXPLAIN` attempt hit the existing v1 DML frontdoor
  fail-closed guard for this remote-placement KNN shape. It is not cited as
  evidence for the calibrated constants; the actual SELECT calibration path and
  the eligibility/cost-model SQL are the cited evidence.
- Additional uncited local logs are now published for visibility:
  `calibrate-spire-cost-baseline.log`, `explain-customscan-after.log`,
  `explain-customscan-after.sql`, `install-ecaz-pg18-pg-test.log`,
  `restart-pg18-loopback-secret-final.log`, and
  `restart-pg18-loopback-secret.log`.
