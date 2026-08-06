# Task 215 release contract

## Exact behavior

The production default change is limited to the two FR-081 search-budget GUCs:

| setting | current default | candidate default |
|---|---:|---:|
| `ec_distann.beam_width` | 4 | 64 |
| `ec_distann.hop_rounds` | 100 | 8 |

`ec_distann.candidate_heap_limit` remains 32 as a session GUC. The existing
runtime safety clamp resolves the effective heap to at least the beam width,
so the control executes with effective L=32 while BW64 executes with effective
L=64; this is not a new index or benchmark-only behavior. With the normal production head
derivation, `head_search_width = (beam_width * 2).max(32)` and the effective
seed count follows that width: 32 in the control and 128 in the candidate.
No benchmark-only seed override is part of the candidate.

The candidate's hard expansion budget is `64 * 8 = 512`, versus `4 * 100 =
400` for the control. Early convergence can stop either path earlier; the
configured product remains the NFR-019 upper bound.

## Compatibility and rollback

The change affects new sessions that inherit the server GUC defaults. Existing
indexes, generations, placement records, persisted head state, wire format,
and materialization policy remain compatible and unchanged. Operators can roll
back without rebuilding an index by setting:

```sql
SET ec_distann.beam_width = 4;
SET ec_distann.hop_rounds = 100;
```

For a persistent rollback, restore the two GUC values in the server
configuration and reload/restart using the normal PostgreSQL procedure. A
release rollback to the prior extension binary is also format-compatible.

The release A/B must reject promotion if ordered results or recall change,
owner engagement/topology fails, normalized NFR-021 evidence is nonconforming,
or latency has no material Pareto improvement. In every failure case the
shipped behavior remains BW4/H100; this contract does not pre-authorize a
default flip.

## Excluded axes

The candidate does not use attribution-only GUCs, the traversal replica,
benchmark seed selectors, scan notices, exact-neighbor or materialization
overrides, a new head construction, or any Task 216 owner serialization
candidate.
