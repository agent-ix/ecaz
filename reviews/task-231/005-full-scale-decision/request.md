---
task: 231
packet: 005-full-scale-decision
agent: Codex
role: coder
model: GPT-5
date: 2026-08-30
seq: 3
---

# Task 231 full-scale decision preregistration

Status: review-open; **no measurement step has run**. Instrumentation/config
checkpoint: `bf4b78ed2ad462e6c15816fa6544dfd46ee7414c` (building on
`be7264b5a` and `9bb9e0f1b`). GitHub ticket: issue #97.

This request freezes the decision contract and asks for review of the runner
instrumentation before the 10k/50k/100k A/B starts. The checked-in
`crates/ecaz-cli/suites/task231-fixed-stride-10k-50k-100k.json` has SHA-256
`48dbcbf38383d99418e99b6f246149c5fb7b552b696444ed6cd8e9379da1d211`
and audits as 27 steps.

## Frozen decision rule

The primary decision metric is `physical_benchmark_latency.mean_ms` for the
physical `production` arm at concurrency 1 in the **warm 100k BW4/H100**
matrix. Fixed-stride earns **PROMOTE** only when both independent,
counterbalanced matched pairs improve over their own current-heap control by
at least **5.0% and 0.50 ms**. Both bounds are conjunctive per pair; pairs are
not averaged and there is no tie-break. One pass plus one failure is **STOP**.
Failure of either pair is STOP even if cold or wide-beam evidence is positive.

The controlled-shared-buffer-cold arms are reported separately as mechanism
evidence and cannot be called a warm-path win. The single 100k BW16/H25
control-first/fixed-second transfer pair is strictly **report-only** because it
is not counterbalanced: it can neither PROMOTE nor STOP, and it cannot rescue
or reverse the primary matrix's verdict. Its only role is to disclose the
observed layout direction under the historically measured wider-beam regime;
position remains a named confounder in its interpretation.

A failed checksum/release/conformance prerequisite invalidates the affected
run rather than becoming a candidate loss; it must be corrected and rerun.

## Predicted fixed-stride storage arithmetic

The staged corpus is 1,536 dimensions, degree 32, RaBitQ bits=1. RaBitQ's
persisted code length is `ceil(1536 / 8) + 12 = 204` bytes. Therefore:

```text
exact vector                         1536 * 4 =  6,144 B
own search code                                      204 B
neighbor ids                           32 * 8 =    256 B
neighbor codes                       32 * 204 =  6,528 B
node body                                         13,132 B
fixed node header                                    80 B
node_record_bytes                                 13,212 B
node_stride_bytes = align8(record)                13,216 B
page_payload_bytes = 8192 - 24 - 80                8,088 B
extent_blocks = ceil(13216 / 8088)                     2
physical bytes per base/overlay node              16,384 B
```

The prediction is one 8 KiB metadata block plus two blocks per base node.
Each node consumes 16,384 physical bytes for 13,212 record bytes: 3,172 bytes
(19.36% of the allocation) are alignment, page headers, and terminal extent
slack. A visited fixed node touches two PostgreSQL buffers / 16 KiB before any
cache-hit distinction. The run must confirm the exact descriptor values and
the `8192 + 16384 * base_nodes` raw-store relation-size formula. Any unexplained
deviation is a conformance failure; the predicted padding is an explicit
design cost, not a post-hoc surprise.

Every warm and cold fixture already emits `physical_benchmark_storage`,
per-owner `physical_benchmark_storage_node`, and relation-level storage at
10k/50k/100k. Storage is a mandatory reported axis; the primary promotion
metric remains the frozen warm latency rule above.

## DML overlay-growth measurement

The existing 32-replacement + 32-delete routed DML workload now snapshots
every owner's topology immediately before and after mutation. It emits
`physical_benchmark_dml_storage` rows with raw-store before/after/growth bytes,
growth per statement, and a cluster aggregate. Current-heap controls must show
zero raw-store growth. Fixed-stride must show positive growth, and that growth
must be an integral number of 16,384-byte extents; dividing by 16,384 exposes
the complete overlay count, including backlink amendments. These bytes remain
unreclaimable until generation rebuild and will be reported alongside DML
latency rather than hidden by the base storage snapshot.

## Recall basis and neutrality rule

Accepted Task 230 controls on this same lane measured 0.9990 at 10k,
0.9540--0.9545 at 50k, and 0.9295--0.9300 at 100k. The suite's absolute
harness-validity floors are consequently 0.995, 0.950, and 0.925; the old
unattainable 0.980 50k floor is removed.

Fixed-stride serves the same exact vectors and ordered adjacency, so recall is
expected to be neutral. At each scale, candidate-minus-matched-control below
-0.001 is a candidate defect to investigate and is not an acceptable
latency/recall trade. The observed Task 230 control-versus-control spread was
0 at 10k and 0.0005 at 50k/100k, so the frozen 0.001 tolerance is at least
twice the measured nonzero lane spread.

## Integrity, residency, and instrumentation under review

- `precheck-host` captures `version()`, `server_version_num`,
  `shared_buffers`, `SHOW data_checksums` through `current_setting`, extension
  git SHA, and extension build profile. Its structured row has an explicit
  `data_checksums_numeric == 1` suite threshold.
- Every fixed-stride cluster node independently runs `SHOW data_checksums` and
  fails before corpus loading unless it is `on`; the result is now persisted
  as `task231_integrity_preflight`. The existing unanimous release/SHA
  preflight remains mandatory and debug override is absent.
- Controlled-cold steps build and scan an unlogged churn relation at 2.5x
  `shared_buffers`, fail unless it reaches at least 2x, and allow exactly one
  timed iteration with zero warmups.
- Fixed reads now report directory probes, logical extents/bytes touched, and
  PostgreSQL `BufferUsage` shared hits versus reads. The graph diagnostic uses
  fixed-stride batch reads and validates directory/node identity. A fixed
  exact-vector fallback into the row tier fails closed.
- Node-store bytes are included in topology and storage totals. The suite
  parser persists checksum and DML-growth rows in `results.jsonl`.
- BW4/H100 remains the counterbalanced primary regime for continuity with the
  current production control. The added BW16/H25 100k pair is the historically
  measured distributed wide-beam/fewer-rounds shape from Task 179 and directly
  checks whether the layout conclusion transfers when each round requests
  more nodes.

## Validation and requested review

Packet-local receipts and hashes are in `artifacts/manifest.md`. The suite
audit passes 27/27 configured steps; the focused CLI tests pass 2/2.

Please verify that seq-02's wide-pair asymmetry is removed and review-close the
preregistration if DONE. I also request the offered exhaustive review of the
full `be7264b5a` + `9bb9e0f1b` + `bf4b78ed2`
instrumentation/config surface before authorizing the run. The
decision run will not start until this preregistration is review-closed;
Packet 004's allocator rereview is now closed DONE at seq-02.
