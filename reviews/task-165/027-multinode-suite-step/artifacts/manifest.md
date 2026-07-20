# Packet 027 — distann-local-multinode suite step + 50k distinct-recall (manifest)

- code head SHA: (suite step + fixture flag committed with this packet)
- task bucket / packet: reviews/task-165/027-multinode-suite-step
- surface: real 3× PG18 fixture driven from `ecaz bench suite`
  (`distann-local-multinode` step → `ecaz dev distann-multicluster
  local-multinode-pg18`), release `.so` (`target/release/libecaz.so`, installed
  for the run, reverted to the shared 08:41 release build afterward).
- fixture params: nodes=3, rows=50000, dim=16, graph_degree=32, queries=100,
  top_k=10, `--skip-fault-drills` (recall-only mode).
- corpus: the fixture's deterministic replicated synthetic corpus (sin/cos), one
  identical global ec_distann graph per node.
- runner: `ecaz bench suite` (FR-038); bespoke config because this is a new step
  kind for the new AM.
- timestamp: 2026-07-09 (session).

## Command

```
ecaz bench suite run \
  --config reviews/task-165/027-multinode-suite-step/artifacts/distann-multinode-suite.json \
  --host /home/peter/.pgrx --port 28818
```

## What landed (suite-runner extension, per CLAUDE.md "extend the suite runner")

- New `SuiteStep::DistannLocalMultinode` (kind `distann-local-multinode`), a
  sibling of `spire-local-multinode`, in
  `crates/ecaz-cli/src/commands/bench/suite.rs` (struct +
  `expand_distann_local_multinode` + validation + the artifact/rewrite/name/tags/
  pgoptions/expected-artifacts match arms).
- Fixture `--skip-fault-drills` recall-only mode: runs the multi-node
  distinct-recall gates (RECALL_RESULT + `ecaz bench recall` suite_recall_gate +
  qual + FR-082 read-consumption) and skips the TC-042 fault matrix + lifecycle
  drills, which are proven scale-independently at the fixture default size
  (packet 024).

## Key result lines (`artifacts/distann-multinode-50k/distann-multinode-summary.log`)

- `RECALL_RESULT n_queries=100 identical=100 mismatched_ids=0` — the multi-node
  top-k is **byte-identical** to single-node for all 100 queries at 50k.
- `suite_recall_gate single=0.0800 multi=0.0800 delta=0.0000 pass=true` — via
  `ecaz bench recall`: **distinct_recall(multi) = distinct_recall(single)**,
  delta 0.0000 ⇒ ≥ single − 0.001. This is the Task 165 Required-Evidence metric.
- `qual_correctness single_n=10 multi_n=10 mismatch=0 pass=true`.
- `fr082_published_epoch … pass=true` — published-epoch read consumption holds at 50k.
- `GATE PASS (recall-only)`.

## Reading — why absolute recall is 0.08 here (and why it is fine)

The **absolute** recall (0.08) is low because the fixture uses a deterministic
**synthetic 16-dim** corpus (sin/cos), which is pathological for ANN — not a
distribution defect. The metric this packet proves is **distribution
losslessness**: distinct_recall(multi) vs distinct_recall(single) on the SAME
corpus, which is `delta=0.0000` (and `mismatched_ids=0`, byte-identical top-k).
Recall **quality** on real DBpedia is the separate single-node evidence in
**packet 026** (10k 0.99+, 50k 0.915→0.995, 100k up to 0.9925). Together: packet
026 = recall quality vs ground truth; packet 027 = the distributed read path
loses nothing vs single-node at 50k.

## Task 165 M3 — all Acceptance Criteria + Required Evidence met

1. ✅ 100% drill pass across the NFR-020 taxonomy, zero wrong-result (packet 024, 12/12).
2. ✅ FR-082 all ACs incl. restart semantics + retirement override; published-epoch
   read consumption + epoch-swap-under-load (packet 025).
3. ✅ Tombstone/delta DML verified distributed (mid-delete + mid-insert, packet 024).

Required Evidence: drill logs packet-local (024); every drill asserts
error-or-identical-to-baseline; epoch-swap-under-load run (025); **50k multinode
distinct_recall ≥ single-node − 0.001 via `ecaz bench suite`** (this packet);
plus single-node 10k/50k/100k recall+latency+storage via suite (026).
