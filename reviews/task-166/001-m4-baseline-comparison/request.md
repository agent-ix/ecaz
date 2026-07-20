# Task 166 (M4) — packet 001: matched-protocol baseline comparison

Coder progress packet toward the M4 program gate. Produces the **ec_distann vs
IVF vs HNSW vs DiskANN** single-node comparison at 10k / 50k / 100k on the same
real DBpedia corpus / head / protocol, release build, via `ecaz bench suite`.

This is the runnable portion of the M4 four-way gate. The fourth column
(**best-SPIRE anchor**) and the pre-registered promote/iterate/shelve verdict
require the Task-138 distinct-recall metric emitter and the Task-146 anchor
evidence to be merged onto the measuring branch — an operator merge decision
(task 166 says "record merge SHAs in the packet manifest"). Those are NOT yet on
this branch, so this packet delivers the three ecaz-AM columns now and leaves the
SPIRE anchor + verdict pending that merge.

## Config

`artifacts/m4-baseline-suite.json` — profiles `ec_hnsw` / `ec_ivf` / `ec_diskann`
× scales 10k/50k/100k × load/recall/latency/storage, each with the registered
`default_sweep` (hnsw `[40,64,100,128,160,200]`, ivf `[8,16,24,32,48,64]`,
diskann `[64,128,200,400,800]`), k=10, queries_limit=200, bits=4, seed=42. The
ec_distann column is packet 026 (same corpus/host/head).

## Evidence

- `artifacts/results.jsonl` + `artifacts/suite-manifest.json` (canonical).
- `artifacts/{recall,latency,storage,load}-{scale}-{am}.log`.
- Comparison table: see `artifacts/manifest.md` (filled after the run).

## Residual for M4 closeout (needs operator action)

1. Merge `task-138-spire-distinct-recall-metric` + `task-146` anchor branch onto
   the measuring line (record SHAs), then run the `comparator` step for the
   best-SPIRE column on the same protocol.
2. Build the `distann-pipeline` suite step (166 scope) for the per-round-counter
   distann column if the pre-registration requires it beyond the standard sweep.
3. Write the pre-registered verdict into ADR-085 status.
