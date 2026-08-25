# Pre-screen snapshot-lifetime failure

- Timestamp: 2026-08-24 01:09:34 PDT
- Runner / installed extension SHA: `b54f321a579ccdac1535aedc4e3387f78811b0af`
- Suite step: `current-head-bw8-production-100k`
- Result: invalid pre-measurement run; no benchmark arm completed and no gate value was produced.
- Last valid fixture evidence: all three owners reached `Published`, with 100,000 total owned rows, `non_owned=0`, and `orphans=0`; the serving smoke returned 10 rows.
- Failure: node 1 aborted the first benchmark-table ANN query on PostgreSQL's `subtrans.c:169` assertion, `TransactionIdFollowsOrEquals(xid, TransactionXmin)`. The extension stack entered `generation_read::lookup_graph_nodes` through `GenerationExpander::expand_nodes_masked`.
- Root cause: the traversal stored a raw pointer to a newly registered snapshot, then dropped the owning `RegisteredSnapshotGuard` at the end of the hop. A later hop could read freed snapshot storage.
- Correction: cherry-picked existing upstream commit `15f7fcf5fe6409578bbffb62b9ac1e48d346e341` as Task 226 commit `c85196ce841c1cbcea187dbefb3c10430fb611be`. It returns no replacement snapshot on an ordinary successful lookup and retains any retry-refreshed guard for the traversal lifetime.
- Disposition: stopped fixture removed after this compact diagnosis; raw PostgreSQL and runner exhaust are not review evidence. The preregistered workload and gate remain unchanged for a fresh rerun.
