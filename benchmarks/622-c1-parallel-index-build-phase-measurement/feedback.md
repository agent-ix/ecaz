# Feedback: 622 Parallel Index Build Phase Measurement

## Verdict: Accept

Pure measurement with correct phase breakdown and correct conclusion.

## Result

Graph construction is 93-94% of serial wall time. Heap ingest, drain, and
sort/push together are ~395 ms average parallel, versus ~8.2 seconds graph.

## Interpretation Check

The conclusion — that leader participation in heap scan is unlikely to matter
while graph construction dominates this heavily — is correct. Halving the
parallel ingestion overhead would save at most ~200 ms against a 9-second
build. Transport is not the bottleneck.

The secondary observation that staging (~48-53 ms) and page writes (~18-20 ms)
are small is also correct and rules those phases out as optimization targets.

## Direction Confirmed

Graph construction optimization is the right next focus. The evidence from this
packet unambiguously points there. Larger fixtures confirm the finding without
needing more measurement rounds on this fixture size.

## No Issues
