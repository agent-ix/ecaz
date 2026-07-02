# Projection Failure Diagnosis

Source log:
`reviews/task-123/011-multi-instance-100k-timeline-rerun/artifacts/n1024-b2-200q-source/coord-postgres.log`

The failed `id,source` projection is attributable to the production remote tuple
payload guard. The committed log records repeated decode warnings immediately
before the executor hard failure:

```text
remote typed tuple payload payload bytes 12316 exceeds ec_spire.max_remote_payload_bytes_per_row 1024
```

The final visible status is:

```text
ERROR: EcSpireDistributedScan production executor blocked: status remote_heap_resolution_failed, next_blocker remote_heap_resolution
STATEMENT: SELECT id, source FROM ec_real_100k_corpus ORDER BY embedding <#> $1::real[] LIMIT $2
```

Interpretation:

- the realistic projection did not produce timing rows;
- the current guard is too low for the `source` vector payload;
- the next communications measurement must either use a narrower realistic
  payload column or raise `ec_spire.max_remote_payload_bytes_per_row` with
  packet-local evidence;
- this packet does not resolve the measurement blocker.
