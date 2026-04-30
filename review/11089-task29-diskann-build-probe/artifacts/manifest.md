# Artifact Manifest: `11089-task29-diskann-build-probe`

Packet: `review/11089-task29-diskann-build-probe`
Head SHA: `70aa867de5c7f788ab48dc626f390f67d6aa07ae`
Timestamp: `2026-04-29T20:04:33-07:00`

## build-probe-release-table.log

- head SHA: `70aa867de5c7f788ab48dc626f390f67d6aa07ae`
- packet/topic: `11089-task29-diskann-build-probe`
- lane: Task 29 DiskANN initial tuning
- fixture: real-10k local PG18 corpus
- database: `task29_diskann_baseline`
- prefix: `task29_diskann_real10k`
- storage format: default `ec_diskann` storage
- rerank mode: default profile behavior; this probe only replays in-memory build core
- isolated/shared surface: isolated one-index-per-table prefix
- command:

```text
cargo run -p ecaz-cli --release -- --host /home/peter/.pgrx --port 28818 --database task29_diskann_baseline --log-file review/11089-task29-diskann-build-probe/artifacts/build-probe-release-cli.log bench diskann-build-probe --prefix task29_diskann_real10k --graph-degree 32 --build-list-size 100 --alpha 1.2 --seed 42 --medoid-sample-cap 1024 --log-output review/11089-task29-diskann-build-probe/artifacts/build-probe-release-table.log
```

- key result lines:
  - rows `10000`, dimensions `1536`, medoid `4514`
  - reachable `10000`, reachable_fraction `1.000000`
  - fetch_seconds `12.448`, medoid_seconds `2.004`, build_seconds `73.211`
  - pass 1 alpha `1.000`, pivots `10000`, visited mean/p95 `101.93/106`, pool mean/p95 `101.93/106`, selected mean/p95 `8.22/12`, backlinks `81353`, reprunes `794`
  - pass 2 alpha `1.200`, pivots `10000`, visited mean/p95 `102.55/105`, existing mean/p95 `18.91/31`, pool mean/p95 `105.42/113`, selected mean/p95 `21.59/32`, backlinks `90571`, reprunes `18607`
  - final out degree min/mean/p50/p95/p99/max `1/24.50/25/32/32/32`
  - final in degree min/mean/p50/p95/p99/max `1/24.50/22/43/61/3250`

## build-probe-release-cli.log

- same command and metadata as `build-probe-release-table.log`
- raw mirrored CLI output

## build-probe-cli.log

- aborted attempt
- command:

```text
cargo run -p ecaz-cli -- --host /home/peter/.pgrx --port 28818 --database task29_diskann_baseline --log-file review/11089-task29-diskann-build-probe/artifacts/build-probe-cli.log bench diskann-build-probe --prefix task29_diskann_real10k --graph-degree 32 --build-list-size 100 --alpha 1.2 --seed 42 --medoid-sample-cap 1024 --log-output review/11089-task29-diskann-build-probe/artifacts/build-probe-table.log
```

- outcome: killed after about 12 minutes because the debug build had not completed and produced no table output.
- key result lines: none; `build-probe-cli.log` is empty.
