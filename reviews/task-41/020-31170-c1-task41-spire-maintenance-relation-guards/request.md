# Review Request: Task 41 SPIRE Maintenance Relation Guards

## Summary

This checkpoint completes the current SPIRE SQL/test relation-guard sweep by replacing the remaining raw `open_valid_ec_spire_index` callers with `AccessShareIndexRelation` guard ownership, then deleting the raw SPIRE compatibility helper.

The migrated production surface covers SPIRE epoch cleanup, leaf/delta snapshots, maintenance planning/runs, scheduler snapshots/runs, insert-debt snapshots, and cost/cost-tuning snapshots. The remaining test callers in insert prepare and libpq identity-cache probe code now use the same guard path and pass `as_ptr()` only across the immediate unsafe AM call.

## Safety Delta

- Baseline entries: `4427` -> `4393`.
- `src/lib.rs` unsafe-comment baseline entries: `214` -> `188`.
- `open_valid_ec_spire_index(` has no remaining callers in `src/lib.rs` or `src/tests`.
- Relation close ownership moved from hand-written `index_close` calls to guard drop paths for this cluster.

## Reviewer Focus

- Confirm `ec_spire_index_epoch_cleanup_summary` intentionally keeps one guard across both AM reads before dropping it.
- Confirm the migrated test helpers drop the guard before subsequent SPI work or query-cancel flag cleanup.
- Confirm deleting raw `open_valid_ec_spire_index` is correct now that all direct callers are gone.

## Validation

- `bash scripts/check_unsafe_comments.sh`
- `make fmt-check`
- `git diff --check`
- `bash scripts/unsafe_baseline_report.sh`
- `cargo check --all-targets --no-default-features --features pg18,bench`

Packet-local logs and baseline snapshots are in `artifacts/`; see `artifacts/manifest.md`.
