# Artifact manifest

- Packet head at canonical run: `45b2249a09d3fa511b526e48532654cd2653e36d`
- Measured implementation: `a4d374c2f294dc209b1b0f499bd527e52b375b06`
- Release runner git commit:
  `07371ff3f9047701bacccbc32bb9b5043414bf78`
- Release runner SHA-256:
  `7e121cbeb039bb7a204d4c9bbb0dd3fac63782fad55b4db2a5b4cea6ee0f6e2c`
- Installed production extension source: `a4d374c2f` (no `src/` or `sql/`
  changes through runner commit `07371ff3f`)
- Installed extension SHA-256:
  `8521ab044440e235bb361101214fc046ffea5a00b7a221ef59fb6b42001c18b5`
- Suite config SHA-256:
  `afabfc3b5f2b400a2d360352bc0ac8ec95f982fa005a71c027bd834395f13e5d`
- Task bucket / packet: `reviews/task-179/057-build-gate-dml-ab`
- Branch: `task-179-ec-distann-physical-shards`
- Lane: local PG18 inactive durable-gate single-row DML microbenchmark
- Host: x86_64 Intel Core i9-10900K, 20 logical CPUs, 62 GiB RAM, Linux WSL2
- PostgreSQL: 18.3, one postmaster restarted at
  `2026-07-13T07:11:37.50386-07:00`, `shared_preload_libraries=ecaz`
- Canonical run: `2026-07-13T07:14:55-07:00` through
  `2026-07-13T07:14:59-07:00`
- Fixture: fresh `template0` databases `task179_gate_control` and
  `task179_gate_installed`; one unlogged two-bigint heap per database
- Storage format: ordinary unlogged heap; no index, corpus, or shared table
- Rerank mode: not applicable
- Isolation surface: one independent table per database/arm

This is a narrow DML-hook latency measurement. It makes no recall, index
latency, storage, durability, or promotion claim.

## SQL fixture hashes

- `create-databases.sql`:
  `9c7a46a426a53da9b40ddabba82013fb853b4aa006033600b915c1c351564f86`
- `install-extension.sql`:
  `4a20bd69eb39cb2a962af8b4ff72724262c6bbc224e9c8bd11b88bb7902ffb0b`
- `prepare-lane.sql`:
  `6dbffb91fb81f586615126989cf377abb6877f6b731b64e5b020d3fec88f6d76`
- `measure-lane.sql`:
  `e46d632176f7b63dabacffca8ade2762896efeb58512c34fa3d0d1cbb59200d2`
- `compare-ab.sql`:
  `3d213d5159ff1e115203d39a486c208ba86b0afe18d252a82fab5af47a20c249`

## Commands

Production extension installation:

```text
cargo pgrx install --release \
  --pg-config /home/peter/.pgrx/18.3/pgrx-install/bin/pg_config \
  --no-default-features --features pg18
```

Exact release runner build and postmaster restart:

```text
cargo build --release -p ecaz-cli
target/release/ecaz dev scratch restart --pg 18
```

Audit, canonical suite, status, and report:

```text
target/release/ecaz bench suite audit \
  --config reviews/task-179/057-build-gate-dml-ab/artifacts/dml-gate-suite.json

target/release/ecaz bench suite run \
  --config reviews/task-179/057-build-gate-dml-ab/artifacts/dml-gate-suite.json

target/release/ecaz bench suite status \
  --manifest reviews/task-179/057-build-gate-dml-ab/artifacts/run/suite-manifest.json

target/release/ecaz bench suite report \
  --manifest reviews/task-179/057-build-gate-dml-ab/artifacts/run/suite-manifest.json
```

## Artifact index

- `dml-gate-suite.json`: canonical nine-step ABBA config and nine thresholds.
- `create-databases.sql`, `install-extension.sql`, `prepare-lane.sql`,
  `measure-lane.sql`, `compare-ab.sql`: checked-in SQL fixtures.
- `run/suite-manifest.json`: expanded commands, runner SHA, exact timing,
  artifacts, and 9/9 threshold results.
- `run/results.jsonl`: 25 normalized setup, per-trial, per-round, and A/B rows.
- `run/control-round-*.log`, `run/installed-round-*.log`: raw per-trial output.
- `run/compare-ab.log`: compact canonical aggregate.
- `runtime-provenance.log`: server version, preload setting, postmaster start,
  extension version, and `ENABLE ALWAYS` (`tgenabled=A`) trigger proof.
- `release-extension-install.log`, `release-cli-build.log`: exact production
  build/install logs.
- `audit.log`, `suite-run.log`, `status.log`, `report.md`: runner lifecycle and
  human-readable result summary.

PostgreSQL server logs and database relation files are not committed.

## Key cited results

```text
completed=9 failed=0 missing_artifacts=0 stale=0
thresholds: 9/9 pass

control_samples=8 installed_samples=8
control_median_us=6.988 installed_median_us=6.903
delta_us=-0.085 ratio=0.988 overhead_pct=-1.2
control_p95_us=7.489 installed_p95_us=7.416 p95_ratio=0.990
```
