# Feedback: 609 PG18 Contributor Drain — Scale-Invariant Diagnostic

## Verdict: Accept

Pure measurement packet with no code changes. Analysis is correct.

## Findings

The `NoVisibleOwner: 4` result at 5k/limit=100 is identical in shape to the
512-row/limit=16 case from packet 607. The two root causes named are right:

- **Graph diameter**: 4D deterministic fixture produces trivially dense graphs.
  Seed diversification from `ee9b405` does not produce diverging neighborhoods
  because all workers start from the same entry point and the effective diameter
  is 2-3 hops.
- **Timing**: The emitter exhausts its candidate set (limit+1 = 101 nodes) before
  any contributor can publish. At this scale the emitter traversal is
  microseconds.

The conclusion — that the handoff model is structurally sound but untestable on
a 4D deterministic fixture — is correct. The `best_hidden_local_only_blocker_locked`
probe is firing zero times, so the drain logic cannot be exercised regardless of
row count.

## Next Step

The identified next step is right: a higher-dimensional randomized fixture
(`--dimensions 16+`, randomized embeddings, `--rows 50000`, `ef_search 500`).
The `--dimensions` flag gap in the `pg18-parallel-scan` CLI command is the
concrete blocker. That flag needs to be added before a meaningful structural
test can run.

## No Issues
