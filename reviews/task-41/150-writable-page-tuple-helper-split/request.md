# Task 41 invariant #2 review request: writable page tuple helper split

## Scope

This packet covers code commit `9b7ec742fbce6890f26855f16a51a9ff475fc71c`
(`Split writable page tuple byte helpers`).

The slice handles packet 147 seq 2 item A and the HNSW-local portion of item B:

- DiskANN insert page tuple rewrites now use a mutable-only
  `with_page_tuple_bytes_mut` helper.
- DiskANN vacuum keeps a read-only `with_vacuum_page_tuple_bytes` helper for
  expected-byte checks and adds mutable-only `with_vacuum_page_tuple_bytes_mut`
  for rewrites.
- HNSW insert and vacuum rewrites now share
  `shared::with_writable_page_tuple_bytes`, which hands callers only
  `&mut [u8]`.
- The old callback shape that co-issued `*mut u8` and `&[u8]` over the same
  tuple bytes was removed from these helpers.

Callers decode from the exclusive mutable slice, build owned encoded bytes, and
copy back with `copy_from_slice`.

## Deferred Items

Packet 147 seq 2 item D, plus item F, are still deferred: HNSW
`with_page_line_tuple_bytes` still returns `Option<R>` and scan_debug still has
its debug-local wrapper. Those are coupled to read-helper return-style
unification and should land as the next slice.

## Validation

See `artifacts/manifest.md` for command metadata.

- `cargo fmt --all --check` passed with the repository's existing stable-rust
  rustfmt configuration warnings.
- `cargo check --no-default-features --features pg18` passed with the known
  pre-existing unused imports warning in `src/am/mod.rs`.
- `git diff --check 9b7ec742^ 9b7ec742` passed.

## Reviewer Focus

- Confirm no writable tuple helper in the touched files passes both `*mut u8`
  and `&[u8]` to the same callback.
- Confirm HNSW insert/vacuum share the same mutable helper in `shared.rs`.
- Confirm DiskANN vacuum still compares expected raw bytes before WAL rewrite.
