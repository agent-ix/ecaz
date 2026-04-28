# Task 28 IVF n128 Cost Diagnostic

This packet follows up the reviewer direction from packet 30070: diagnose
whether the planner still blocks `nlists=128` IVF surfaces after the earlier
cost repair.

## Measurement Result

Fixture:

- Local PG18 scratch database `postgres`.
- Existing isolated DBPedia-derived 10k surfaces:
  - `task28_ivf_postopt10k_n64w25`
  - `task28_ivf_postopt10k_n128w25`
- Prepared KNN query over `task28_ivf_postopt10k_n128w25_corpus`.
- Normal planner path with `enable_seqscan=on`; no forced index GUC.
- `ec_ivf.nprobe` sweep: `8,16,24,32`.
- Cache state: warm local development run; no explicit cache drop.
- Memory high-water mark: not captured.

| nprobe | selected plan | modeled index cost | execution time | buffers |
| ---: | --- | ---: | ---: | --- |
| 8 | `Index Scan` | `43.00..594.25` | 77.796 ms | `hit=396 read=237` |
| 16 | `Index Scan` | `43.00..644.51` | 50.494 ms | `hit=605 read=134` |
| 24 | `Index Scan` | `43.00..694.77` | 59.792 ms | `hit=739 read=70` |
| 32 | `Index Scan` | `43.00..745.03` | 67.941 ms | `hit=809 read=39` |

The current branch no longer reproduces the original packet-30053 planner
blocker for the n128 surface. Under normal planning, PG18 selects the IVF index
for nprobe 8, 16, 24, and 32. The earlier cost repair remains effective for
this high-nlists diagnostic.

This is not a recommendation to move the working point to n128 immediately:
packet 30054 still shows the n128 recall/latency tradeoff, and packet 30070
keeps n64 as the current high-recall reference. This packet only closes the
planner-selection question so the next implementation slice can focus on
scored-posting volume.

## Artifacts

- `artifacts/n128_cost_diagnostic.sql`
- `artifacts/n128_cost_diagnostic.log`
- `artifacts/manifest.md`

## Validation

- `git diff --check`

No Rust code changed for this packet.

## Recommendation

Do not spend another slice on n128 cost tuning unless a later larger fixture
reproduces a planner miss. Continue with the reviewer-directed posting-scan
score-volume work next. DiskANN remains task 29 and is not included here.
