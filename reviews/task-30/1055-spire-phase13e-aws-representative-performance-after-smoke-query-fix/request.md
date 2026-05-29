# Review Request: AWS Representative Performance After Smoke Query Fix

## Summary

This packet records the first Graviton representative pass after the smoke query selection fix.

The run used the established AWS lane: `us-west-2`, `us-west-2a`, one coordinator plus three remotes, all `m7g.large`. It loaded the real representative corpus (`ec_real_100k`: 100,000 corpus rows, 1,000 query rows), built coordinator and remote indexes, materialized remote shards, published static remote placements, and executed real remote heap reads through `EcSpireDistributedScan`.

The run did not complete the whole representative priority plus pooling suite. It failed during the priority suite `k=100` recall step with:

`ERROR: ec_spire production executor cannot merge remote heap candidates while node_id 3 is in state CandidateReceiveFailed with status remote_candidate_receive_failed`

Per the AWS gate rule, this should not be debugged further in AWS. The next step is local reproduction/debugging of the generic remote candidate receive failure.

## Confirmed Functionality

- Real remote placements were present for nodes 2, 3, and 4.
- Remote leaf materialization completed:
  - node 2: 34,589 rows
  - node 3: 32,930 rows
  - node 4: 32,481 rows
- Smoke executed `EcSpireDistributedScan` with `remote_fanout: 3`.
- Production read profile returned `status=ready`, `result_source=remote_heap_candidates`, `final_heap_fetch_status=remote_ready`, and 10 returned rows.
- Smoke benchmark ran on five real queries with remote production-read profile ready at nprobe 8, 16, and 32.

## Real-Corpus Measurements Collected

Smoke benchmark, 5 real queries:

- nprobe 8: recall@10 `0.7600`, latency p50 `86.792 ms`
- nprobe 16: recall@10 `0.8200`, latency p50 `91.230 ms`
- nprobe 32: recall@10 `0.9200`, latency p50 `93.363 ms`

Priority suite `k=10`, 1,000 real queries:

- nprobe 8: recall@10 `0.7868`, mean q-time `90.45 ms`
- nprobe 16: recall@10 `0.8626`, mean q-time `94.16 ms`
- nprobe 24: recall@10 `0.8962`, mean q-time `94.87 ms`
- nprobe 32: recall@10 `0.9187`, mean q-time `96.58 ms`

## Failure Classification

This was not an AWS provisioning failure and not the previous smoke query ID issue.

The failure occurred after remote placement/read functionality was already live and after the full `k=10` recall sweep passed. It is a generic SPIRE remote execution failure surfaced by the production executor during the `k=100` recall step:

- failing step: `13a3a-recall-k100`
- failing node: `node_id 3`
- state/status: `CandidateReceiveFailed` / `remote_candidate_receive_failed`
- harness exit: status 2
- teardown: completed with Terraform state clean
- independent post-run EC2 inventory: no pending/running/stopping instances printed

## Harness Observation

The current recall harness repeatedly fetches `SELECT id, source FROM ec_spire_aws_repr_1m_corpus ORDER BY id` over the SSM operator tunnel before each recall step. Packet-local `pg_stat_activity` snapshots show this as long-running `ClientWrite` waits. This is benchmark harness overhead, separate from the SPIRE remote query failure.

## Evidence

See `artifacts/manifest.md`.

## Review Focus

- Confirm the functional proof is sufficient: remote placement, remote materialization, remote fanout, and remote heap result return all occurred on real corpus.
- Confirm the `k=100` failure should be reproduced locally before any further AWS run.
- Confirm the benchmark harness needs a cached or server-side recall fixture before more AWS measurement loops.
