# Task 102 Packet 002: NEON Repack Port, DiskANN TQ Batching, Quantized-LUT Deferral

Code checkpoints under review:

- `90a221a20` — `Port lut32 NEON kernel to shuffle-repack; add SVE live-width tails`
- `2f99971c5` — `Route DiskANN TurboQuant prefilter batches through the lut32 kernels`
- Task-file edit (this packet's commit): quantized-LUT deferral recorded in
  `plan/tasks/99-cross-am-quant-isa-block-kernel-closeout.md`

Motivation for all three: the Graviton 4 evidence pass is paid and should
run once, against final shapes. Packet 001 feedback approved the AVX2 slice
with the G4 pass as the open follow-up; this packet removes the three known
ways that pass could be invalidated or forced into a second trip.

## 1. NEON shuffle-repack port (supersedes the packet-001 NEON shape)

The NEON kernel the packet 001 reviewer statically accepted still extracted
nibbles scalar per quad into a stack array — the same store-to-load
forwarding pattern that made the AVX2 v1 kernel **slower than the scalar
block** on Intel (1,371 vs 1,054 ns/candidate, recorded in packet 001's
"Interim v1 measurement"). Rather than risk discovering that on a paid G4
instance, the kernel now mirrors the measured AVX2 v2 shape:

- 3-pass `vzip` 8×16 byte transpose (identical network and output mapping
  to the AVX2 unpack version; `vzip1q` = `unpacklo`, `vzip2q` = `unpackhi`),
  aarch64-gated transpose unit test included.
- In-vector nibble widening (`vand`/`vshr` → `vmovl` chain) feeding
  `vqtbl4q_u8` selects over the 64-byte per-dim register table — no scalar
  round trips in the scoring loop.
- Octet-granular lane counts (8..=32), `score_octets_neon` entry for tails.

SVE additionally gains a **live-width partial entry**: the existing gather
helper's `whilelt` loop predicates tails natively, so SVE hosts score
exactly the live lanes with no padding at all (`score_partial_sve`, no new
asm). Partial dispatch order: SVE live-width on Sve/Sve2 hosts, octet
padding on AVX2/NEON hosts, scalar fast path for single lanes. Intel
behavior is unchanged; per-lane dim-order accumulation (bit-exact contract)
is preserved everywhere.

## 2. DiskANN × TurboQuant batch registration

Task 91 landed `storage_format=turboquant` for DiskANN, but the prefilter
batch dispatch had no TurboQuant arm — TQ-storage indexes scored through
the `QuantCodec` default per-candidate loop: no kernels, no counters. Fixed
with a batched arm (gated on `ec_diskann.candidate_batch_scoring`, polarity
matching the per-candidate path) plus the `score_ip_batch` codec override,
and a 39-candidate bit-exactness + counter-attribution test.

Local evidence (see `artifacts/manifest.md`; release backend verified by
install SHA + `ecaz_build_profile()` probe + suite preflight):

| Gate | Result |
| --- | --- |
| Direct rows | `surface=diskann quant=turboquant isa=avx2 scalar_candidates=0` (plus single-lane fast-path flushes attributed scalar) |
| Kernel rate | 265–283 ns/candidate, consistent with packet 001's lut32 ladder given mid-width (8–31) DiskANN flushes |
| Recall | byte-equal kernel-on/off (`0.7516` / `0.9726`) |
| End-to-end p50 | kernel-on **−26.3% / −29.8%** vs kernel-off (4.01 vs 5.44 ms; 4.47 vs 6.37 ms) |

With this, the G4 trip can cover the complete quant × AM matrix including
DiskANN × TQ in one pass.

## 3. Quantized-LUT deferral record

`plan/tasks/99-...md` "Absorbed deferrals" now records the operator
decision: the u8 fast-scan lut32 variant is deferred indefinitely — it
breaks the byte-equal recall regime, the lane is no longer
scoring-dominated post-Task-102, and landing it after G4 would invalidate
the paid lut32 ARM evidence. Any revisit must precede an ARM trip.

## Review focus

1. NEON transpose/widening correctness for G4 readiness (cannot execute
   locally; the aarch64 transpose unit test plus the per-ISA parity tests
   are the day-one G4 smoke set).
2. The partial dispatch ordering in `score_lut_no_qjl_4bit_partial`
   (SVE-live-width vs octet-padding preference is a measurement question
   for G4; the ordering is a best guess and cheap to flip there).
3. The DiskANN arm's gating choice (`candidate_batch_scoring`-gated like
   RaBitQ, unlike the ungated GroupedPq arm) and `CandidateMeta::None`.
4. Whether the deferral note's placement in Task 99 (feeding ADR-077) is
   the right durable record.

Remaining for Task 102 closeout after this packet: the G4 pass only.
