# Cell Checkpoint: phase2-rabitq-1m-l1

Status: setup failed after successful coordinator load/build; resume required.

## Intent

Run the Task 107 Phase 2 distributed RaBitQ 1m cell on the AWS topology with one coordinator and two remotes. This is not a single-node baseline and not a Task 106 rerun.

## Scope

- Prefix: `task107_phase2_rabitq_1m_l1`.
- Storage format: `rabitq`.
- Bits: `4`.
- Store count: `local_store_count=1` on coordinator and remotes.
- Topology: coordinator `i-0b4386fa5017f1363`; remotes `i-07bcc98c3d5d027ee` and `i-00c2f2aca9dbdd6bd`.
- Artifact directory: `reviews/task-107/004-distributed-completion/artifacts/phase2-rabitq-1m-l1/direct-ssm-distributed/`.

## Execution Policy

Run this cell by AWS SSM, one cell at a time. Benchmark sweeps run through `ecaz bench suite` using `suite-node.json`.

## Setup Command

- SSM setup payload: `setup-ssm-parameters.json`.
- Intended setup output prefix: `s3://ecaz-spire-aws-20260614203301860100000009/task107/004/phase2-rabitq-1m-l1/direct-ssm-distributed/`.

## Setup Attempt Result

- SSM command id: `8aabd685-0484-4668-954e-4e51bd26d1a6`.
- SSM result: `Status=Failed`, `ResponseCode=2`, elapsed `PT1H1M28.808S`.
- Coordinator load/build completed before the failure:
  - copied 990000 corpus rows in 317.43s;
  - encoded corpus in 475.40s;
  - copied 10000 queries in 3.44s;
  - built `task107_phase2_rabitq_1m_l1_idx` in 2678.33s;
  - completed coordinator prefix load/build in 3602.41s.
- Failure: remote corpus export reported
  `row_count_mismatch node=2 rows=0 assignments=504734`.
- Root cause: the inline resume/export script used `psql -At` without forcing
  tab-separated output, then extracted row ids with `cut -f12`. The assignment
  export was pipe-delimited, so the row-id extraction did not produce stable
  corpus ids.
- Evidence:
  - `setup/ssm-command-invocation.final.json`
  - `setup/coordinator-load.log`
  - `setup/coordinator-inspect.log`
  - `setup/coordinator-drop.log`
- Resume plan: keep the completed coordinator index in place, rerun only the
  remote corpus export using tab-separated assignment output or `cut -d '|'
  -f12`, then load remote node 2 and node 3 sequentially before running the
  suite.
- Resume export payload:
  - script: `resume-export.sh`;
  - SSM parameters: `resume-export-ssm-parameters.json`.

## Resume Export Result

- SSM command id: `b989c56a-c4e1-4389-8b0a-9b40f9caefd5`.
- SSM result: `Status=Success`, `ResponseCode=0`, elapsed `PT1M38.141S`.
- Resume reused the completed coordinator index and exported two remote corpus
  shards:
  - node 2: 504734 rows;
  - node 3: 485266 rows.
- Evidence:
  - `resume-export/ssm-command-invocation.final.json`
  - `resume-export/`
  - `distributed-representative/distributed-placement-plan.json`
  - `distributed-representative/remotes.jsonl`
  - `distributed-representative/node-*/coordinator-base-assignments.stderr.log`
  - `distributed-representative/node-*/upload-remote-corpus.log`

## Remote Node 2 Load Result

- SSM command id: `595ba388-99da-4802-b813-3cbbff2fc927`.
- SSM result: `Status=Success`, `ResponseCode=0`, elapsed `PT24M35.211S`.
- Remote prefix: `task107_phase2_rabitq_1m_l1_node_2`.
- Remote index: `task107_phase2_rabitq_1m_l1_remote_idx`.
- Loaded 504734 corpus rows with `bits=4`, `storage_format=rabitq`, and
  `local_store_count=1`.
- Load timings:
  - copy: 162.76s;
  - encode: 211.15s;
  - index build: 1003.53s;
  - total: 1442.10s.
- Inspect evidence reports the remote SPiRE index at 402.5 MiB with
  `{local_store_count=1,storage_format=rabitq}`.
- Evidence:
  - `remote-load-node-2/send-command.json`
  - `remote-load-node-2/ssm-command-invocation.final-summary.json`
  - `remote-load-node-2/load.log`
  - `remote-load-node-2/inspect.log`
  - `remote-load-node-2/drop.log`
  - `remote-load-node-2/corpus-row-count.log`

## Remote Node 3 Load Result

- SSM command id: `271e027f-a9e6-49b5-9de3-bf594ebd219d`.
- SSM result: `Status=Success`, `ResponseCode=0`, elapsed `PT23M17.992S`.
- Remote prefix: `task107_phase2_rabitq_1m_l1_node_3`.
- Remote index: `task107_phase2_rabitq_1m_l1_remote_idx`.
- Loaded 485266 corpus rows with `bits=4`, `storage_format=rabitq`, and
  `local_store_count=1`.
- Load timings:
  - copy: 155.56s;
  - encode: 205.22s;
  - index build: 945.39s;
  - total: 1368.81s.
- Inspect evidence reports the remote SPiRE index at 387.1 MiB with
  `{local_store_count=1,storage_format=rabitq}`.
- Evidence:
  - `remote-load-node-3/send-command.json`
  - `remote-load-node-3/ssm-command-invocation.final-summary.json`
  - `remote-load-node-3/load.log`
  - `remote-load-node-3/inspect.log`
  - `remote-load-node-3/drop.log`
  - `remote-load-node-3/corpus-row-count.log`

## Remote Node 2 Materialization Result

- SSM command id: `67f905e2-5c8a-4f57-841f-772c8ad1e040`.
- SSM result: `Status=Success`, `ResponseCode=0`, elapsed `PT24.392S`.
- Materialized 504734 coordinator leaf-base assignments into
  `task107_phase2_rabitq_1m_l1_remote_idx`.
- Materialization output: `active_epoch=1`, `leaf_count=498`,
  `assignment_count=504734`, `status=materialized`.
- Leaf parity evidence:
  - coordinator required leaves: 498;
  - remote observed leaves: 498;
  - missing or mismatched leaves: 0.
- Endpoint identity: `endpoint_status=ready`, `tuple_transport_status=ready`,
  `remote_index_identity_hex=a7b56a9cf0d817f9`.
- Evidence:
  - `remote-materialize-node-2/send-command.json`
  - `remote-materialize-node-2/ssm-command-invocation.final-summary.json`
  - `remote-materialize-node-2/remote-materialize.log`
  - `remote-materialize-node-2/coordinator-required-leaves.txt`
  - `remote-materialize-node-2/remote-observed-leaves.txt`
  - `remote-materialize-node-2/missing-or-mismatched-leaves.txt`
  - `remote-materialize-node-2/identity.json`
