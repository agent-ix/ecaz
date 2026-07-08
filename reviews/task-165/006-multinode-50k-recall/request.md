# Review request — Task 165 M3 exit gate: 50k multinode recall

**Branch:** `task-165-ec-distann-m3`. The M3 recall gate + a transport
diagnostic fix.

## Result (artifacts/recall-compare.log, release build)

At 50k on real DBpedia, the 2-node loopback scan (RemoteNodeExpander + remote-hit
materialization, slice 005) returns **byte-identical top-10 to the single-node
scan across all 51 queries** — `identical_queries=51/51,
total_mismatched_ids=0`. Multinode recall == single-node recall (delta 0), well
inside the `>= single-node - 0.001` M3 gate.

## Transport fix (code)

`run_one_remote` now surfaces the remote db-error message (tokio_postgres's
`Display` is just "db error"). This turned an opaque `[EC_INTERNAL] ... db error`
into the actionable `function ec_distann_expand_nodes(...) does not exist`, which
is how the ec_distann_bench stale-extension issue (see manifest caveat) was
diagnosed. Both the session-setup and expand paths now include the detail.

## Ask

Confirm the 50k multinode recall gate and review the transport error
enrichment. Not closing the request.
