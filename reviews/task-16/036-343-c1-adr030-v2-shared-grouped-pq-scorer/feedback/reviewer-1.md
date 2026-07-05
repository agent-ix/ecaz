## Feedback: ADR-030 v2 Shared Grouped PQ Scorer

Read `src/quant/grouped_pq.rs` (verified: `pack_grouped_pq_nibbles`,
`grouped_pq_nibble`, `grouped_pq_score_f32`, plus three unit tests).

### What's right

- `grouped_pq_nibble` is the right inverse of `pack_grouped_pq_nibbles`. Even-group
  reads low nibble; odd-group reads high nibble. Symmetric with the packer. Dedicated
  even/odd nibble-reading test exists.
- `grouped_pq_score_f32` is the reference scalar implementation of the LUT
  aggregation. Clear, small, no SIMD tricks. This is the right reference for the
  eventual vpshufb runtime scorer to be checked against.
- Routing the study harness through the shared helper means the 311 feasibility
  numbers are now tied to the same aggregation code that will be used at runtime.
  No study/runtime divergence on the LUT arithmetic.

### Concerns

1. **Score shape is scalar f32.** The eventual runtime scorer will be SIMD — vpshufb
   LUT lookups for 4-bit indices. `grouped_pq_score_f32` is the reference but not
   the implementation the inner loop will use. When the SIMD scorer lands, there
   needs to be a test that `grouped_pq_score_f32` and the SIMD scorer agree to
   within rounding on the same input. The shared helper gives you the reference
   side of that comparison, which is good. Plan for the SIMD-vs-reference test.

2. **LUT layout.** `grouped_pq_score_f32` assumes `lut_f32[group_index * 16 +
   centroid_index]`. That's row-major with group as outer dimension. For vpshufb
   the LUT needs to be packed into 128-bit lanes with interleaved structure. The
   runtime LUT preparation code will have to adapt. Worth documenting the current
   LUT layout in a comment on `grouped_pq_score_f32` so the runtime side has a
   stable reference to match.

3. **Bounds check at `lut_f32[group_index * 16 + centroid_index]`.** No explicit
   bounds check, but `centroid_index < 16` is ensured by `grouped_pq_nibble`
   (4-bit) and `group_count` is caller-provided. A malformed call with
   `group_count` exceeding `packed_nibbles.len() * 2` would panic on OOB read.
   Fine — the assertion is that callers provide matched inputs. Worth a debug-
   assert that `packed_nibbles.len() >= group_count.div_ceil(2)` to catch caller
   mistakes loudly.

### Observation

The packet title says "shared grouped PQ scorer" but what actually shipped is the
reference scalar scorer and packed-code decode primitive. That's the right
sequencing: reference first, SIMD second. The SIMD scorer packet is probably the
next-next step (after the approximate scorer is wired through the runtime helper).

### Cross-cutting status update

With 343 landing:

- **Encoder contract:** shared (packet 336)
- **Insert safety:** gated (packet 337)
- **Vacuum safety:** gated (packet 338)
- **Cold rerank fetch:** wired (packets 339, 340)
- **Metadata validation:** strict (packet 341)
- **Exact rerank scoring:** implemented (packet 342)
- **Shared grouped PQ reference scorer:** landed (packet 343)
- **Still open:** grouped approximate scorer runtime integration, end-to-end recall
  measurement, SIMD scorer.

Nearly there on the pre-scorer runway. The next one or two packets should be the
grouped approximate scorer itself, followed by the first end-to-end measurement
that justifies lifting the gate.

### Code-level nit still open from earlier feedback

`src/am/graph.rs:183` — `GraphTupleRef::binary_word_count()` on `GroupedHot` still
calls `collect_binary_words().len()`, allocating a Vec. Close this before the
scorer starts running on hot paths. Flagged on packet 324, still present.
