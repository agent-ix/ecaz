# Artifact manifest

- Head SHA: `cec8abba1770dc500a890c7ad57a932deae4c51c`
- Instrumentation commits: `a675da3e3bb21088431d918b1fe9796686149d99`,
  `cec8abba1770dc500a890c7ad57a932deae4c51c`
- D8 implementation: `de9d6fca3e0bd05f44ad6b0d376a2480e4023798`
  plus test-only follow-up `c9b74c4f258c699def7ffc951e6abf47762565a4`
- Pre-D8 quality baseline: `a375d56dd70f364f8c2389201e5524e578f0ff14`
- Release runner git commit: `cec8abba1770dc500a890c7ad57a932deae4c51c`
- Release runner SHA-256:
  `edb7867abf87a1c5e77972d34807174650940a7209de72730e01b61e4bf9e9aa`
- Installed release extension SHA-256:
  `318624bbce5c20868f1307db16f7abc2f8fda994321119b820dbd37e4f66fbf0`
- Suite config SHA-256:
  `2ae4ce2a8fce91d44596a0ca318676302da3aa92fec1645d6915f8644947dd2f`
- Task bucket / packet: `reviews/task-163/005-d8-scale-memory`
- Branch: `task-179-ec-distann-physical-shards`
- Lane: local Intel PG18 legacy ec_distann sharded build, post-D8
- Host: x86_64 Intel Core i9-10900K, 20 logical CPUs, 62 GiB RAM,
  Linux 6.18.33.2 WSL2, 1 TiB ext4 virtual disk
- PostgreSQL: 18.3 release extension, port 28818, socket `/home/peter/.pgrx`,
  database `ec_distann_bench`, `shared_preload_libraries=ecaz`
- Run: `2026-07-13T05:27:07-07:00` through
  `2026-07-13T05:35:40-07:00`
- Fixture: DBpedia staged real vectors at 10k/50k/100k, dimension 1536;
  current 10k recall uses 200 queries / 2,000 recall@10 trials
- Access method: `ec_distann`, RaBitQ, build shards 4, closure epsilon 0.1,
  seed 42, graph degree/build list/head cap profile defaults
- Storage format: legacy single-node ec_distann persisted graph; temporary
  per-shard PostgreSQL `BufFile` spill during construction
- Rerank mode: legacy ec_distann exact-vector rerank defaults
- Isolation surface: isolated one-index-per-table prefixes
  `d8_current_{10k,50k,100k}`; no shared-table surface

Corpus/query TSVs and truth data are not committed. Their immutable SHA-256
values are recorded in the load logs:

- 10k corpus `c67c5810b66d982d705974e48d4775479adfbd92a988f694091266e049a35e75`,
  queries `a2c191bb742017d849e73f6e6866e8e0f0bac1579ba212f7fc76b8eb09904ae8`;
- 50k corpus `56023baaa7bc42f758272e8617603d538808e6290a8a70a3a84e057571240133`,
  queries `95ac7992578aa80bb193657f10fbcbf1ea3867e559739244bf5a467f7a5a9fa3`;
- 100k corpus `07275cfd5a7a4b415ddf5eacc086de98294ac978532df46ffae30f9202323a95`,
  queries `a7cbec6fc44f6c148234538f61339d00d2f10646febc8f667dcbe75d9cf41782`.

## Commands

Exact release extension installation:

```text
cargo pgrx install --release \
  --pg-config /home/peter/.pgrx/18.3/pgrx-install/bin/pg_config \
  --no-default-features --features pg18
```

Exact release runner build:

```text
cargo build --release -p ecaz-cli
```

Canonical suite:

```text
target/release/ecaz --host /home/peter/.pgrx --port 28818 \
  --database ec_distann_bench bench suite run \
  --config reviews/task-163/005-d8-scale-memory/artifacts/candidate-suite.json \
  --continue-on-error \
  --log-file reviews/task-163/005-d8-scale-memory/artifacts/suite-run.log
```

Final audit/status and focused validation commands are captured verbatim in
their named packet artifacts.

## Artifact index

- `candidate-suite.json`: canonical five-step config and 11 thresholds.
- `candidate/suite-manifest.json`: exact commands, source/config provenance,
  durations, 5/5 success, and 11/11 threshold results.
- `candidate/results.jsonl`: structured build-memory, shard-build NOTICE, and
  recall rows.
- `candidate/load-{10k,50k,100k}.log`: corpus hashes, exact build options,
  durable PostgreSQL NOTICE, memory samples, and build time.
- `candidate/recall-10k.log`: current 200-query recall sweep.
- `candidate/precheck-host.log`: release profile and PG18 settings.
- `comparison.md`: derived scale table and immutable pre-D8/current quality
  comparison.
- `release-extension-install.log`, `release-cli-build.log`: exact build/install
  provenance.
- `configure-preload.log`, `create-database.log`,
  `create-extension-final.log`: isolated scratch-database setup.
- `audit-final.log`, `status.log`, `suite-run.log`: canonical suite lifecycle.
- `shard-tests.log`, `cli-tests.log`, `cargo-check.log`: focused validation.

## Key cited results

```text
status: completed=5 failed=0 missing_artifacts=0 stale=0
thresholds: 11/11 pass

scale  hwm_peak_kib  spill_bytes  completion_peak_bytes  stitch_retained_bytes
10k       397428       1283964          464244                  35784
50k     1185676       8505972         3307900                  36104
100k    2170028      17524784         6289260                  36240

pre-D8/current recall@10, widths 16/32/64/100/200:
0.9950 / 0.9985 / 1.0000 / 1.0000 / 1.0000 in both arms
```
