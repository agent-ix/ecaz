# Seq-02 reviewer finding disposition

Request seq-04 addresses the two local-read blockers in
`feedback/2026-08-26-02-reviewer.md`. No source implementation began, and no
accepted format, descriptor, catalog, lifecycle, DML, threshold, or evidence
decision changed.

| Finding | Revised disposition |
| --- | --- |
| B1 | `RemoteSkipped` is now explicitly remote-only. Covered local `Frozen` rows preserve the control's two attempts: sidecar and exact row-tier TID under `estate.es_snapshot`, then both under the latest snapshot. A visible row-tier tuple without its same-transaction sidecar is corruption; both tuples invisible after the latest retry raise the existing `EC_GENERATION_MISSING: published row-tier tuple ({},{}) disappeared` error verbatim and never skip. |
| B2 | Eligible local outputs become `FrozenPayloadPending { vec_id, row_tid }` while both identities are available. `materialize_pending_physical_window` batches them at lazy-window materialization (one batch over the proven set in eager mode) and performs at most one local SPI `unnest(row_tids, vec_ids) WITH ORDINALITY` lookup per snapshot attempt and window, never one SPI query per row. Both identity echoes are checked, and packet-002 telemetry separates local initial/retry batches from remote owner lookup work. |
