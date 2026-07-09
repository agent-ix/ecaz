# Task 167 (M5) — packet 005: fold-recall parity at scale (10k/50k)

Coder measurement packet. Closeout evidence for the packet-004-P2 finding "fold
recall materially below rebuild," measured at scale (10k/50k) beyond the 2k in
packet 004.

## Finding

Incremental graph-fold recall is far below full rebuild and the gap WIDENS with
scale: recall@10 (tk=200) fold 0.878 vs full 1.0 at 10k (−0.122), and fold 0.691
vs full 0.995 at 50k (−0.304) — with only ~5% of rows folded incrementally. See
`artifacts/manifest.md` + `artifacts/fold-recall-{10k,50k}.log`.

## Conclusion

The D5 delta-buffer + operator-`fold` path is a functional interim insert
posture but is NOT recall-viable as a standalone incremental-insert mechanism at
scale; REINDEX (epoch rebuild) is the parity mechanism. Task 167 (M5) is NOT
complete against its distinct_recall-parity acceptance criterion — the fold needs
a bounded, non-destructive back-edge amendment redesign before signoff. This
packet makes that verdict evidence-backed rather than asserted.
