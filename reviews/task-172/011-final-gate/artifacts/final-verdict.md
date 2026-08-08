# Task 172 final gate verdict

## Disposition

The final matrix is decision-complete for the physical distributed correctness,
engagement, recall, latency, throughput, storage, and build-time questions
defined by Task 172. It supports closing the task as a completed measurement
gate. It does not support claiming that this local three-node fixture is faster
than the single-instance control.

| Criterion | Evidence | Disposition |
| --- | --- | --- |
| Physical placement / NFR-021 | 10k/50k/100k; zero non-owned records, orphan vectors, missing owned records, and coordinator-resident unsharded bytes | Pass |
| Remote expansion and materialization | Two remote owners and two materialization probes at each scale | Pass |
| Recall | Physical/control: 1.0000/1.0000, 0.9750/0.9800, 0.9550/0.9500; intervals overlap at every scale | Recall-neutral in this run |
| Latency / throughput | Full 1/2/4/8/16 sweep at all scales; physical c1 QPS 56.185/50.630/49.379 and c16 QPS 14.412/19.031/18.794 | Measured; no performance promotion |
| Cluster storage | Amplification 1.235600/1.332693/1.351147 at 10k/50k/100k | Pass for physical accounting |
| Build / publish | Physical build grows from 59.7 s to 873.3 s; publish from 74.5 s to 1001.5 s | Measured capacity constraint |
| Full-metrics diagnostic | 10k physical mean latency 17.60 ms versus 17.80 ms benchmark mode, with stage/materialization counters captured | Diagnostic pass |

## Capacity note

The measured 100k cluster graph-side footprint is 830,144,512 bytes. Linear
extrapolation gives about 8.3 GB at 1m and 83.0 GB at 10m, excluding head,
metadata, and operational overhead. The 100k build/publish time also shows
that larger-scale validation needs a host and run budget appropriate to the
workload; these estimates must not be treated as benchmark results.

## Follow-up

Task 219 can now make the Pareto recall decision against this completed gate.
Any performance-promotion claim or larger-scale capacity claim requires a
separate benchmark packet; this task's accepted result is the correctness and
recall-neutrality gate, with measured local distributed performance recorded as
the control comparison.
