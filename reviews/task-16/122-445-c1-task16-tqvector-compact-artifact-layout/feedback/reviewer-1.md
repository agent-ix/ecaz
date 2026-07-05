## Feedback: compact `tqvector` artifact layout — ACCEPTED

Verified against current head at `5ff3703`:

- `src/lib.rs:68-73`: `HEADER_BYTES = 2` (dim only), `MIN_BINARY_BYTES = 6`
  (dim + gamma), `DEFAULT_QUANT_BITS = 4`, `DEFAULT_QUANT_SEED = 42`
- `src/lib.rs:82-98`: `validate_tqvector_seed` / `validate_tqvector_bits`
  reject non-canonical values at the type surface, so the invariants are
  enforced, not assumed

### What's right

- **Wire format is compact and honest.** `[u16 dim][f32 gamma][codes]`
  is exactly what a sibling artifact needs: enough to be self-describing
  at the per-row level for `dim` (so operator functions work without
  typmod), with no per-row `bits`/`seed` bytes eating space. 15 B → 6 B
  header is a real 60% reduction in the per-row overhead.
- **The compromise is the right compromise.** The decision note
  acknowledges that a pure typmod-only layout (hoisting `dim` too)
  wasn't viable because `tqvector`'s SQL I/O functions don't receive
  typmod without a larger refactor. Keeping `dim` inline and treating
  `bits`/`seed` as canonical invariants matches the artifact type's
  actual role — it's not a general quantizer container, it's the
  TurboQuant-family artifact in its one canonical configuration. If a
  second configuration ever needed a persisted form, it would be its
  own sibling type per the ADR-043 family-specific-sibling rule, not a
  parameterized variant of this one.
- **Invariant enforcement at encode time is the right gate.**
  `encode_to_tqvector(embedding, bits, seed)` now rejects
  non-canonical `bits`/`seed` before bytes hit the heap, not just at
  operator time. Prevents a class of "the bytes parse but the invariant
  was never actually checked" bugs that a pure wire-format gate would
  leave open.
- **Canonical/sibling pg regression finally exists.** The `442`
  review flagged that sibling containment was structural-only with no
  test. `test_pq_fastscan_indexed_ecvector_ignores_tqvector_sibling`
  closes that gap end-to-end: a table carrying both columns, an index
  built on the `ecvector` column, and a positive assertion that the
  runtime state + emitted scores match `ecvector`, not the sibling.
  This is exactly the Quant-fields checklist item.
- **Encoder round-trip is asserted by
  `test_encode_to_tqvector_round_trips_canonical_artifact_layout`.**
  Locks in "the bytes the encoder produces are the bytes the type
  persists" — so a future layout regression can't silently pass.
- **README front door updated.** Quick-start examples build on
  `ecvector` with `ecvector_ip_ops`, `tqvector` is labeled an
  artifact/debugging surface. A new user reading the README can no
  longer accidentally put `tqvector` in a row column.

### Concerns

1. **Invariant lock-in is load-bearing on the type contract, and
   that's worth documenting visibly.** Today `tqvector` literally
   means "TurboQuant family, 4-bit, seed 42, raw MSE codes + gamma".
   If `DEFAULT_QUANT_BITS` or `DEFAULT_QUANT_SEED` ever shifted,
   every existing `tqvector` heap datum would become unreadable.
   That's fine — it's an artifact type — but the constraint should
   be written down on the ADR-043 §Quantized sibling artifacts
   section so a future contributor doesn't consider "just change the
   default" without understanding the artifact surface.
2. **Migration-from-old-layout posture.** Any existing `tqvector`
   datums in pre-`445` databases carry the 15 B header layout; the
   new I/O code won't read them. Posture here is "no deprecated
   formats kept around" — users rebuild. That's the right call for
   pre-release, but worth one line in the packet / ADR confirming
   the artifact type explicitly does not promise cross-version
   binary stability yet.
3. **`parse_text` still accepts the broader surface, just rejects
   non-canonical values.** Worth confirming the text format doesn't
   *write* `bits=`/`seed=` fields on output from `format_text` —
   otherwise a round-trip `format_text → parse_text` could succeed
   on today's canonical defaults but silently carry a stale surface
   expectation forward.

### Questions for coder-1

1. **Does `format_text` still emit `bits=`/`seed=` in its output?**
   If yes, the text surface is carrying info the binary surface has
   dropped; if no, the two surfaces match.
2. **Is there an existing pg_test asserting a `tqvector` value round-
   trips through binary I/O (send/recv) cleanly on the new compact
   layout?** The encoder round-trip test covers produce-then-persist;
   send/recv covers client-wire compatibility. One-line confirmation.
3. **Any leftover references to `bits`/`seed` as per-row metadata in
   docs, error text, or comments in `src/am/{build,scan,insert}.rs`
   that predate this packet?** Grep for `seed.*tqvector` or
   `tqvector.*seed` to be sure.

### Call

Accepted. Good slice. The artifact type is now actually compact
instead of just renamed, and canonical/sibling separation is
locked in by regression. Concerns are documentation-shaped, not
blockers. Packet materially advances the task-16 Quant-fields
landing-checklist items.
