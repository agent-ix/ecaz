# Review request — Task 162 packet 003: Gate G0 kill-check spike (go/no-go)

- Branch: `task-162-ec-distann-m0`; evidence `artifacts/manifest.md` +
  `artifacts/results.jsonl` (28/28 suite steps).
- What this is: the ADR-085 D2 program kill-check — single-node
  recall-vs-H curves × the measured SPIRE per-round pooled-transport cost,
  projecting multinode p50 before any distributed code exists (M2).

## Verdict: **GO**

At the gate-relevant operating point (BW=32, H=8: recall@10 0.9940,
compute p50 12.3 ms), projected multinode p50 is **20.3–28.3 ms** against
the NFR-017 anchor of 37.6 ms (IVF 100k 0.9980) — under the anchor across
the whole measured transport band, with the compute term conservatively
serial. The architecture's core claim holds in the measurement: recall
0.994 is reachable in **8 rounds** at BW=32, and per-query work observed
the BW×H cap exactly (`records_expanded=256 = 32×8`,
`artifacts/nfr019-counter-sample.log`).

Shape findings the projection rests on:

1. **Wide beam, few rounds is the only viable multinode shape.** BW=4
   needs H=64 for 0.995 → projected 78–142 ms (dead). BW=32 reaches
   0.994 at H=8. This matches the DistributedANN paper's regime and
   should inform the M2 defaults (beam_width default 4 is a single-node
   default; multinode wants ≥32).
2. **Recall plateaus at 0.9965 (50k, rabitq, R=32)** past H=16 — the
   graph itself is the ceiling, not the round budget. Raising it is
   build-side work (R, alpha, build L), relevant to the M4 gate at
   matched-recall 0.998.
3. **D4 watch item**: projected transport share at the operating point is
   40–57% of multinode p50 — straddling the D4 baton-passing reopen
   trigger. M2's measured hop RTT decides; flagged now so M2
   pre-registers that measurement.

## Asks

1. Concur with GO (program proceeds to M1 stitch / M2 remote path).
2. Concur that the plateau finding (2) belongs to the M4 gate risk list —
   at 0.998 matched recall the current 50k graph cannot get there at any
   H; levers are build-side.
3. Sanity-check the transport-band choice (1–2 ms/round from task-142
   packet 016 phase rows) — if you read those rows differently, say so
   before M2 builds on this.
