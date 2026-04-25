# Artifact Manifest

Packet: `629-c1-native-decoded-score-workspace-measurement`

Head SHA: `76e1b6cfe39180616f1ac6d884a8b4f924941c63`

Baseline SHA: `184a030` (`Add native graph scratch cache measurement packet`)

Current code checkpoint: `76e1b6c` (`Precompute native build code score values`)

Timestamp:
- Baseline run: `2026-04-25T12:36:11-07:00` through `2026-04-25T12:46:27-07:00`
- Current run: `2026-04-25T12:47:08-07:00` through `2026-04-25T12:49:06-07:00`

Lane: PG18 pgrx local cluster, PostgreSQL 18.3, port 28818.

Fixture:
- Table: `ec_hnsw_native_decoded_score_10k1536_measure`
- Rows: 10,000
- Dimensions: 1,536
- Encoded type: `tqvector`
- Storage format: default `turboquant`
- Rerank mode: default, no `build_source_column`
- Index reloptions: `m = 6, ef_construction = 40`

Surface:
- Isolated one-table fixture.
- One `ec_hnsw` index existed at a time.
- Serial run dropped its index before the parallel run.
- Serial run used `max_parallel_maintenance_workers = 0` and table
  `parallel_workers = 0`.
- Parallel run used `max_parallel_maintenance_workers = 4` and table
  `parallel_workers = 4`.
- Phase counters came from `tests.ec_hnsw_debug_last_build_timing()`.
- The fixture intentionally uses indexed `tqvector`, not `ecvector`, so native
  graph build uses `BuildGraphMetric::Code` and exercises the decoded score
  workspace. `ecvector` builds keep raw source vectors and score with
  `BuildGraphMetric::Source`, which is not this optimization path.

Commands:

Baseline install:

```sh
cargo pgrx install --pg-config /home/peter/.pgrx/18.3/pgrx-install/bin/pg_config --features 'pg18 pg_test' --no-default-features
```

Baseline measurement:

```sh
cargo run -p ecaz-cli -- --database postgres dev sql --pg 18 --raw --file review/629-c1-native-decoded-score-workspace-measurement/artifacts/pg18_native_decoded_score_workspace_10k1536_timing.sql --log-output review/629-c1-native-decoded-score-workspace-measurement/artifacts/pg18_native_decoded_score_workspace_10k1536_baseline_184a030.log
```

Current install:

```sh
cargo pgrx install --pg-config /home/peter/.pgrx/18.3/pgrx-install/bin/pg_config --features 'pg18 pg_test' --no-default-features
```

Current measurement:

```sh
cargo run -p ecaz-cli -- --database postgres dev sql --pg 18 --raw --file review/629-c1-native-decoded-score-workspace-measurement/artifacts/pg18_native_decoded_score_workspace_10k1536_timing.sql --log-output review/629-c1-native-decoded-score-workspace-measurement/artifacts/pg18_native_decoded_score_workspace_10k1536_current_76e1b6c.log
```

Artifacts:
- `pg18_native_decoded_score_workspace_10k1536_timing.sql`: SQL fixture and
  timing script.
- `pg18_native_decoded_score_workspace_10k1536_baseline_184a030.log`: raw psql
  output for baseline `184a030`.
- `pg18_native_decoded_score_workspace_10k1536_current_76e1b6c.log`: raw psql
  output for current head `76e1b6c`.

Key Result Lines:
- Baseline fixture load: `INSERT 0 10000`, `Time: 8885.167 ms (00:08.885)`
- Current fixture load: `INSERT 0 10000`, `Time: 8948.604 ms (00:08.949)`
- Baseline serial create index: `Time: 304071.708 ms (05:04.072)`
- Current serial create index: `Time: 54708.383 ms (00:54.708)`
- Baseline parallel create index: `Time: 303680.367 ms (05:03.680)`
- Current parallel create index: `Time: 54874.921 ms (00:54.875)`
- Baseline serial phases: heap ingest `134842 us`, graph `303529692 us`,
  stage `277494 us`, write `106122 us`
- Current serial phases: heap ingest `131757 us`, graph `54198793 us`,
  stage `254804 us`, write `99949 us`
- Baseline parallel phases: heap ingest `137090 us`, begin `2441 us`,
  drain `11335 us`, sort/push `123309 us`, graph `303183836 us`,
  stage `246880 us`, write `94121 us`
- Current parallel phases: heap ingest `156639 us`, begin `2997 us`,
  drain `25125 us`, sort/push `128512 us`, graph `54360998 us`,
  stage `247937 us`, write `90365 us`
- Both baseline and current measured index sizes: `11739136` bytes
