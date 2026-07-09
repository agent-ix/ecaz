# Task 165 — packet 024: NFR-020 fault-taxonomy extension (remote timeout + co-placement drift)

Coder review request. Addresses the packet-021 **P1 "NFR-020 fault taxonomy is
still incomplete"** finding by extending the TC-042 fault matrix from 6 to 8
drills and proving the two new cases on the real 3× PG18 fixture.

## Summary

- **remote_statement_timeout** (new, fail-closed): one owner's conninfo carries
  `statement_timeout=1ms`; its expand is cancelled server-side and the
  coordinator surfaces the remote error — never a partial result. `pass=true`.
- **missing_heap_row_co_placement_drift** (new): the record's heap row is deleted
  on **every** node while the index record survives (cluster-wide dangling
  record / missing co-placed vector), across a coordinator-owned **and** a
  remote-owned target. Asserts the NFR-020 disjunction — error **or**
  correct-complete (equal to single-node over the same deleted corpus, target
  excluded). Both arms observed `correct_complete`. `pass=true`.
- Full matrix + recall + qual + retention + AC-5 + disjoint all still green;
  **GATE PASS** at head `f5e40831`.

## Evidence

- Config/params, command, and key result lines:
  `reviews/task-165/024-fault-taxonomy/artifacts/manifest.md`
- Summary log: `artifacts/distann-multinode-summary.log`
- Full run log: `artifacts/fixture-run.log`

## Design note the reviewer should check

The drift drill was reshaped based on fixture evidence, not assumption: a
single-owner delete is masked by the replicas (the coordinator serves its own
copy), so genuine drift requires a cluster-wide delete. Once cluster-wide, the
read path returns a *correct complete* result (skips the MVCC-invisible
co-placed row consistently on local and remote owners), which is one of the two
NFR-020-compliant outcomes. The `EC_VECTOR_MISSING` error arm (FR-079 case d)
is reachable only under genuine unreadable corruption, not a clean SQL DELETE;
it stays covered by single-node pg_test TC-040. Please confirm you agree this is
compliant rather than a masked fail-open.

## Still open (tracked, not in this packet)

`hop_round_failure_mid_beam`, `missing_node_record`, `mid-insert failure`
(FR-083) remain NFR-020 gaps; the FR-082 published-epoch read wiring and the
full `ecaz bench suite` 10k/50k/100k matrix remain the other two packet-021 P1s.
