# Review request: Task 144 — auto default-path confirmation (packet 002)

- Task: `plan/tasks/144-hnsw-int8-approx-default-revisit.md`
- Follows the packet-001 ACK
  (`reviews/task-144/001-hnsw-scorer-default/feedback/2026-07-03-01-reviewer.md`),
  which approved the flip and asked for "a narrow commit plus a small
  default-path confirmation."
- Head SHA `3f69d74c0` (the flip commit). Evidence:
  `artifacts/manifest.md` + `artifacts/*.log` + `artifacts/results.jsonl`.

## What landed

1. **The narrow commit** is already on main: `3f69d74c0` flips
   `ec_hnsw.turboquant_exact_score_mode` `exact` → `auto`
   (`src/am/ec_hnsw/{options,scan}.rs`).
2. **This packet** confirms the installed default path and closes a landed
   test-expectation regression the flip introduced (details below).

## Results (default path, no session GUCs, m5-local, PG18)

**Auto resolves to int8 on the no-QJL 4-bit lane — recall@k byte-for-byte
identical to packet 001's explicit `int8_approx`, distinct from `exact`:**

| prefix | ef | exact (001) | int8 (001) | **auto (002)** |
|--------|----|-------------|------------|----------------|
| 10k  | 64 | 0.9219 | 0.9203 | **0.9203** |
| 50k  | 64 | 0.9375 | 0.9333 | **0.9333** |
| 100k | 64 | 0.8781 | 0.8750 | **0.8750** |

Match holds at **all 18** cells (10k/50k/100k × ef 40–200). Latency sits in
001's int8 band (100k ef64 mean 1.05 ms); storage unchanged (query-side mode).
Full tables + files in `artifacts/manifest.md`.

## Note on the reviewer's `current_setting` expectation

The ACK expected `current_setting(...)` to report `int8_approx`. It reports
**`auto`** — by design. A literal `int8_approx` default would `pgrx::error!`
on every QJL-active / non-4-bit TQ HNSW scan ("requires the no-QJL 4-bit
lane"); `auto` resolves per scan (`resolve_turboquant_exact_score_mode`,
`scan.rs:1395`). The resolved mode is proven instead:

- **Behavioral:** the byte-identical recall match above (installed default runs
  the int8 kernel on the 1536-dim tables).
- **Fallback smoke:** on a QJL-active dim-64 lane the auto default scans clean
  (exit 0, resolves to exact), while an explicit `int8_approx` GUC on the same
  lane errors — the exact counterfactual a naive literal default would hit.
- **pg_test surface:** new
  `test_turboquant_scan_stage_profile_auto_default_resolves_int8` asserts the
  stage profile reports `int8_approx_no_qjl_4bit` on the no-override lane.

## Landed test-expectation regression fixed here (please review)

The flip changed the resolved default on the 1536-dim no-QJL 4-bit fixture,
so `test_ech_debug_turboquant_scan_stage_profile_sql_surface` (which pinned the
pre-flip `mse_no_qjl_4bit` exact-mode string and an exact-path deferred-rerank
invariant, with no explicit override) would now regress. Fix
(`src/tests/ec_hnsw_runtime_profiles.rs`, `src/tests/mod.rs`):

- Pin that test to `exact` explicitly (`ScopedEnvVar::set(..., "exact")`) so it
  keeps covering the exact-score surface it was written for — no expectation
  guessing.
- Add `ScopedEnvVar::unset` and thread an `Option<&str>` through
  `assert_turboquant_scan_stage_profile_mode` so `None` exercises the resolved
  default; add the auto-default guard test.

**Validation:** `cargo check --no-default-features --features pg18 --tests`
(compile gate). pg_test execution deferred to Linux — macOS pgrx-test is
blocked by the known `dyld _BufferBlocks` issue. No code-path change; the diff
is test-only plus the already-landed flip.

## Caveats

m5-local only (Graviton deferred, operator 2026-07-03). Recall samples
64/48/32 queries at 10k/50k/100k, matching packet 001 for comparability.
Fallback fixture TSVs are regenerable and not committed (recorded in manifest).
