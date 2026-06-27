# Cell Checkpoint: phase1-rabitq-100k-l1-control

Status: checkpoint prepared; not started.

## Intent

- Checklist cell: `phase1-rabitq-100k-l1-control`.
- Phase: 1, single-node multi-disk / multi-store control.
- Scale: 100k representative corpus.
- Storage format: RaBitQ.
- Bits: 4.
- Store count: 1.
- Prefix: `task107_phase1_rabitq_100k_l1`.
- Index: `task107_phase1_rabitq_100k_l1_idx`.
- Artifact directory:
  `reviews/task-107/004-distributed-completion/artifacts/phase1-rabitq-100k-l1-control/`.
- Work directory:
  `reviews/task-107/004-distributed-completion/work/phase1-rabitq-100k-l1-control/`.

## Execution Policy

Run the cell to completion or command failure. The planned work covers AWS
start/status checks, SSM tunnel preflight, 100k corpus fetch/prepare reuse or
regeneration, coordinator load/build, one single-node `ecaz bench suite` run,
storage capture, cleanup, and AWS stop.

## Stop Conditions

- Stop immediately if the 100k prepared corpus would be replaced by a 1m
  corpus or by the default distributed prefix accidentally.
- Stop immediately if another `task107_%` relation already exists on the
  coordinator before load.
- Stop immediately if the suite runner cannot express the recall/latency
  matrix without code or script changes.
- Stop and package failure if cleanup does not leave zero
  `task107_phase1_rabitq_100k_l1%` relations.

## Planned Commands

All commands run from repository root. AWS instance start/stop commands must
write packet-local artifacts.

### Start Topology

```bash
scripts/spire-aws/start-topology-instances.sh \
  reviews/task-107/002-aws-provisioning/artifacts/aws-topology.json \
  reviews/task-107/004-distributed-completion/artifacts/phase1-rabitq-100k-l1-control/aws-start
```

### Refresh AutoStop

Use an 8-hour AutoStop tag and save the `describe-instances` output under
`artifacts/phase1-rabitq-100k-l1-control/aws-start/`.

### Preflight

```bash
scripts/spire-aws/with-ssm-port-forwards.sh \
  reviews/task-107/002-aws-provisioning/artifacts/aws-topology.json \
  reviews/task-107/004-distributed-completion/artifacts/phase1-rabitq-100k-l1-control/preflight \
  reviews/task-107/004-distributed-completion/artifacts/phase1-rabitq-100k-l1-control/preflight/aws-topology.tunneled.json \
  -- \
  target/release/ecaz dev sql \
    --host 127.0.0.1 --port 15432 --user ecaz_coord --database postgres \
    --sql "SELECT current_database(), current_user; SELECT relname FROM pg_class WHERE relname LIKE 'task107_%' ORDER BY relname;" \
    --log-output reviews/task-107/004-distributed-completion/artifacts/phase1-rabitq-100k-l1-control/preflight/coordinator-task107-objects.log
```

### Fetch And Prepare 100k Corpus

```bash
target/release/ecaz corpus fetch \
  --dataset qdrant-dbpedia-openai3-large-1536-1m \
  --output-dir reviews/task-107/004-distributed-completion/work/phase1-rabitq-100k-l1-control/qdrant-dbpedia \
  --log-file reviews/task-107/004-distributed-completion/artifacts/phase1-rabitq-100k-l1-control/corpus-fetch.log

target/release/ecaz corpus prepare \
  --profile ec_real_100k \
  --parquet reviews/task-107/004-distributed-completion/work/phase1-rabitq-100k-l1-control/qdrant-dbpedia/data \
  --output-dir reviews/task-107/004-distributed-completion/work/phase1-rabitq-100k-l1-control/qdrant-dbpedia/prepared \
  --dim 1536 \
  --source-dataset qdrant-dbpedia-openai3-large-1536-1m \
  --log-file reviews/task-107/004-distributed-completion/artifacts/phase1-rabitq-100k-l1-control/corpus-prepare.log
```

### Load / Build One Index

```bash
scripts/spire-aws/with-ssm-port-forwards.sh \
  reviews/task-107/002-aws-provisioning/artifacts/aws-topology.json \
  reviews/task-107/004-distributed-completion/artifacts/phase1-rabitq-100k-l1-control/load \
  reviews/task-107/004-distributed-completion/artifacts/phase1-rabitq-100k-l1-control/load/aws-topology.tunneled.json \
  -- \
  target/release/ecaz corpus load \
    --host 127.0.0.1 --port 15432 --user ecaz_coord --database postgres \
    --prefix task107_phase1_rabitq_100k_l1 \
    --corpus-file reviews/task-107/004-distributed-completion/work/phase1-rabitq-100k-l1-control/qdrant-dbpedia/prepared/ec_real_100k_corpus.tsv \
    --queries-file reviews/task-107/004-distributed-completion/work/phase1-rabitq-100k-l1-control/qdrant-dbpedia/prepared/ec_real_100k_queries.tsv \
    --manifest-file reviews/task-107/004-distributed-completion/work/phase1-rabitq-100k-l1-control/qdrant-dbpedia/prepared/ec_real_100k_manifest.json \
    --allow-manifest-mismatch \
    --profile ec_spire \
    --dim 1536 \
    --bits 4 \
    --seed 42 \
    --storage-format rabitq \
    --index-name task107_phase1_rabitq_100k_l1_idx \
    --reloption local_store_count=1 \
    --reloption storage_format=rabitq \
    --log-file reviews/task-107/004-distributed-completion/artifacts/phase1-rabitq-100k-l1-control/load.log
```

