---
task: 224
packet: 002-locality-attribution
agent: Codex
role: coder
model: gpt-5
date: 2026-08-25
seq: 01
---

# Task 224 locality attribution and MAT-26 GO request

This packet requests outside re-review of the Task 224 attribution
implementation, 100k measurement, and single-candidate decision after
addressing reviewer seq-01. The reviewer accepted packet 001 as realized and
accepted the substantive MAT-26-only decision, but held packet 003 on evidence
and reporting corrections.

The implementation adds default-off, feature-only owner locality telemetry. It
measures TID/block dispersion, buffer observations, external TOAST and byte
counts, exact PostgreSQL binary-send time, the enclosing SPI payload span, and
post-SPI response construction. It also adds four suite-addressable projection
shapes: id-only, narrow scalar, vector-bearing, and a forced external-TOAST
payload with a matching predicate. The normal extension excludes the profiler
and retains the production payload SQL; focused featureless and feature tests
pin both sides.

The suite runner gained an explicit `skip_routed_delete_vacuum_drill` option so
later projection-only steps can reuse an exact generation without the standard
destructive DML drill changing its row count. This is opt-in, requires physical
benchmark mode, and is set on all four attribution steps. The final live suite
created one 100k fixture outside the repository, reused its exact epoch
fingerprint for all four shapes, and succeeded at the final release SHA. The
fixture was removed after evidence capture.

The preregistered result is a **MAT-26 GO, MAT-25 no-advance**:

- vector-bearing binary send is 6.967996 ms/scan against 28.20 ms warm mean,
  a 24.709206% summed-owner ceiling; the endpoint critical path bounds any
  serial saving to at most 5.148990 ms/scan / 18.258830%, and the bucket still
  passes both the 1 ms and 5% gates;
- toasted SPI-minus-send is a conservative MAT-25 ceiling of 1.617951 ms/scan
  against 46.10 ms, so it passes only the absolute gate at 3.509655%; and
- the plan requires advancing exactly one candidate when both pass, so MAT-26
  wins the percentage tie-break. The result remains the same if toasted work is
  normalized by end-user query rather than qualified custom-scan execution.

The physical result is clearer than the timer tie-break: 6.785 requested TIDs
land on 6.770 distinct heap blocks while a block sort moves 72% of rows. Heap
locality offers essentially nothing to coalesce, so MAT-25 is retired. All
reported TOAST buffer reads occur inside the sender and belong to MAT-26 under
its registered detoast/binary-send definition.

The four arm labels are now explicit. Id-only is the shipped control; narrow
scalar is shipped-capable; vector-bearing is a shipped-capable exploratory
stress projection; and the forced-external toasted arm is synthetic
exploratory stress. The plan permits an exploratory result to authorize only
the isolated screen. It does not authorize default-on production behavior.

The packet contains no uninstrumented production-SQL warm denominator. Every
arm uses the feature-only profiled wrapper, whose cost can inflate the warm
mean and MAT-25 residual but cannot inflate the exact `SendFunctionCall`
numerator. The GO is therefore stable, but none of these warm means may become
packet-003 baselines. That A/B must keep both arms in the same instrumentation
state, preferably unprofiled production SQL.

The measurement authorizes only packet 003: one isolated block-batched
detoast/binary-send candidate and same-generation 100k A/B. If that candidate
is not useful, Task 224 STOPs without packet 004; if useful, it must then
receive independent 10k/50k/100k recall, latency, storage, build, and DML
evidence.

## Response to reviewer seq-01

1. Restored all six build/test log SHA-256 values requested by the reviewer.
2. Commit `a96bfdc29` makes skipped concurrency and routed-delete/vacuum drills
   remain `pass=skipped reason=...` in the durable summary. The suite parser now
   emits a structured skipped outcome with no numeric pass; the focused test is
   packet-local and green. The historical raw run is retained unchanged and
   its false-positive drill row is explicitly withdrawn in the manifest.
3. Recorded that no uninstrumented production control exists and prohibited
   reuse of these profiled warm means as candidate baselines.
4. Resolved SQL `LIMIT 10` versus the separately swept
   `ec_distann.top_k=32` GUC in the manifest; `client_result_rows=10` confirms
   the SQL limit.
5. Labeled every projection arm shipped, shipped-capable, or exploratory and
   bounded what an exploratory arm may authorize.
6. Printed the 18.258830% critical-path ceiling everywhere beside the
   24.709206% summed-owner work share.
7. Recorded that PostgreSQL performs TOAST access inside the type sender and
   that MAT-26 owns those touches by definition.
8. Recorded that zero read nanoseconds reflect disabled `track_io_timing`, not
   free reads.

Please verify specifically:

1. the feature gate and production SQL exclusion;
2. timer nesting and the `payload_spi - binary_send` conservative MAT-25 bound;
3. the exact-generation/projection-shape provenance and counter parsing;
4. the `skip_routed_delete_vacuum_drill` reuse invariant; and
5. the gate arithmetic and single-candidate tie-break.

Coder recommendation: **mark seq-01 addressed, ACCEPT packet 002, authorize
MAT-26 only for packet 003, and retire MAT-25.**
