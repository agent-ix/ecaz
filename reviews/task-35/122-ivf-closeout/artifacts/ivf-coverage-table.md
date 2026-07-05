# IVF Task 35 Production Coverage

This table summarizes the Task 35 packets that cleared
`src/am/ec_ivf` production-source unsafe-comment baseline entries.

| Packet | Surface | File(s) | Baseline Movement |
|---|---|---|---|
| 024 | admin / cost / options bundle | `admin.rs`, `cost.rs`, `options.rs` | 22 → 0 |
| 025 | page read traversal | `page.rs` | 134 → 121 (−13) |
| 026 | page read stream | `page.rs` | 121 → 100 (−21) |
| 027 | page buffer visitor | `page.rs` | 100 → 91  (−9)  |
| 028 | page append range | `page.rs` | 91  → 81  (−10) |
| 029 | page append mutation + WAL | `page.rs` | 81  → 64  (−17) |
| 030 | page tuple rewrite | `page.rs` | 64  → 45  (−19) |
| 031 | page debug wrapper | `page.rs` | 45  → 38  (−7)  |
| 032 | page exclusive rewrite | `page.rs` | 38  → 31  (−7)  |
| 033 | page tuple read | `page.rs` | 31  → 25  (−6)  |
| 034 | page helper | `page.rs` | 25  → 20  (−5)  |
| 035 | page metadata + series closeout | `page.rs` | 20  → 0   (−20) |
| 036 | scan callback state | `scan.rs` | 101 → 90  (−11) |
| 037 | scan allocation | `scan.rs` | 90  → 69  (−21) |
| 038 | scan rerank probe | `scan.rs` | 69  → 43  (−26) |
| 039 | scan debug tail | `scan.rs` | 43  → 0   (−43) |
| 040 | vacuum | `vacuum.rs` | 26 → 0  |
| 041 | insert | `insert.rs` | 21 → 0  |
| 042 | build | `build.rs` | 23 → 0  |

## Per-Wave Subtotals

| Wave | Packets | Surface | Cleared |
|---|---|---|---|
| Small files bundle | 024 | admin + cost + options | 22 |
| Page series | 025–035 | `page.rs` | 134 (133 + 1 absorbed drift) |
| Scan series | 036–039 | `scan.rs` | 101 |
| Maintenance | 040, 041, 042 | vacuum + insert + build | 70 |
| **Total** | **19 packets** | **`src/am/ec_ivf/*`** | **326** (327 with drift) |

## Drift Note

Packet 022 (Hadamard SIMD) absorbed a `+1` line-drift artifact on
`src/am/ec_ivf/page.rs` (133 → 134) from upstream churn. The page
series then reduced 134 → 0; the real pre-drift page.rs reduction
is 133 → 0. Total IVF baseline cleared from the pre-drift count is
**326** (the headline figure); under absorbed-drift accounting the
page series shows −134 and the total reads as **327**.

## Current Residual

- `src/am/ec_ivf`: `0` unsafe-comment baseline entries.
- `src/tests/ec_ivf.rs`: `0` entries (cleared in packet 108 via the
  `ec_ivf_debug!` macro consolidation).
- Global baseline at IVF closeout: `0` (Task 35 complete).

## Notes

- IVF closed before the closeout template (083 SPIRE) was established;
  this retroactive packet supplies the missing summary.
- The page series (025–035) is the second-largest single-file series
  in Task 35 (11 packets, 134 entries) and was the template for the
  layered-slicing strategy reviewer feedback later asked for on
  larger files.
- The scan series (036–039) closed `scan.rs` in 4 packets matching
  the same architectural-seam slicing pattern.
- The cost callback in 024 was the first AM cost-callback site to be
  documented; its template (PG18 callback role + null-check +
  `pgrx_extern_c_guard` unwind contract) was reused across SPIRE 044
  (`cost/mod.rs`), DiskANN 062 (`cost.rs`), and HNSW.
