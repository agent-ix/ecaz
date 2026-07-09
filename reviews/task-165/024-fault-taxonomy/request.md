# Task 165 — packet 024: NFR-020 fault-taxonomy completion (12-drill TC-042 matrix)

Coder review request. Addresses the packet-021 **P1 "NFR-020 fault taxonomy is
still incomplete"** finding by extending the TC-042 fault matrix from 6 to **12**
drills — closing **all six** of the reviewer's named cases — proven on the real
3× PG18 fixture. **GATE PASS.**

## Summary (six new cases)

- **hop_round_failure_mid_beam**: debug GUC injects a failure at hop round 1 after
  round 0 executed; the partial beam is discarded and the query errors (names
  `round 1`). `pass=true`.
- **remote_statement_timeout**: one owner's conninfo carries `statement_timeout=1ms`;
  its expand is cancelled server-side and the coordinator surfaces the remote
  error — never a partial result. `pass=true`.
- **missing_node_record** (FR-079 case c): the local expander reports an owned
  record absent; the scan errors, never under-returns. `pass=true`.
- **missing_heap_row_co_placement_drift**: heap row deleted on every node, index
  record surviving, over coordinator-owned and remote-owned targets; asserts the
  NFR-020 disjunction (error OR correct-complete vs single-node). `pass=true`.
- **mid_insert_failure** (FR-083 fold): `graph_insert_record` errors after staging,
  before publish; on an isolated table the aborting statement rolls staged pages
  back and a post-fold scan is byte-identical to pre-fold. `pass=true`.
- **mid_delete_lost_tombstone_no_resurrect**: `apply_record_writes` errors after the
  WAL-logged flag flip; the monotonic tombstone stays set (PG does not undo
  index-page writes on abort) so the row is deleted and stays deleted — errors,
  never resurrects. `pass=true`.
- Full matrix + recall + qual + retention + AC-5 + disjoint all green; **GATE PASS**.

## Evidence

- Config/params, command, and key result lines:
  `reviews/task-165/024-fault-taxonomy/artifacts/manifest.md`
- Summary log: `artifacts/distann-multinode-summary.log`
- Full run log: `artifacts/fixture-run.log`

## Design notes the reviewer should check (findings, not assumptions)

Two behaviors were corrected mid-build after the fixture contradicted the initial
assertion — please confirm you agree both are NFR-020-compliant:

1. **co-placement drift.** A single-owner delete is masked by the replicas (the
   coordinator serves its own copy), so genuine drift needs a cluster-wide delete.
   Once cluster-wide, the read path returns a *correct complete* result (skips the
   MVCC-invisible co-placed row consistently on local and remote owners) — one of
   the two NFR-020-compliant outcomes. The `EC_VECTOR_MISSING` error arm (FR-079
   case d) is reachable only under genuine unreadable corruption, not a clean SQL
   DELETE; it stays covered by single-node pg_test TC-040.
2. **mid-delete / lost tombstone.** The tombstone flag is a monotonic WAL-logged
   set, and PG does not undo index-page writes on abort — so a lost tombstone
   write errors but the row is deleted and STAYS deleted (never resurrects). The
   caller sees an error yet the delete holds; worth confirming this is the
   intended contract (a lost ack is safe to retry idempotently).

## Still open (tracked, not in this packet)

NFR-020/TC-042 taxonomy is now complete (12/12). The remaining packet-021 P1s are
the FR-082 published-epoch read wiring and the full `ecaz bench suite`
10k/50k/100k matrix — the substantive M3 closeout gates.
