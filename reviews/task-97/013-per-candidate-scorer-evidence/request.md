# Task 97 Packet 013: Per-Candidate Scorer Evidence

This packet adds the missing local Criterion row for packet 004 F1's production per-candidate QJL scorer concern.

Code checkpoint: `8742d7f2bca185262889038628ead2756c120da9`

Change:

- Added `1024d/4-bit` to `benches/criterion/quant_score.rs` for the existing `quant/score_ip_from_parts` group.

Local validation:

- `cargo fmt --check`
- `git diff --check`
- `cargo bench --features bench --bench quant_score 'quant/score_ip_from_parts/d1024_b4' -- --sample-size 10 --warm-up-time 1 --measurement-time 2`

Result:

- `quant/score_ip_from_parts/d1024_b4/1024`: `[874.53 ns 887.34 ns 904.33 ns]`

No GitHub CI or AWS runs were used.

## Reviewer Notes

This is a current-head evidence packet, not a kernel optimization. It does not fully close packet 004 F1 by itself because it does not include the old pre-`b0efa19d9` multi-accumulator comparison. It gives the reviewer a durable current row to use with packet 011's scoring-ladder evidence when deciding whether Task 97 proceeds to a separate qjl32 AVX2 optimization slice or accepts a stop-condition disposition.
