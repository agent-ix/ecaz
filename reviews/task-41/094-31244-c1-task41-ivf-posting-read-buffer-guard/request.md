# Review Request: Task 41 IVF posting read buffer guard

## Summary

Task 41 buffer-resource slice for IVF non-PG18 posting-list readers.

This adds `PinnedBufferGuard::read_main` for non-PG18 builds and uses it in
`src/am/ec_ivf/page.rs` for:

- `visit_ivf_postings_for_list_block`
- `visit_all_ivf_postings_for_block`
- `visit_all_ivf_posting_refs_for_block`

Code commit: `962cff13`

## Safety Effect

- Moves pin-only posting-list `ReleaseBuffer` ownership into
  `PinnedBufferGuard`.
- Keeps the visitor calls scoped inside the guard lifetime.
- Gates `PinnedBufferGuard::read_main` to non-PG18 because this slice only
  uses it on the non-PG18 fallback path.
- Updates the unsafe comment baseline from `3928` to `3922`.

## Review Focus

- Confirm references yielded by `visit_all_ivf_posting_refs_for_block` cannot
  outlive the pinned buffer guard.
- Confirm non-PG18 `PinnedBufferGuard::read_main` correctly matches
  `ReadBufferExtended` pin ownership.
- Confirm the PG18 build does not carry a dead-code warning for the non-PG18
  helper.

## Validation

See `artifacts/manifest.md` and `artifacts/validation.md`.
