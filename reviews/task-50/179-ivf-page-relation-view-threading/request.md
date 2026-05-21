# Review Request: IVF Page Relation View Threading

## Scope

This packet reviews commit `dc6db527c14d0e483bfa755610d1d83ce9df3fa7` (`Thread IVF page relation view through helpers`).

The slice carries the existing `IvfPageRelation` view through IVF posting rewrite and diagnostics helpers instead of reconstructing it from raw `pg_sys::Relation` pointers in each private helper.

## Unsafe Burndown

- Constructs the IVF page relation view once at the unsafe rewrite entry point, then passes it through the private per-block rewrite helpers.
- Reuses the existing diagnostics relation view while walking posting blocks.
- Removes redundant private raw-relation boundary blocks without making raw-pointer helper APIs look safe.

Unsafe ledger movement:

- previous packet 178 ledger: `1843`
- packet 179 ledger: `1841`
- net reduction: `2`

High-signal file counts from `make unsafe-block-count`:

- `src/am/ec_ivf/page.rs`: `33 -> 31`

## Validation

Packet-local artifacts are under `reviews/task-50/179-ivf-page-relation-view-threading/artifacts/`.

Passed:

- `cargo-check-pg18-bench.log`
- `cargo-check-pg18-pg-test.log`
- `git-diff-check.log`
- `unsafe-block-count.log`
- `unsafe-ledger-generate.log`
- `unsafe-ledger-check.log`

## Reviewer Focus

Please check that the `IvfPageRelation` view remains rooted at the unsafe entry-point contracts and that the helper signatures do not broaden the lifetime or ownership assumptions for the underlying PostgreSQL relation.
