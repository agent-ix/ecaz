# A4 Upper-Hierarchy Oracle-K Calibration

## Context

A4 is still blocked on live graph-first recall. The repaired quantized path is no longer the main issue on the deterministic `10k` fixture: exact quantized Recall@10 is high, while the live graph path is still far below it.

This slice used the persistent fixture-backed harness on one deterministic `10k` graph so the only moving part was search behavior.

## Goal

Calibrate whether the remaining miss lives in:

- layer-0 search itself
- upper-layer navigation from the stored entry point
- or graph construction / hierarchy quality

## What I Tried

### 1. Failed runtime experiment: carry a multi-seed upper-layer result window into layer-0

I added an exploratory runtime path that replaced the single descended seed with the current upper-layer result window before starting layer-0 search.

Result on deterministic `10k`, `m=8`, `ef_search=128`, `100` fixed queries:

- previous live graph recall on this fixture family: `27.4%`
- exploratory multi-seed upper-layer bootstrap: `25.4%`

This is a failed lane. Widening the current upper-layer search from the stored entry point did not help; it regressed slightly.

### 2. Reconfirmed layer-0 is fine when seeded correctly

Exact-seed summary on the same built `10k` graph, `m=8`, `ef_search=128`, `50` fixed queries:

- live graph: `25.8%`
- exact top-1 seeded layer-0 search: `28.8%`
- exact top-10 seeded layer-0 search: `83.4%`
- exact quantized: `83.4%`

Interpretation:

- a single perfect layer-0 seed is not enough
- but `10` correct layer-0 seeds fully recover exact quantized recall on this sample

### 3. New deterministic upper-hierarchy oracle-k probe

I added a debug surface that:

- scores **all nodes at `metadata.max_level`**
- takes the best `k` top-level seeds for the query
- starts layer-0 search directly from that seed set

This is not a proposed production path. It is a calibration oracle on the same built graph.

Results on the same deterministic `10k` graph, `m=8`, `ef_search=128`, `50` fixed queries:

- `k=3`: `49.8%`
- `k=5`: `63.2%`
- `k=8`: `79.6%`
- `k=10`: `83.4%`
- exact quantized on the same sample: `83.4%`

## What This Means

The strongest current read is:

1. The built graph is not fundamentally unusable.
2. Layer-0 search is not the primary blocker.
3. The upper hierarchy **does contain enough good entry nodes** to recover exact quantized recall.
4. The live path is failing because it does not surface enough of those good top-level seeds from the stored entry point.

That narrows the real problem to the upper hierarchy:

- upper-layer reachability / connectivity
- or a meaningful divergence between tqvector upper-layer navigation and `hnsw-rs` search behavior

## Failed Paths Worth Keeping On Record

- raw-source build objective swap: flat
- reference-style metadata entry point: slight regression
- direct multi-seed carrydown from the stored entry point: slight regression
- single exact seed: insufficient
- single top-level oracle seed: insufficient

## Likely Next Steps

- Compare tqvector upper-layer navigation against `hnsw-rs` search behavior directly.
- Audit whether the relevant top-level nodes are unreachable from the stored entry point on the persisted hierarchy.
- If the hierarchy is good but the stored entry point is too narrow, consider whether build-time persistence of a small top-level entry set is a cleaner fix than more search heuristics.

## Notes

- The deterministic fixture reset is still expensive (`~11m`), but report/oracle probes on the built graph are cheap enough to use as a debugging rig.
- This slice is evidence-driven only. No production fix is claimed yet.
