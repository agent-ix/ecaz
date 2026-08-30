# Task 231 final decision summary

- Accepted measurement extension SHA:
  `66b53998a955b583ca43c0e967806aa29e0a4404`.
- Frozen suite config SHA-256:
  `48dbcbf38383d99418e99b6f246149c5fb7b552b696444ed6cd8e9379da1d211`.
- Matrix: 27/27 steps succeeded; PostgreSQL 18.3, release extension,
  checksums on, 128 MiB shared buffers, isolated one-index-per-table fixtures.
- Decision: **STOP — do not promote fixed-stride node blocks on this local
  PostgreSQL 18 lane with 128 MiB shared buffers.** A host whose shared buffers
  can hold the index is a different experiment; this matrix sits at the OS
  page-cache boundary at 50k and 100k.

## Frozen warm decision matrix

All latency values are concurrency-1 `physical_benchmark_latency.mean_ms`.
The frozen rule requires fixed-stride to improve by at least 5.0% and 0.50 ms
in both independent 100k BW4/H100 matched pairs.

| Scale | Pair | Control ms | Fixed ms | Fixed direction | Recall control/fixed | Result |
|---|---:|---:|---:|---:|---:|---|
| 10k | A | 7.97 | 8.23 | 0.26 ms / 3.3% slower | 0.9990 / 0.9990 | report |
| 10k | B | 7.53 | 7.07 | 0.46 ms / 6.1% faster | 0.9990 / 0.9990 | report |
| 50k | A | 10.20 | 9.02 | 1.18 ms / 11.6% faster | 0.9545 / 0.9545 | report |
| 50k | B | 8.82 | 9.85 | 1.03 ms / 11.7% slower | 0.9545 / 0.9540 | report |
| 100k | A | 8.60 | 9.50 | 0.90 ms / 10.5% slower | 0.9290 / 0.9300 | **fail** |
| 100k | B | 9.79 | 8.11 | 1.68 ms / 17.2% faster | 0.9290 / 0.9295 | pass |

One 100k pair passed and one failed. The frozen rule explicitly defines that
outcome as STOP; pairs are not averaged and no secondary result can reverse it.
All recall results cleared the per-scale floor and the fixed-minus-control
neutrality tolerance.

The split is dominated by fixture position, not a sign-changing layout effect.
At first position, control/fixed measured 8.60/8.11 ms; at second position they
measured 9.79/9.50 ms. Fixed is marginally faster at both matched positions,
but by only 0.49 ms first and 0.29 ms second. Order-averaged latency is 9.195 ms
control versus 8.805 ms fixed, a 0.390 ms / 4.2% candidate improvement—below
both frozen PROMOTE bounds. The conservative both-pairs rule requires a layout
effect large enough to survive the roughly 1.4 ms first-to-second position
effect in both directions; this one does not.

## Secondary evidence

- The report-only 100k BW16/H25 position-confounded pair was 8.32 ms control
  versus 7.59 ms fixed (0.73 ms / 8.8% faster), with recall 0.9695 versus
  0.9700. It does not affect the primary verdict.
- Controlled-cold one-shot latency (control/fixed) was 2905.5/3277.9 ms and
  4227.1/3498.0 ms at 10k, 3780.4/4301.1 ms and 3548.6/3859.4 ms at 50k,
  and 7939.9/7928.5 ms and 7074.9/7217.8 ms at 100k for pairs A and B.
  These mechanism-only results were also mixed and are not warm-path evidence.
- All 36 per-node cold-residency rows passed. Every row reported
  `resident_buffers_after=0` and `evicted_fraction=1.0` after a churn relation
  exceeding twice the 134,217,728-byte shared-buffer budget.
- All 39 candidate per-node storage rows exactly matched
  `node_store_bytes = 8192 + 16384 * hot_tier_rows`; there were zero formula
  deviations.
- Pair-A physical generation bytes at 10k/50k/100k were respectively
  242,860,032 / 1,243,496,448 / 2,498,207,744 for control and
  331,857,920 / 1,657,700,352 / 3,315,032,064 for fixed.
- The 64-statement warm DML drill produced zero raw-store growth for every
  control. Fixed growth was 14,647,296 bytes at 10k, 14,909,440 at 50k, and
  15,171,584 at 100k (228,864 / 232,960 / 237,056 bytes per statement).
  Dividing by the 16,384-byte extent gives 894 / 910 / 926 extents total, or
  13.97 / 14.22 / 14.47 extents per statement. Under the preregistered model
  where each of 32 deletes appends one tombstone extent, the 32 replacements
  account for 26.94 / 27.44 / 27.94 extents apiece: one replacement node plus
  roughly 26 backlink amendments, consistent with the `1 + R` model at degree
  32 and exposing the candidate's write amplification.
  Every DML gate passed, but concurrency was deliberately skipped, so the
  implementation remains single-writer evidence only.
- Corrected NFR-021 derived evidence is conforming and decision-eligible for
  both roles. Maximum normalized per-owner growth is 1.095044 for control and
  0.998937 for candidate, below the 2.0 bound; non-owned records, orphans,
  unsharded derived bytes, and coordinator-resident unsharded bytes are all
  zero, and head capacity is constant. The candidate's lower ratio is the
  expected small benefit of constant bytes per record; the control's
  variable-length graph representation drifts more across scales.

## Runner and attempt boundaries

The full fixture run first exited nonzero only while deriving NFR-021. The
pre-correction result incorrectly mixed same-variant control and candidate
rows and reported 2.171204 for both. Packet 007 checkpoint
`795af9616a304f2bf276d57c2c151270198f9bd4` scopes labeled rows by decision
role. Resuming the unchanged all-succeeded manifest reused all 27 fixtures,
regenerated the derived rows above, and exited zero. No extension or fixture
measurement changed.

An earlier startup collision at the 100k pair-A fixed fixture was diagnosed as
a port already held by a stale task-owned process. That attempt is preserved
separately and excluded. The fixture was rerun fresh; no result from the
collision attempt contributes to this decision.

The canonical raw source is `run/results.jsonl`; `run/suite-manifest.json`
records every step as succeeded. Per-arm `distann-multinode-summary.log` files
are the compact fixture receipts cited by the structured results.
