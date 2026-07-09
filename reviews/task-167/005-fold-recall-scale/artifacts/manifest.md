# Packet 167/005 — M5 fold-recall parity at scale (10k / 50k)

- task bucket / packet: reviews/task-167/005-fold-recall-scale
- surface: single-node ec_distann, release `.so`, PG18 (port 28818), DB
  `distann_t165`, staged real DBpedia (`ec_real_{10k,50k}`).
- method (A/B, per prefix): build the ec_distann graph on the first ~95% of rows
  (10k→9500, 50k→47500), `INSERT` the remaining ~5% (→ D5 delta buffer via
  `aminsert`), `ec_distann_fold_delta_into_graph(idx)` to graph-fold them, then
  measure recall vs brute-force ground truth over the FULL corpus with
  `ecaz bench recall` (sweep `[16,32,64,100,200]`, k=10, 200 queries). The
  **full-rebuild baseline is packet 026** (same corpus/host).

## Purpose

Task 167 (M5) closeout evidence item: quantify the incremental-fold recall vs a
full rebuild, at scale (packet 004 only measured 2k). Directly answers the
packet-004-P2 finding "fold recall materially below rebuild."

## Result — recall@10, fold vs full rebuild

| scale | sweep | **fold** | full rebuild (026) | gap |
| ----- | ----- | -------- | ------------------ | --- |
| 10k | 16  | 0.7950 | 0.9935 | −0.198 |
| 10k | 32  | 0.8250 | 0.9990 | −0.174 |
| 10k | 64  | 0.8470 | 0.9995 | −0.153 |
| 10k | 100 | 0.8570 | 1.0000 | −0.143 |
| 10k | 200 | 0.8780 | 1.0000 | −0.122 |
| 50k | 16  | 0.6260 | 0.9150 | −0.289 |
| 50k | 32  | 0.6520 | 0.9545 | −0.303 |
| 50k | 64  | 0.6700 | 0.9840 | −0.314 |
| 50k | 100 | 0.6810 | 0.9880 | −0.307 |
| 50k | 200 | 0.6910 | 0.9950 | −0.304 |

Folded fraction: 499/10000 and 2499/50000 (~5% at both scales).

## Reading — decision-grade finding

- The incremental fold does **NOT** reach rebuild recall parity, and the gap
  **widens with scale**: ~0.12–0.20 at 10k → ~0.29–0.31 at 50k, even though only
  ~5% of rows were folded incrementally.
- A ~5%-folded corpus losing 0.30 recall means the fold's back-edge reprune
  degrades the *existing* graph's connectivity broadly, not just the newly-folded
  records — the O(N) per-row full-directory rewrite + full-reprune (004-P2) is
  the suspect.
- **Acceptance conclusion for Task 167:** the current D5 delta-buffer +
  operator-`fold` path is a functional interim insert posture but is **not
  recall-viable as a standalone incremental-insert mechanism at scale**; a full
  `REINDEX` (epoch rebuild) is the parity mechanism. This confirms M5 is NOT
  complete against its distinct_recall-parity acceptance criterion — the fold
  needs redesign (bounded, non-destructive back-edge amendment) before M5 can be
  signed off.

Numbers trace to `artifacts/fold-recall-{10k,50k}.log`.
