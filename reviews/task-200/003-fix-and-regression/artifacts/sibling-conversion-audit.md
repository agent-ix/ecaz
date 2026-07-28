# Task 200 sibling conversion audit

Audit head: `e566ebec1` plus the executable regression-gate changes in the
working tree. Search used:

```text
rg -n 'value::<(Vec<u8>|pgrx::datum::Array)' src/am/ec_distann
```

## Serving-path conversions

| Site | Shape | Scope / disposition |
| --- | --- | --- |
| `src/am/ec_distann/generation_read.rs:1327` | `payload_values.value::<pgrx::datum::Array<&[u8]>>()` inside the row materialization map | Production physical-row payload materialization. The array elements are copied into the returned `Vec<Vec<u8>>`, bounded by the resolved request batch and projection column count. The clean A1 held-transaction run exercised this path for 300 ANN calls and plateaued; it is not the Task 200 owner-seed leak site. |
| `src/am/ec_distann/remote_endpoint.rs:538` | `payload_values.value::<pgrx::datum::Array<&[u8]>>()` inside the SPI result row loop | Production remote payload materialization. It has the same bounded request-batch/column-count ownership shape as the generation-read site. It is explicitly covered by the clean A1 result: 300 ordinary ANN calls on one backend in one transaction, RSS plateaued at 260780 KB. No unbounded growth was observed, so this sibling remains unchanged and is recorded as bounded by the returned batch. |

These two array conversions are the only `Array<&[u8]>` serving-path matches.
The regression gate must continue to be run when either materialization path is
changed, because the Task 200 coverage helper itself is benchmark-only and does
not exercise the production CustomScan in the same way.

## Non-serving conversions

The remaining `Vec<u8>` matches in `ec_distann` are catalog/state lookups or
build/lifecycle loops, not repeated production ANN row materialization:

- `generation_read.rs:1951-1966` and `:2203` decode one active-head/active-pointer
  row per operation.
- `head_sample.rs:864-886`, `participant_lifecycle.rs:920`,
  `traversal_replica.rs:537-557` and `:1105-1113` decode bounded state rows.
- `handoff.rs:738` and `:991` decode graph rows in build/handoff cursor loops,
  outside the production read path and outside the benchmark coverage helper.
- The other `Vec<u8>` matches in `build_coordinator/`, `coordinator_*`,
  `generation_catalog.rs`, `generation_store.rs`, `node_registry.rs`, and
  `traversal_replica.rs` are lifecycle, catalog, or replica-build operations;
  they are not per-row serving conversions in a held ANN transaction.

The audit does not claim that every future pgrx owned-value conversion is safe
by inspection. Any new conversion inside a repeated serving loop needs the same
held-transaction RSS-slope check. The production A1 evidence is why
`remote_endpoint.rs:538` is currently treated as bounded rather than silently
accepted from code reading alone.
