# Task 41 invariant #2 review request: feedback cleanup

## Scope

This packet covers code commit `268d4f63a1fd701a151613019352ea671b1aa0ee`
(`Tighten invariant2 detoast lifetime helpers`) and the strategy-artifact
update in this packet. It responds to packet 147 seq 2 action items E, I, and
the DiskANN `with_ecvector_datum_slice` portion of C.

The code slice handles the actionable invariant #2 feedback from packets 118,
121, 122, 123, and 147 seq 2:

- adds `crate::am::common::detoast::DetoastedVarlena` as the shared palloc
  ownership guard for repeated detoast helper shapes;
- routes HNSW build/source, IVF build, DiskANN build, SPIRE build/scan, and
  typmod detoast ownership through the shared guard;
- tightens DiskANN `with_ecvector_datum_slice` and the heap-source wrapper to
  higher-ranked callback bounds so the slice lifetime cannot be selected as
  the return type;
- rewrites the detoast cleanup contract to state that `pgrx::error!` unwinds
  Rust frames and that PostgreSQL memory-context cleanup is the fallback for
  non-unwinding longjmp paths;
- removes `DetoastedTypmodArray::as_ptr`, replacing it with a
  guard-contained `single_typmod()` accessor.

The strategy artifact in packet 121 now explicitly says scoped slice closures
must use higher-ranked callback bounds.

## Feedback Processed

Read reviewer feedback through packet 147 seq 2. The packets without blockers
were recorded as accepted. The concrete cleanup implemented here addresses:

- packet 118 / 124 / 121: DiskANN scoped slice callback was not HRTB;
- packet 117 / 118 / 119 / 120 / 121: repeated detoast guards justified
  consolidation (147 seq 2 item E);
- packet 122: typmod guard exposed a raw `as_ptr` accessor;
- packet 123 / 121: detoast cleanup wording needed to distinguish Rust unwind
  from PostgreSQL longjmp fallback (147 seq 2 item I);
- packet 121 / 131: strategy should require HRTB for scoped closures, and the
  DiskANN ecvector helper should use that shape (part of 147 seq 2 item C).

## Remaining Approved Action Items

Packet 147 seq 2 promotes the remaining notes to approved action items. They
are intentionally not changed in this detoast/HRTB cleanup slice:

- A/B: split writable page helpers into read-only and writable variants, then
  consolidate shared helpers;
- remaining C/D: HRTB sweep over page/DSM helpers and return-style
  unification;
- F/G/H: scan_debug shared-helper wrapper, DSM init scoped slices, and UUID
  fixed-payload helper.

## Validation

See `artifacts/manifest.md` for command metadata.

- `cargo fmt --all --check` passed with the repository's existing stable-rust
  rustfmt configuration warnings.
- `cargo check --no-default-features --features pg18` passed with the known
  pre-existing unused imports warning in `src/am/mod.rs`.
- `git diff --check 268d4f63^ 268d4f63` passed.

## Reviewer Focus

- Confirm `with_ecvector_datum_slice` and the DiskANN heap-source wrapper are
  genuinely higher-ranked and cannot return the borrowed slice.
- Confirm repeated detoast ownership now funnels through `DetoastedVarlena`
  without changing AM-specific error messages.
- Confirm the remaining packet 147 seq 2 action items are correctly deferred
  to subsequent local slices rather than mixed into this detoast/HRTB cleanup.
