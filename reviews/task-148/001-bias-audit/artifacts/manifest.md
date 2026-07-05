# Task 148 / Packet 001 Bias Audit Artifact Manifest

- task bucket: `reviews/task-148/001-bias-audit`
- product head at audit run: `4003fbc6be20e110cd6b57dafef4f3598a0ea1a3`
- audit runner commit: `edafc52d1e6726fc032a3c6ef32b53d83be173d9`
- lane: local staged TSV audit, no SQL/index build, no product scoring path change
- storage format / rerank mode: TurboQuant 4-bit no-QJL offline encode/decode only
- pairing: corpus row ordinal paired with query ordinal modulo query row count
- filtered query-dot threshold: `abs(<q,x>) >= 0.02`
- timestamp: 2026-07-05 local session; artifact mtimes recorded by filesystem

## Artifacts

### `tq-bias-audit-10k.log` / `tq-bias-audit-10k.json`

Command:

```sh
./target/release/ecaz bench tq-bias-audit --label ec_real_10k --corpus data/staged-current/ec_real_10k_corpus.tsv --queries data/staged-current/ec_real_10k_queries.tsv --log-output reviews/task-148/001-bias-audit/artifacts/tq-bias-audit-10k.log --json-output reviews/task-148/001-bias-audit/artifacts/tq-bias-audit-10k.json
```

Key result lines:

- rows: `10000`, query rows: `200`
- `norm_ratio`: mean `0.99264737`, p01 `0.98168747`, p99 `1.00272913`
- `self_dot_ratio`: mean `0.98741150`, p01 `0.97532244`, p99 `0.99807919`
- `query_dot_ratio_filtered`: mean `0.98637467`, p01 `0.89654055`, p99 `1.06605646`

### `tq-bias-audit-50k.log` / `tq-bias-audit-50k.json`

Command:

```sh
./target/release/ecaz bench tq-bias-audit --label ec_real_50k --corpus data/staged-current/ec_real_50k_corpus.tsv --queries data/staged-current/ec_real_50k_queries.tsv --log-output reviews/task-148/001-bias-audit/artifacts/tq-bias-audit-50k.log --json-output reviews/task-148/001-bias-audit/artifacts/tq-bias-audit-50k.json
```

Key result lines:

- rows: `50000`, query rows: `1000`
- `norm_ratio`: mean `0.99273127`, p01 `0.98158664`, p99 `1.00266888`
- `self_dot_ratio`: mean `0.98751405`, p01 `0.97522872`, p99 `0.99803592`
- `query_dot_ratio_filtered`: mean `0.98672958`, p01 `0.85926145`, p99 `1.11615593`

### `tq-bias-audit-100k.log` / `tq-bias-audit-100k.json`

Command:

```sh
./target/release/ecaz bench tq-bias-audit --label ec_real_100k --corpus data/staged-current/ec_real_100k_corpus.tsv --queries data/staged-current/ec_real_100k_queries.tsv --log-output reviews/task-148/001-bias-audit/artifacts/tq-bias-audit-100k.log --json-output reviews/task-148/001-bias-audit/artifacts/tq-bias-audit-100k.json
```

Key result lines:

- rows: `100000`, query rows: `1000`
- `norm_ratio`: mean `0.99278999`, p01 `0.98167054`, p99 `1.00271736`
- `self_dot_ratio`: mean `0.98758200`, p01 `0.97532644`, p99 `0.99806855`
- `query_dot_ratio_filtered`: mean `0.98681072`, p01 `0.85721613`, p99 `1.11723377`

### `tq-bias-audit-anchor-990k.log` / `tq-bias-audit-anchor-990k.json`

Command:

```sh
./target/release/ecaz bench tq-bias-audit --label ec_real_anchor_990k --corpus data/staged-current/ec_real_ann_benchmarks_anchor_corpus.tsv --queries data/staged-current/ec_real_ann_benchmarks_anchor_queries.tsv --log-output reviews/task-148/001-bias-audit/artifacts/tq-bias-audit-anchor-990k.log --json-output reviews/task-148/001-bias-audit/artifacts/tq-bias-audit-anchor-990k.json
```

Key result lines:

- rows: `990000`, query rows: `10000`
- `norm_ratio`: mean `0.99281619`, p01 `0.98174221`, p99 `1.00268438`
- `self_dot_ratio`: mean `0.98761515`, p01 `0.97540358`, p99 `0.99802266`
- `query_dot_ratio_filtered`: mean `0.98661024`, p01 `0.84083837`, p99 `1.13011197`

## Validation

- `cargo check -p ecaz-cli` passed after adding the audit command.
- `cargo build -p ecaz-cli --release` passed and produced the runner used for the audit artifacts.
- `cargo test -p ecaz-cli distribution_percentiles_are_interpolated` did not reach the new audit test because existing `suite.rs` test initializers fail to compile: missing `runner_git_commit` in `SuiteManifest` and missing `ivf_stage_counters` in `LatencyStep`.

## Verdict

The per-vector spread is not negligible. The stable norm shrinkage (`norm_ratio` mean about `0.9928`, p01 about `0.9817`) and filtered paired-query score spread support proceeding to Slice 2 length renormalization instead of closing the renorm branch as a measured negative.
