## Feedback: ecvector canonical row model — ACCEPTED, with naming follow-on already scoped as 443

Verified against:

- commits `40659e6` (replace tqvector rows with ecvector and ecqvector),
  `10495af` (remove tqvector compatibility alias surface)
- `sql/bootstrap.sql`: `ecvector` + `ecvector_ip_ops` declared; prior
  canonical public `tqvector` surface and `encode_to_tqvector(...)`
  helper removed at this head
- `src/am/source.rs`: `IndexedVectorKind` distinguishes raw indexed
  `ecvector` from indexed quantized sibling
- `src/am/build.rs`, `src/am/insert.rs`, `src/am/scan.rs`,
  `src/am/vacuum.rs`: indexed-column fallback resolves to `ecvector`

### What's right

- **Productizes packet `441` as a type, not a recipe.** The canonical
  row column is now raw-f32 via `ecvector(dim)`. No user has to
  `ALTER COLUMN … SET STORAGE PLAIN` to get the inline win that
  packet `441` measured. Implicit in the type, explicit in the
  wire format. This is the right shape.
- **Quantization moved out of the row type.** Previously a user had
  two uncomfortable options (store the quantized main column and
  manage a raw sibling, or store raw and duplicate build source).
  Post-442 the canonical row is raw, quantization is an index/
  runtime concern, and explicit quantized artifacts live in a
  separate sibling type. That matches the product's actual
  architecture.
- **Indexed-column fallback is coherent across the four AM paths.**
  Build, insert, scan, and vacuum all use the same resolution rule
  (indexed `ecvector` is the default raw source when no alternate
  source column is configured). One rule, four call sites, single
  point of truth — makes the contract reviewable.
- **Old SQL surface removed, not aliased.** `tqvector` public type
  and `encode_to_tqvector(...)` are gone at this head. No
  compatibility shim. That matches the project's "no deprecated
  names kept around" posture and avoids a doc/maintenance tax.
  The `ecqvector` sibling name chosen here is transitional; see
  §Concerns 1.
- **Test SQL updated alongside.** Fixtures that still intentionally
  exercise the quantized persisted artifact now use the sibling
  type and its encoder, so the quantized-path regression coverage
  isn't dropped — it is just explicit instead of implicit.

### Concerns

1. **`ecqvector` sibling name is too generic.** The persisted
   quantized artifact *is* the TurboQuant-family wire format. A
   future PqFastScan-family or OPQ-family artifact would need a
   different layout, and "ecqvector" gives no indication which
   family a given artifact row belongs to. Packet `443` already
   addresses this by renaming the sibling back to `tqvector` (with
   narrowed scope). Good — but `442` shipping first means head
   briefly carried a name that wasn't stable. Worth tightening the
   submission chain in future so naming decisions land before the
   rename surfaces in commits.
2. **Sibling-type containment contract is implicit, not asserted.**
   The resolution layer *should* never fall back from indexed
   `ecvector` to a sibling artifact, but there's no pg_test that
   locks this in. A future refactor of `source.rs` could re-introduce
   a fallback path without any test catching it. Tracked in the
   task-16 plan landing checklist.
3. **Green checkpoint covers `cargo test` + pg17 + clippy, but no
   recall-parity cell against prior canonical-tqvector head.**
   Because the previous public type is gone, a straight
   before/after recall comparison is not possible on current
   head. The packet-`441` measurement contract in ADR-043
   §Validation is the right forward-looking target (reproduce
   `~3.1ms` q200 on an `ecvector` column without the `STORAGE PLAIN`
   ritual). Worth calling that out explicitly as the head-to-head
   measurement surface for task-16 closure.
4. **No mention of how existing deployments migrate.** The repo's
   posture is "no deprecated names kept around", so there is no
   alias bridge — users rebuild. For an unreleased product that is
   fine; if `tqvector` had ever shipped to external users, this
   would be different. Worth a one-line confirmation that this is
   pre-release territory so the no-bridge posture is the right call.

### Questions for coder-1

1. **Does any non-test code path still mention the old public
   `tqvector` type by name?** Grep should return nothing in `src/`
   outside possibly error-text historical comments. Worth a one-line
   confirmation.
2. **Is `IndexedVectorKind`'s discriminant actively consulted at
   every scan path, or are there paths that assume one kind?**
   Specifically the heap-f32 rerank resolution — does it check the
   kind before selecting `ecvector` as the source?
3. **What happens if a user creates an index on a `tqvector`
   (sibling) column?** Is that rejected at build time, or is it
   silently accepted as an artifact-backed index? A reject + clear
   error is probably the right posture for v1.

### Call

Accepted. Correct architectural correction — canonical row is raw,
quantized artifacts are a separate sibling surface, indexed-column
resolution falls back to the canonical raw column. The `ecqvector`
sibling name was transitional; packet `443` corrects that to
`tqvector` (narrowed). Concerns `2` and `3` land as task-16 plan
items under the new "Quant fields" landing-checklist subsection.
