# Task 122 Packet 009: Sidecar TQ Stage-2 Suite

This packet measures whether TurboQuant can act as a compressed stage-2
candidate reducer before exact f32 rerank. The tested shape is:

```text
ec_ivf / RaBitQ rerank=off frontier -> sidecar score -> exact f32 over sidecar top-M
```

The matrix covers staged 10k, 50k, and 100k real corpora with recall, latency,
sidecar bytes, sidecar storage, and base index storage.

## Scope

- Fresh isolated `ec_ivf` RaBitQ `rerank=off` loads for 10k/50k/100k.
- Sidecar variants: `f32`, `rabitq8`, `turboquant4`.
- Candidate budgets: `50` and `100`.
- Final exact f32 widths: `10`, `25`, and `50`.
- Nprobe sweep: `32/64`.
- Read modes: `free` and `tid-sorted`.
- Query count: `100`.

## Evidence

- Manifest: `artifacts/manifest.md`
- Base suite config: `artifacts/task122-sidecar-tq-stage2-suite.json`
- Width suite config: `artifacts/task122-sidecar-tq-stage2-width-suite.json`
- Base suite results: `artifacts/suite/results.jsonl`
- Width suite results: `artifacts/width-suite/results.jsonl`
- Compact summaries:
  - `artifacts/sidecar-summary.txt`
  - `artifacts/sidecar-width-summary.txt`
- Status:
  - base suite: `12` succeeded, `0` failed
  - width suite: `6` succeeded, `0` failed

## Results

At `candidate_k=100`, TQ stage-2 with exact f32 over the top `25` matched the
full-f32 sidecar baseline recall across all tested scales and nprobe points,
while reducing exact f32 work from 100 candidates to 25 candidates.

```text
scale  nprobe  f32 recall/p95      TQ->f32@25 recall/p95
10k    32      1.0000 / 2.389 ms   1.0000 / 1.855 ms
10k    64      1.0000 / 2.898 ms   1.0000 / 2.176 ms
50k    32      0.9960 / 4.822 ms   0.9960 / 3.885 ms
50k    64      1.0000 / 7.835 ms   1.0000 / 6.819 ms
100k   32      0.9730 / 8.713 ms   0.9730 / 7.953 ms
100k   64      1.0000 / 13.815 ms  1.0000 / 13.517 ms
```

TQ was close to RaBitQ8 on latency and used half the sidecar bytes:

```text
scale  nprobe  RaBitQ8->f32@25 p95  TQ->f32@25 p95  bytes touched
10k    32      1.923 ms             1.855 ms         151.17 KiB vs 75.39 KiB
50k    64      6.876 ms             6.819 ms         151.17 KiB vs 75.39 KiB
100k   64      13.522 ms            13.517 ms        151.17 KiB vs 75.39 KiB
```

Final exact width `10` was too narrow for TQ stage-2 at larger scales:

```text
50k candidate_k=100,nprobe=64:  TQ->f32@10 recall 0.9420
100k candidate_k=100,nprobe=64: TQ->f32@10 recall 0.9570
```

Persisted sidecar size at 100k:

```text
f32:         585.94 MiB
rabitq8:     147.63 MiB
turboquant4:  73.62 MiB
```

## Decision

Task 122 should not be closed as a TQ deferral. The measurements show a viable
stage-2 path:

```text
RaBitQ frontier -> TQ compressed reducer -> exact f32 width 25
```

The right closeout status is **keep experimental / promote follow-up**, because
the winning shape is currently measured in the CLI sidecar harness, not landed
as an in-engine AM pipeline. The follow-up should implement the stage-2 path
inside the product scan/rerank pipeline and carry forward the packet 009
matrix as the baseline gate.

This packet also confirms the negative bound: TQ final width `10` is too narrow
for 50k/100k, and direct SPIRE TQ-vs-RaBitQ evidence in packets 006/007 did not
show a separate SPIRE product win.
