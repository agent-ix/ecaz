# Task 97 Phase 0 QJL Surface Inventory

## Result

Task 97 should proceed, but only against currently implemented QJL/gamma-aware
TurboQuant surfaces:

- IVF TurboQuant generic `IvfPreparedQuery::TurboQuant`
- SPIRE TurboQuant generic `SpirePreparedAssignmentScorer::TurboQuant`
- HNSW TurboQuant default exact scorer `HnswTurboQuantPreparedQuery::Exact`

The standard 1536-dim / 4-bit benchmark cells do not reach QJL. They select
the tiled no-QJL lane and belong to the existing LUT32 / HNSW exact-mode work,
not to Task 97. DiskANN TurboQuant is also out of scope because its search-code
surface explicitly requires the no-QJL 4-bit dimension lane.

## Mode Rule

`ProdQuantizer::qjl_enabled(dim, bits)` disables QJL only when both are true:

- `bits == 4`
- `rotation::tile_dim(dim).is_some()`

`rotation::tile_dim` currently returns `Some(512)` only for `dim == 1536`.
Therefore:

- `dim=1536,bits=4` is no-QJL (`ExactScoreMode::MseNoQjl4Bit`)
- `dim!=1536,bits=4` is QJL (`ExactScoreMode::MseLutQjl`)
- `bits!=4` can be QJL-capable internally, but public `tqvector`/`ecvector`
  SQL surfaces currently enforce canonical 4-bit encoding, so Task 97 should
  not introduce non-4-bit production surfaces.

## AM Matrix

| AM | Standard 1536d/4-bit | Non-1536d/4-bit | Task 97 disposition |
| --- | --- | --- | --- |
| IVF TurboQuant | no-QJL LUT batch path | QJL generic `TurboQuant` prepared query | in scope |
| SPIRE TurboQuant | no-QJL LUT batch path | QJL generic prepared assignment scorer | in scope |
| HNSW TurboQuant | no-QJL exact modes available; default exact still works but standard 1536 is not QJL-active | QJL default exact scorer | in scope |
| DiskANN TurboQuant | no-QJL search-code prefilter | rejected by current build/query prep | out of scope |

## Standard Fixture Reachability

The existing standard/DBpedia evidence path is 1536-dimensional. At canonical
4-bit encoding, this is the special tiled no-QJL compatibility lane. Task 97
must not claim QJL coverage from the standard 1536d fixture.

## Proposed Measurement Fixture

Use a deterministic synthetic non-tiled fixture for Task 97 local evidence:

- `dim=1024`
- `bits=4`
- `seed=42`
- storage format `turboquant`
- AMs: IVF, SPIRE, HNSW

`dim=1024,bits=4` is production-reachable through current canonical 4-bit
encoding, is QJL-active, and avoids creating a new TQ mode or storage surface.
The standard 1536d cells should be reported as no-QJL/absent for Task 97 and
left for the Task 99 complete profile.

## Implementation Scope After Approval

Proceed with a `src/quant/qjl32/` block-kernel family only after reviewer
acceptance of this inventory. The first design packet should target the current
gamma-aware scalar math:

```text
mse_sum + gamma * prepared.qjl_scale * qjl_sum
```

Candidate metadata should continue using `CandidateMeta::Gamma` where residual
signs are carried in the code payload. `CandidateMeta::GammaAndResidualSigns`
remains available if a future storage surface separates signs from code bytes,
but Task 97 should not introduce that split.
