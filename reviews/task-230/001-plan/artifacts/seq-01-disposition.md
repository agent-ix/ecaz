# Task 230 packet 001 reviewer seq-01 disposition

Reviewer feedback:
`../feedback/2026-08-28-01-reviewer.md`.

1. **Exact-vector TOAST mechanism — resolved.** Hot `ecvector` is explicitly
   `STORAGE PLAIN`, the hot heap is `fillfactor=100`, validation pins
   `attstorage='p'`, V1 is capped at 1,536 dimensions, and larger or oversized
   tuples fail build preflight without fallback. Section 6 names the one-page
   versus TOAST-fetch mechanism and expected roughly 30% hot-page
   amplification.
2. **Cold locator contradiction — resolved.** The hot tuple has no `cold_tid`.
   Cold and hot insert first; the sole authoritative pair is then written to
   graph V2.
3. **Visibility contradiction — resolved.** Graph current/tombstone state is
   the gate. Hot MVCC and tombstone are integrity echoes; cold-only success does
   not open hot.
4. **`bytea(16)` identity — resolved.** Source identity is mandatory implicit
   hot state, cannot be named as an additional hot scalar, and remains legal in
   both UUID and validated `bytea(16)` forms.
5. **Graph offsets — resolved.** V2 appends `cold_tid` after the variable arrays
   and leaves every V1 header offset unchanged. Versioned size/offset helpers,
   all named raw consumers, the V1-only generation-descriptor predicate, and
   Task 230-before-231 ordering are explicit.
6. **Byte bounds — resolved.** Wire-derived 258-byte and 96-byte claims are
   removed. The plan uses a versioned PG18 native heap estimator, a maximal
   formed-tuple check, total-column NULL bitmap, per-tier TOAST/index/page
   accounting, and a descriptor-bound maximum hot tuple size.
7. **Task 229 prior and primary shape — resolved.** Section 6 carries the two
   100k regressions, predicts each projection direction, and fixes id-only
   end-to-end ANN retrieval as the single primary decision shape before full-
   scale results.

No persisted-format source is submitted with this revision. Packet 002 remains
gated on outside acceptance of this revised contract.
