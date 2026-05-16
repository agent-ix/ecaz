# Review Request: SPIRE Vector Dedupe Key Prefixes

Code checkpoint: `72dd95fe` (`Disambiguate SPIRE remote dedupe key prefixes`)

## Scope

- Processes reviewer feedback from `review/30667-spire-vector-identity-contract`.
- Moves remote candidate vec-id dedupe key prefix bytes to `0xA0`/`0xA1`, so
  they no longer visually overlap the storage vec-id discriminator bytes
  `0x01`/`0x02`.
- Adds a regression test asserting the remote dedupe key prefixes stay distinct
  from the persisted vec-id discriminators.
- Tracks the remaining global vector ID allocation requirement under Phase 10.6.
- Tracks deeper recursive routing diagnostic drift coverage under Phase 10.1a.

## Validation

- `cargo fmt --check`
- `git diff --check`
- `cargo test --no-default-features --features pg18 remote_candidate_dedupe_key_prefixes_do_not_overlap_vec_id_discriminators --lib`

## Review Focus

- Confirm the new prefix values make the in-memory merge key encoding less
  ambiguous without implying an on-disk format migration.
- Confirm the remaining writer-side global-ID and deeper diagnostic coverage
  items are captured in the right Phase 10 sections.
