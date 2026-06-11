# Task 103 Packet 002: int8_approx32 AVX2 Kernel (AC1)

Code under review: commit `248472ea2` — the real AVX2 backend for the
int8_approx32 family, replacing the scalar-delegating placeholder.

## What changed

`src/quant/int8_approx32/avx2.rs`: per-candidate kernel processing 64
dims per iteration — `vpshufb` codebook dequant from the nibble-split
packed codes, byte re-interleave (unpack + cross-lane permute) back
into natural dim order so the rotated query loads stay contiguous,
sign-extend to i16 + `vpmaddwd` exact pair sums into i32 lanes,
horizontal reduce, scalar dim-tail identical to the reference.

**Deliberate deviation from the Task 98 deferral note**: `vpmaddubsw`
is not used. Its i16 pair sums saturate, and at the ±128 corner
(128×128×2 = 32768 > i16::MAX) that silently breaks the family's
integer-exact contract. The widen-then-`vpmaddwd` path is exact for
all i8 inputs. A new test (`block32_is_bit_equal_at_extreme_i8_values`)
pins exactly this corner; a second new test covers synthetic small-dim
tails (7/64/100/191 — the no-QJL quantizer prep only exists at 1536).

## Results (manifest has full tables)

- **AC1 gate: 88.6 ns/candidate at `isa=avx2`, `scalar_candidates=0` —
  10.4× the packet-001 scalar anchor (919 ns/c)**; gate was ≥2×.
- Recall byte-equal batch-on vs batch-off (0.6230 / 0.9319, all
  percentiles identical).
- End-to-end p50: batch-on 3.63 / 5.37 ms vs batch-off 4.14 / 6.34 ms
  (ef 80 / 160) — batch-on flips from losing (packet 001) to winning
  by 12–15%, and int8_approx becomes the fastest measured HNSW exact
  mode on this host (full_lut: 4.52 / 6.76 ms).
- Fresh release install at this head, restart, `release` probe; suite
  audit passed 4/4. Focused tests + clippy logs in artifacts; no
  pg_tests ran.

## Review focus

1. The saturation argument for rejecting `vpmaddubsw` (and whether the
   extreme-values test convinces you the contract holds).
2. The byte re-interleave in the kernel (unpack works per 128-bit
   lane; the two `permute2x128` ops restore contiguous halves) —
   correctness is covered by parity tests, but the lane bookkeeping
   deserves a second pair of eyes.
3. Whether citing packet 001's 919 ns/c as the same-head scalar anchor
   is acceptable (this commit does not touch the scalar path; only
   `avx2.rs` and tests changed).

Remaining Task 103 slice after this packet: rabitq32 AVX2 validation +
bench (AC4).
