# Task 205 review request: Algorithm 1 A/B evidence

This packet contains the preregistered PG18 physical owner-traversal A/B
matrix at fixed BW=4, H=100, graph degree 32, head cap 4096, head width 32,
head seed count 32, top-k 10, 200 queries, and 50 iterations after 10
warmups. The control is `owner-traversal-control`; the candidate is
`algorithm1-pushdown`. Both use three physically sharded owners and
`traversal_replica=false`. All six steps succeeded through `ecaz bench suite`.

The staged inputs were present at `/home/peter/dev/ecaz/data/staged-current`:
10k/50k/100k real corpora with 200/1000/1000 queries. The candidate matrix is
under `artifacts/run-candidate-stage2/`; the parent-build baseline is under
`artifacts/run-baseline/`. Both retain packet-local `results.jsonl`, suite
manifests, and per-step summaries. The baseline per-node storage excerpts are
in `artifacts/baseline-storage-topology.log`.

## A/B result

| scale | arm | recall | mean latency ms | transport wait mean ms | request bytes | response bytes | max node graph-side bytes |
|---|---|---:|---:|---:|---:|---:|---:|
| 10k | baseline | 0.9990 | 29.00 | 3.589841 | 683,484 | 389,574 | 25,706,496 |
| 10k | control | 0.9990 | 28.80 | 3.512662 | 683,484 | 389,406 | 25,706,496 |
| 10k | candidate | 0.9990 | 28.80 | 3.579261 | 683,484 | 389,406 | 25,706,496 |
| 50k | baseline | 0.9545 | 38.50 | 4.996836 | 712,808 | 684,268 | 137,379,840 |
| 50k | control | 0.9545 | 38.90 | 5.030952 | 712,808 | 684,064 | 137,379,840 |
| 50k | candidate | 0.9545 | 38.70 | 5.059101 | 712,808 | 684,064 | 137,379,840 |
| 100k | baseline | 0.9275 | 36.60 | 4.587832 | 708,664 | 620,978 | 277,372,928 |
| 100k | control | 0.9275 | 36.90 | 4.593938 | 708,664 | 620,066 | 277,372,928 |
| 100k | candidate | 0.9275 | 36.90 | 4.765928 | 708,664 | 620,066 | 277,372,928 |

Recall is identical across the baseline, control, and candidate at every
scale. The candidate does not reduce request bytes; response bytes are nearly
identical and the candidate transport-wait values are not lower than control
at any scale. Per-node published rows are balanced and all six topology gates
passed with zero non-owned/orphan rows.

## Historical NFR note (superseded)

The 10k/50k/100k max-node graph-side values are 25,706,496 / 137,379,840 /
277,372,928 bytes. The 100k/10k growth is `10.789993627` (50k/10k is
`5.344168260`), exceeding the then-applied NFR-021 `<= 2.0` comparison. That
comparison is withdrawn: the fixed three-node roster makes raw per-node bytes
grow with the corpus even for a correctly sharded owner surface, and the paper
does not state this raw fixed-roster threshold. The calculation remains as
historical evidence, but it is not an inadmissibility verdict. The current
rerun registers NFR-021 context arms and reports normalized bytes-per-owned-
record evidence; see `reviews/task-205/004-l-bounded-rerun/`.

The old A/B is still not a decision-bearing Algorithm 1 measurement because
the pre-rerun implementation recorded zero pushdown activity. The corrected
decision is: **do not advance from this inert measurement; use the bounded-L
rerun for the implementation decision**.

The review request remains open for external review; this is not a promotion
or a claim that the implementation is admissible.
