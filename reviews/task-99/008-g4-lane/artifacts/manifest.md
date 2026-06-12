# Manifest — Task 99 packet 008: Graviton 4 lane (IN PROGRESS)

- Lane: **Graviton 4 production column** — `10k-medium`, db instance
  `i-00bfd67d8b5ed7959` (m8g.2xlarge, Neoverse V2), us-west-2,
  restored from `snap-0e9c7743263e61d70`; `ecaz cloud status` cost line:
  ~$0.346/hr running.
- Git ref installed: `63bf4d78c` (main, post Task-99 merge);
  installed backend `/usr/lib64/pgsql/ecaz.so` sha256 `c785e749…`
  (unchanged across the day-one test runs — bracketed in the logs).
- Database: `tqvector_bench` (snapshot's corpus DB; sources
  `real_100k_ivf_rabitq1_rerank_{corpus,queries}` — raw-f32 embeddings,
  profile-portable per packet 002 §3).

## Day-one gate (PASSED)

- `day1-smoke-attempt1.log`: lut32 11 / qjl32_ 11 / rabitq32 6 passed —
  includes the SVE backends executing for real (`*_sve_*_when_available`
  asserting `Isa::Sve2`); run aborted at a `#[pg_test]` matched by the
  `grouped_pq` filter (pgrx harness permission-denied — see finding 1).
- `day1-smoke2.log`: **SVE vector length = 16 bytes → `sve2-128`**
  (`/proc/sys/abi/sve_default_vector_length`); grouped_pq 34 /
  hamming32 3 / int8_approx32 4 / candidate_batch 19 / quant::isa 8 —
  all passed with `--skip pg_test_`; backend sha unchanged.

## Findings so far

1. **pg_tests compile under plain `cargo test --lib`** and the pgrx
   harness then attempts a debug extension build+install into the
   system PG (failed on permissions here, which protected the release
   backend). On-host test runs MUST pass `--skip pg_test_`. This
   generalizes the local debug-install trap.

## Remaining steps (runbook packet 006)

Fixtures (SSM `941b4894…`, in progress) → main profile suite →
NEON-capped pass → Task 97 suite → snapshot → down.