### Render Single-Node Suite

```bash
jq \
  --arg artifact_dir "reviews/task-107/004-distributed-completion/artifacts/phase1-rabitq-100k-l1-control" \
  --arg prefix "task107_phase1_rabitq_100k_l1" \
  --arg truth_corpus_file "reviews/task-107/004-distributed-completion/work/phase1-rabitq-100k-l1-control/qdrant-dbpedia/prepared/ec_real_100k_corpus.tsv" \
  --arg truth_cache_dir "reviews/task-107/004-distributed-completion/artifacts/phase1-rabitq-100k-l1-control/truth-cache" \
  '.name = "task107-phase1-rabitq-100k-l1-control"
   | .artifact_dir = $artifact_dir
   | .steps |= map(
       .prefix = $prefix
       | if .kind == "recall" then
           .truth_corpus_file = $truth_corpus_file
           | .truth_cache_file = ($truth_cache_dir + "/" + .name + ".json")
         else
           .
         end
     )' \
  reviews/task-107/003-aws-benchmarks/artifacts/rabitq-100k-l4/suite-single-node.json \
  > reviews/task-107/004-distributed-completion/artifacts/phase1-rabitq-100k-l1-control/suite-single-node.json
```

### Run Suite

```bash
scripts/spire-aws/with-ssm-port-forwards.sh \
  reviews/task-107/002-aws-provisioning/artifacts/aws-topology.json \
  reviews/task-107/004-distributed-completion/artifacts/phase1-rabitq-100k-l1-control/bench \
  reviews/task-107/004-distributed-completion/artifacts/phase1-rabitq-100k-l1-control/bench/aws-topology.tunneled.json \
  -- \
  target/release/ecaz bench suite run \
    --host 127.0.0.1 --port 15432 --user ecaz_coord --database postgres \
    --config reviews/task-107/004-distributed-completion/artifacts/phase1-rabitq-100k-l1-control/suite-single-node.json \
    --manifest-output reviews/task-107/004-distributed-completion/artifacts/phase1-rabitq-100k-l1-control/suite-manifest-single-node.json \
    --results-output reviews/task-107/004-distributed-completion/artifacts/phase1-rabitq-100k-l1-control/suite-results-single-node.jsonl \
    --log-file reviews/task-107/004-distributed-completion/artifacts/phase1-rabitq-100k-l1-control/run-suite.log
```

### Storage Evidence

```bash
scripts/spire-aws/with-ssm-port-forwards.sh \
  reviews/task-107/002-aws-provisioning/artifacts/aws-topology.json \
  reviews/task-107/004-distributed-completion/artifacts/phase1-rabitq-100k-l1-control/storage \
  reviews/task-107/004-distributed-completion/artifacts/phase1-rabitq-100k-l1-control/storage/aws-topology.tunneled.json \
  -- \
  target/release/ecaz bench storage \
    --host 127.0.0.1 --port 15432 --user ecaz_coord --database postgres \
    --prefix task107_phase1_rabitq_100k_l1 \
    --log-file reviews/task-107/004-distributed-completion/artifacts/phase1-rabitq-100k-l1-control/storage.log
```

### Cleanup And Stop

```bash
scripts/spire-aws/with-ssm-port-forwards.sh \
  reviews/task-107/002-aws-provisioning/artifacts/aws-topology.json \
  reviews/task-107/004-distributed-completion/artifacts/phase1-rabitq-100k-l1-control/cleanup \
  reviews/task-107/004-distributed-completion/artifacts/phase1-rabitq-100k-l1-control/cleanup/aws-topology.tunneled.json \
  -- \
  target/release/ecaz dev sql \
    --host 127.0.0.1 --port 15432 --user ecaz_coord --database postgres \
    --sql "DROP INDEX IF EXISTS task107_phase1_rabitq_100k_l1_idx; DROP TABLE IF EXISTS task107_phase1_rabitq_100k_l1_queries CASCADE; DROP TABLE IF EXISTS task107_phase1_rabitq_100k_l1_corpus CASCADE; SELECT relname FROM pg_class WHERE relname LIKE 'task107_phase1_rabitq_100k_l1%' ORDER BY relname;" \
    --log-output reviews/task-107/004-distributed-completion/artifacts/phase1-rabitq-100k-l1-control/cleanup-drop.log
```

Then stop the three Task 107 AWS instances and save the final
`describe-instances` output under
`artifacts/phase1-rabitq-100k-l1-control/aws-stop/`.
