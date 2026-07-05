# Mutation triage: `src/quant/simd.rs`

- Command: `cargo mutants --in-place --package ecaz-careful-hardening --file 'hardening/careful/src/../../../src/quant/simd.rs' --output reviews/task-39/005-simd-mutation-triage/artifacts/simd-careful-inplace.mutants`
- Result: 9 mutants tested; 6 caught; 3 unviable; 0 missed; 0 timed out
- Test package: `ecaz-careful-hardening`
- Reason for `--in-place`: local checkout contains large untracked benchmark artifacts, and cargo-mutants copies untracked files when using scratch mode.

## Survivors

No surviving mutants.

## Outcomes

| Mutant | Outcome | Triage verdict |
| --- | --- | --- |
| `has_avx2_fma -> bool with true` | caught | killed by `x86_backend_gate_requires_avx2_and_fma` |
| `has_avx2_fma -> bool with false` | caught | killed by `x86_backend_gate_requires_avx2_and_fma` |
| `replace && with || in has_avx2_fma` | caught | killed by `x86_backend_gate_requires_avx2_and_fma` |
| `forced_backend_from_env -> Option<SimdBackend> with None` | caught | killed by forced backend override tests |
| `backend_name -> &'static str with ""` | caught | killed by `backend_name_matches_detected_backend` |
| `backend_name -> &'static str with "xyzzy"` | caught | killed by `backend_name_matches_detected_backend` |
| `backend -> SimdBackend with Default::default()` | unviable | mutant does not build; `SimdBackend` intentionally has no `Default` |
| `detect_backend -> SimdBackend with Default::default()` | unviable | mutant does not build; `SimdBackend` intentionally has no `Default` |
| `forced_backend_from_env -> Option<SimdBackend> with Some(Default::default())` | unviable | mutant does not build; `SimdBackend` intentionally has no `Default` |

The previous ARM-only blind spot was the x86 `avx2 && fma` expression. This packet extracts that boolean gate into `has_avx2_fma` so the truth table is testable on ARM and the `&&` to `||` mutant is caught without executing AVX2 code.
