# Review request — Task 162 M0 bench cells (parity A/B, D7 codecs, NFR-018, D3)

- Branch: `task-162-ec-distann-m0`; measured build `1fd015935`
- Evidence: `artifacts/manifest.md` (head SHA, commands, cited lines),
  `artifacts/results-10k.jsonl`, `artifacts/results-50k.jsonl`, suite config
  `task-162-m0-suite.json` (bespoke; justification in its description and
  the manifest)
- Release backend verified in-run; recall-before-latency protocol; one
  index per replicated table per arm.

## Verdict summary (numbers + trace in `artifacts/manifest.md`)

1. **FR-075-AC-4 parity (rabitq vs rabitq)**
   - 10k: PASS — recall matched within 0.002 across the sweep; matched-recall
     latency 0.72×–1.11× (distann *dominates* below 0.9995); the perfect-recall
     point is 1.47×.
   - 50k: PASS through ~0.988 recall (1.1×); **FAIL at 0.995** (2.05×:
     13.7 ms vs 6.67 ms). The high-recall tail needs more expansions per
     query and each expansion is heavier than a diskann hop (records are
     ~15× larger, one heap read per expansion).
2. **D7 codec choice**: measured, and the data contradicts the GroupedPq
   default — gpq tops out at 0.9905 (10k) / 0.9245 (50k) while rabitq
   reaches 1.0000 / 0.9950 faster. TQ at default R=32 cannot even build
   (record > page). **Recommendation: flip `neighbor_code_format` default
   to `rabitq`** (operator/spec decision — D7 says default pinned at M0).
3. **NFR-018 / D1**: all buildable formats well inside the 4.0× budget
   (rbq 1.38–1.88×, gpq 0.43–0.88×). D1's ~4× arithmetic corresponds to
   the TQ-class 768 B code, which is exactly the format that fails to fit
   a page — so the *practical* D1 posture is: keep D11 lean records, use
   rabitq/gpq codes, no fallback layout needed at M0 scales. Two storage
   footnotes: one-record-per-page waste (~23% for rbq) and the head-sample
   tier (C×6.1 KB) are the next storage levers.
4. **D3 / FR-080-AC-4**: recall sensitivity to C is ≲0.02 over
   C∈{1024,4096,16384} at 50k; default C=4096 is defensible; smaller C
   saves storage and head-build time. C=16384 costs a ~2-minute first-query
   head build — see finding below.

## Findings the coder (me) proposes acting on

- **Sweep semantics** (already landed, `1fd015935`): run 1 measured the
  hop_rounds sweep inert — the D9 early-exit bar (`ec_distann.top_k`) is
  the quality knob; the profile now sweeps it. Run-1 evidence preserved.
- **Per-backend head-graph build cost** (~10 s at C=4096) shows up as
  first-query latency. Not a gate-blocker (p50/p95 clean) but it is real
  operational pain; candidate fixes (persist head adjacency with the epoch,
  or a shared-memory cache) belong with FR-082 (M3). Flagged, not fixed.
- **50k high-recall parity gap (2.05×)**: the honest M0 reading is that
  the single-node distann scan pays for its lean-record design at the deep
  end. Candidate levers before M1: beam_width > 4 at scan time (fills the
  32-wide kernels), batched neighbor scoring via CandidateBatch (currently
  per-code scalar scoring in `DistannPreparedQuery::score_dist`), and heap
  prefetch for the expansion batch. I propose these as the next code slice
  in this task **before** declaring the M0 exit criterion met or missed.

## Asks

1. Concur or push back on the D7 default flip to rabitq.
2. Concur that the 50k 0.995-point 2.05× is a "fix with the identified
   levers, re-measure" rather than a G0-relevant architectural smell (the
   G0 kill-check in packet 003 measures the multinode projection
   separately).
3. Anything else the bench matrix should cover before 100k (M4 gate runs
   the full 10/50/100k × release-anchor protocol; M0's task text asks
   10k/50k only).
