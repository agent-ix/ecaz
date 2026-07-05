# Task 103 Packet 003: rabitq32 AVX2 Validation (AC4)

Closes the Task 93 deferral: the rabitq32 AVX2 backend landed in Task
93 packet 005 without ever being compiled/run/benched on an Intel
host. This packet validates it on the local Intel desktop. One small
test-only commit rides along (`f271717aa`); the kernel itself is
untouched.

## Evidence (manifest has full tables)

1. **Parity on AVX2**: the when-available suite (6 tests) passes with
   `Isa::Avx2` asserted — tolerance vs the forced-scalar anchor and
   bit-equality with the production batch slab scorer. AM-level
   dispatch tests (5) confirm routing through the block kernel with
   bit-exact scores.
2. **Counter rows**: `surface=diskann quant=rabitq isa=avx2`,
   `scalar_candidates=0`, **80.4–81.1 ns/candidate** on a new
   synthetic `task103_diskann_rabitq_2k` fixture (bits=1 sidecar; the
   existing IVF rabitq fixtures are bits=4, which the kernel does not
   serve).
3. **Recall byte-equal** kernel-on vs kernel-off (0.5984 / 0.9541,
   identical percentiles); end-to-end within noise both directions.

## Rode-along commit `f271717aa` (test-only)

The two `am::ec_ivf::quantizer` rabitq dispatch tests score through
the real batch path, which mutates the global candidate-batch
counters, but did not take `CANDIDATE_BATCH_COUNTER_TEST_LOCK`. Under
a filter that selects them together with the counter-asserting
`candidate_batch` tests (e.g. `cargo test --lib rabitq_bits1`), the
ivf/rabitq rows double-count and poison the lock. Historical filters
never selected both groups at once, so this never fired before. Fix:
both tests now take the lock, matching every other counter-touching
test.

## Review focus

1. Is the synthetic 2k DiskANN fixture acceptable as the AC4 bench
   surface (rationale: it is the only AM whose rabitq storage is
   bits=1 by construction and whose batch path has a GUC off-switch
   for the A/B; the IVF rabitq batch path is always-on and the local
   IVF fixtures are bits=4)?
2. The test-lock fix — agree it is an isolation gap and not masking a
   counter bug? (Single-threaded run passed before the fix; the
   double-count required concurrent mutation.)

## Task 103 status after this packet

- AC1 int8_approx32 AVX2 kernel: **done** (packet 002, 10.4×).
- AC2 tiled_lut32 disposition: proposed **retire** (packet 001).
- AC3 hamming32 decision: proposed **skip** (packet 001).
- AC4 rabitq32 AVX2 validation: **this packet**.
- AC5 recall byte-equal + no regression: held at every measured cell
  (packets 001–003).
- AC6 no `missing_kernel` Intel cells: satisfied once packets 001–003
  are accepted (real: lut32/qjl32/grouped_pq/int8_approx32/rabitq32;
  documented decisions: tiled_lut32 retire, hamming32 skip).
