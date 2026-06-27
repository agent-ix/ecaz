# Task 122 Packet 006: SPIRE Recall Width Sweep

This packet records a release-backend exploratory sweep for Task 122's TQ
rerank-pipeline work. It tests whether TQ can reduce the f32 rerank/candidate
width relative to RaBitQ in the current SPIRE scan shape before spending more
time on latency matrices.

## Scope

- Fresh PG18 release loads for 10k, 50k, and 100k staged real corpora.
- `ec_spire` with `turboquant` and `rabitq`, bits `4`, `k=10`, `queries_limit=100`.
- Widths `25/50/100/200`.
- Nprobe sweep `24/48/96/192`.
- TQ runs used `ec_spire.pre_materialization_prune=on` and matched
  `ec_spire.max_candidate_rows` to the tested width. RaBitQ runs matched
  `ec_spire.max_candidate_rows` to the tested width.

## Evidence

- Manifest: `artifacts/manifest.md`
- Suite config: `artifacts/task122-spire-recall-width-sweep.json`
- Suite results: `artifacts/suite/results.jsonl`
- Suite manifest: `artifacts/suite/suite-manifest.json`
- Audit log: `artifacts/suite-audit.log`
- Run log: `artifacts/suite-run.log`
- Backend/GUC check: `artifacts/guc-check.log`

The suite audit passed (`30` steps), and the final suite manifest records all
`30` steps as succeeded on a release backend.

## Results

Rerank/candidate width did not change recall for either format at any tested
scale. The repeated recall/NDCG pattern was:

```text
10k:  all widths, both formats, all nprobe values => recall@10 1.0000 / ndcg@10 1.0000

50k:
  nprobe 24  => recall@10 0.9450 / ndcg@10 0.9969
  nprobe 48  => recall@10 0.9760 / ndcg@10 0.9993
  nprobe 96  => recall@10 0.9940 / ndcg@10 0.9999
  nprobe 192 => recall@10 1.0000 / ndcg@10 1.0000

100k:
  nprobe 24  => recall@10 0.8940 / ndcg@10 0.9893
  nprobe 48  => recall@10 0.9430 / ndcg@10 0.9948
  nprobe 96  => recall@10 0.9860 / ndcg@10 0.9981
  nprobe 192 => recall@10 0.9980 / ndcg@10 0.9997
```

TQ and RaBitQ were recall-equivalent across the grid. Increasing
`rerank_width` / `ec_spire.max_candidate_rows` from `25` to `200` did not
recover additional neighbors; only increasing `nprobe` moved recall.

## Interpretation

This narrows the next optimization work:

- The current SPIRE path is not showing a quality reason to widen past `25`
  candidates for these 10k/50k/100k gates.
- A TQ-as-candidate-reducer lane does not yet show a recall advantage over
  RaBitQ in this suite shape.
- The next useful measurement should be a latency/storage matrix at the
  minimal recall-matched points, especially 50k `nprobe=192,width=25` and
  100k `nprobe=192,width=25`, plus any lower-nprobe point intentionally
  accepted as a quality tradeoff.

This is not a Task 122 closeout request. It is a checkpoint for the seven-phase
optimization exploration.
