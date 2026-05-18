# Task 41 Review Request: SPIRE Snapshot Relation Guards

## Scope

This checkpoint migrates SPiRE coordinator aux-store snapshot and cleanup
loops from manual `relation_open` / `relation_close` pairs to the shared
`RelationGuard` introduced in packet 31191.

Touched file:

- `src/am/ec_spire/coordinator/snapshots.rs`

Code commit: `959ab8e5e41bf29b7c3578da5f7d77b7d1bdf93a`

## Safety Invariant

`open_storage_relation_or_index` returns the root index relation directly
when the storage relid equals the index relid. For aux-store relids, it
opens the relation with the requested lockmode and returns the raw relation
pointer plus an owning `RelationGuard`.

Callers bind the guard for the full loop iteration, so aux-store relations
stay open through block counting, tuple scans, and tuple deletion. Early
`Err` returns and pgrx unwinds drop the guard and close the relation with
the matching lockmode.

## Baseline Impact

Unsafe comment baseline decreased:

- before: `4251`
- after: `4245`

This removes six tracked unsafe sites from manual aux-store relation
open/close handling.

## Validation

See `artifacts/validation.md`.

Commands run:

- `cargo fmt`
- `bash scripts/check_unsafe_comments.sh --update-baseline`
- `git diff --check`
- `bash scripts/check_unsafe_comments.sh`
- `make fmt-check`
- `bash scripts/unsafe_baseline_report.sh`
- `cargo check --all-targets --no-default-features --features pg18,bench`

## Review Focus

- Confirm `_storage_relation_guard` lifetime covers each scan/delete use.
- Confirm the helper preserves the previous no-open behavior for
  `storage_relid == index_relid`.
- Confirm RowExclusive and AccessShare lockmodes are still paired correctly
  with relation close through `RelationGuard`.
