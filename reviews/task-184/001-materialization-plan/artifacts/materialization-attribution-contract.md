# Task 184 materialization attribution contract

## Fixture and reset boundary

- Scale: `ec_real_100k` from `/home/peter/dev/ecaz/data/staged-current`.
- Topology: three local PG18 instances, exact/disjoint physical owners.
- Policy: Task 182 production trained head, cap 4,096, 32 seeds, BW4/H100,
  RaBitQ traversal, exact final rank.
- Query identity: file SHA-256
  `a7cbec6fc44f6c148234538f61339d00d2f10646febc8f667dcbe75d9cf41782`;
  evaluation slice rows 1--200 with the same digest; training slice rows
  201--400, SHA-256
  `30f11df03f6e988adfe531e2bf54b75b8515fa207fee1212dd0774acffec7471`.
- Latency: 10 warmups, then reset every Task 183/184 stage/work counter, then
  50 timed queries at concurrency 1 on the same connection.
- Isolation: one fresh index/table generation, compact artifacts, fault drills
  skipped only for the measurement fixture.

## Time counters

All counters are compiled only with `distann-head-attribution-benchmark`.
Every duration row reports samples, total nanoseconds, and mean per timed scan.

### Existing independent stages

`query_prep`, `head_score`, `seed_select`, `traversal_total`,
`remote_materialize`, `output_merge`, and `custom_scan_total` remain for
continuity. `local_expand` and `remote_expand` remain documented nested
traversal work.

### New coordinator materialization partition

| Stage | Boundary | Nesting |
| --- | --- | --- |
| `materialize_prepare` | schema/projection conversion, remote-ID partitioning, owner request construction | independent child of `remote_materialize` |
| `materialize_connection_ready` | pooled connection and prepared outer-statement readiness | independent child |
| `materialize_request_wait` | concurrent owner queries from dispatch through raw PostgreSQL rows available | independent child; contains owner/network/server work |
| `materialize_coordinator_decode` | raw row extraction and owned Rust payload construction | independent child |
| `materialize_map_insert` | order/count/failure validation and result-map insertion | independent child |
| `materialize_output_associate` | ranked hit association and installation into CustomScan outputs | outside `remote_materialize`, separately measured |

The first five plus uncategorized bookkeeping must reconcile to
`remote_materialize`. Output association remains separately reconcilable to
`output_merge`/`custom_scan_total`.

### Nested owner/request telemetry

| Metric | Meaning |
| --- | --- |
| `materialize_request_roundtrip_work` | sum of individual per-owner client request durations |
| `materialize_owner_endpoint_work` | sum of server-reported endpoint durations |
| `materialize_owner_endpoint_critical` | maximum owner endpoint duration in the concurrent batch |
| `materialize_owner_open_validate_work` | generation open, schema/fingerprint/projection validation |
| `materialize_owner_node_lookup_work` | ownership validation and graph-directory locator resolution |
| `materialize_owner_payload_sql_work` | row-tier lookup, binary send calls, and endpoint-side payload extraction |

`materialize_request_wait - materialize_owner_endpoint_critical` is the
critical-path server-return/client residual. The sum of individual round trips
minus `materialize_owner_endpoint_work` is the corresponding owner-work
residual. Neither is mislabeled as pure network time: it includes PostgreSQL
result encoding, transport, tokio-postgres protocol handling, and scheduling.

## Work counters

| Counter | Increment boundary |
| --- | --- |
| `ranked_candidates` | hits returned by global ranking |
| `remote_candidates_requested` | vec IDs submitted to remote owners |
| `remote_owners_requested` | nonempty owner requests |
| `remote_rows_returned` | rows decoded from owners |
| `remote_tombstones` | returned tombstoned rows |
| `payload_columns_requested` | projection columns summed across remote rows |
| `payload_bytes_returned` | null flags plus binary payload bytes returned |
| `remote_payloads_installed` | live remote payloads installed in scan output state |
| `output_rows_associated` | ranked local plus live remote output rows built |
| `executor_rows_consumed` | CustomScan rows yielded to the executor |
| `executor_remote_rows_consumed` | remote payload rows yielded to the executor |
| `executor_local_rows_consumed` | local/frozen rows yielded to the executor |

For unfiltered `LIMIT 10`, `executor_rows_consumed` is expected to be 10 per
successful query. Under quals it includes rows evaluated and rejected above
the access callback, which is exactly the demand a lazy materializer must
satisfy. The client result-row count is reported separately by the benchmark
runner so filtered rows are derivable as consumed minus returned.

## Required structured evidence

The suite parser must emit compact `physical_benchmark_stage` rows for time
counters and `physical_benchmark_materialization_work` rows for work counters.
Packet 002 retains its suite config, manifest, results JSONL, report, status,
and compact cited logs. Corpus TSVs, truth caches, node logs, and run-directory
exhaust remain uncommitted.

## Candidate trigger

- Eager-work branch: requested/installed remote rows materially exceed remote
  rows consumed; choose one bounded incremental design.
- Owner branch: owner open/lookup/payload SQL dominates the critical path;
  choose one reuse/lookup change.
- Coordinator branch: decode/map/association dominates; choose one ownership
  or ordered-layout change.
- Residual branch: request residual dominates without a bounded in-scope
  remedy; STOP or route protocol work to Task 187/190.

No threshold is invented in advance. Candidate usefulness is judged by the
measured end-to-end ceiling and complete relative Pareto evidence.
