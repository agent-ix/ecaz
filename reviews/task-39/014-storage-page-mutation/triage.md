# Storage Page Mutation Triage

Task: `plan/tasks/39-test-quality-measurement.md`

Target: `src/storage/page.rs`

## Initial Survivors

The first bounded run found 9 missed mutants, all in alignment helper
arithmetic:

| Mutant | Verdict | Resolution |
| --- | --- | --- |
| `align_up`: replace `%` with `/` | Killed by new test | `page_alignment_helpers_preserve_exact_and_round_up_cases` asserts `align_up(17, 8) == 24`. |
| `align_up`: replace `==` with `!=` | Killed by new test | Same test asserts both exact-aligned and round-up cases. |
| `align_up`: replace `+` with `*` | Killed by new test | Same test rejects the incorrect rounded value. |
| `align_up`: replace `-` with `+` | Killed by new test | Same test rejects the incorrect rounded value. |
| `align_up`: replace `-` with `/` | Killed by new test | Same test rejects the incorrect rounded value. |
| `aligned_tuple_bytes`: replace `==` with `!=` | Killed by new test | Same test asserts `aligned_tuple_bytes(8) == 16`. |
| `aligned_tuple_bytes`: replace `+` with `-` | Killed by new test | Same test asserts `aligned_tuple_bytes(9) == 24`. |
| `aligned_tuple_bytes`: replace `-` with `+` | Killed by new test | Same test rejects the incorrect rounded value. |
| `aligned_tuple_bytes`: replace `-` with `/` | Killed by new test | Same test rejects the incorrect rounded value. |

## Rerun Result

The rerun has no survivors:

```text
88 mutants tested in 2m: 81 caught, 7 unviable
```

`rerun/page.rs.mutants/mutants.out/missed.txt` is empty.
