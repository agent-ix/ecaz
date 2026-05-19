# Unsafe Block Count Tooling Plan

Task 50 measures direct unsafe block count, not Task 35 baseline entries.

## Required Make Target

Add a narrow target:

```make
.PHONY: unsafe-block-count

PATHS ?= src

unsafe-block-count:
	@if command -v rg >/dev/null 2>&1; then \
		rg --count-matches 'unsafe\s*\{' $(PATHS); \
	else \
		grep -R -E -c 'unsafe[[:space:]]*\{' $(PATHS); \
	fi | awk -F: '$$2 > 0 {printf "%4d %s\n", $$2, $$1}' | sort -nr
```

The target should print a stable per-file table, sorted descending by count.
It should not read `scripts/unsafe_comment_baseline.txt`.
It must not assume ripgrep is installed; use the `grep -R -E -c` fallback above
or fail with an explicit install message.

## Optional Script Form

If quoting in Make becomes awkward, add:

```text
scripts/unsafe_block_count.sh
```

Contract:

- default root: `src`;
- optional path arguments for packet-scoped counts;
- output format: `<count> <path>` sorted descending;
- nonzero exit only for tool failure, not for finding unsafe blocks.

Suggested command:

```sh
if command -v rg >/dev/null 2>&1; then
  rg --count-matches 'unsafe\s*\{' "$@"
else
  grep -R -E -c 'unsafe[[:space:]]*\{' "$@"
fi | awk -F: '$2 > 0 {printf "%4d %s\n", $2, $1}' | sort -nr
```

## Packet Artifact Convention

Before editing:

```sh
make unsafe-block-count > reviews/task-50/NNN-topic/artifacts/block-count-before.log
```

After editing:

```sh
make unsafe-block-count > reviews/task-50/NNN-topic/artifacts/block-count-after.log
```

For narrow packets, also capture touched-file-only logs:

```sh
make unsafe-block-count PATHS="src/am/ec_ivf/page.rs src/am/ec_ivf/scan.rs"
```

## Top-15 Snapshot At Planning Time

Captured from direct source grep at HEAD `363b13c5f717`:

| Rank | Unsafe blocks | File |
| ---: | ---: | --- |
| 1 | 356 | `src/am/ec_hnsw/scan_debug.rs` |
| 2 | 226 | `src/am/ec_hnsw/scan.rs` |
| 3 | 203 | `src/am/ec_hnsw/build_parallel.rs` |
| 4 | 160 | `src/am/ec_spire/dml_frontdoor/mod.rs` |
| 5 | 134 | `src/am/ec_ivf/page.rs` |
| 6 | 133 | `src/am/ec_hnsw/insert.rs` |
| 7 | 102 | `src/am/ec_ivf/scan.rs` |
| 8 | 99 | `src/am/ec_hnsw/vacuum.rs` |
| 9 | 92 | `src/am/ec_diskann/routine.rs` |
| 10 | 78 | `src/am/ec_hnsw/source.rs` |
| 11 | 73 | `src/am/ec_hnsw/shared.rs` |
| 12 | 71 | `src/am/ec_spire/coordinator/hierarchy_snapshots.rs` |
| 13 | 63 | `src/am/common/parallel.rs` |
| 14 | 62 | `src/quant/hadamard.rs` |
| 15 | 62 | `src/am/ec_spire/coordinator/snapshots.rs` |

Execution should not blindly start at rank 1. The priority bridge is:
SPIRE and IVF/RaBitQ first, then shared helpers, then HNSW/DiskANN density
cleanup.

`top-15-coverage-map.md` reconciles this priority order with the Task 50 exit
criterion by assigning each top-15 file to projected direct or shared-helper
reduction slices.
