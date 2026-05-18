# Manifest: Task 41 Unsafe Surface Strategy

- head SHA: `7df79b8cf70a439e3ad550ff68ab74a14361462f`
- packet/topic: `31161-c1-task41-unsafe-surface-strategy`
- timestamp: `2026-05-16T23:57:56Z`
- isolated one-index-per-table or shared-table surfaces: not applicable; this
  packet is a source/baseline survey, not a PostgreSQL execution lane.

## Artifacts

### baseline-report.log

- lane / fixture / storage format / rerank mode: unsafe-comment baseline source
  survey; no fixture/storage/rerank.
- command used: `make unsafe-baseline-report`
- key cited result lines:
  - `entries: 4579`
  - `files: 106`
  - `3606 src/am`
  - `539 src/tests`
  - `366 src`
  - `68 src/quant`

### strategy-matrix.md

- lane / fixture / storage format / rerank mode: source-pattern strategy
  synthesis; no fixture/storage/rerank.
- commands used:
  - `rg -n "open_valid_ec_.*_index\\(" src/lib.rs src/tests src/am`
  - `rg -c "unsafe fn|unsafe \\{" src/am src/lib.rs src/quant src/tests`
  - `rg -c "from_raw_parts|from_raw_parts_mut|std::slice|ptr::|\\.add\\(|read_unaligned|write_unaligned|BufferGet|PageGet|GenericXLog|ReleaseBuffer|ReadBuffer|LockBuffer|LWLock|SpinLock|atomic|palloc|pfree|RegisterSnapshot|UnregisterSnapshot|table_open|table_close|relation_open|relation_close|index_open|index_close|SPI_|Spi::connect|libpq|PQ" src/am src/lib.rs src/quant src/tests`
- key cited result lines:
  - remaining `open_valid_ec_*_index` callers are concentrated in `src/lib.rs`
    and test debug exports.
  - top unsafe block/function counts include `src/am/ec_hnsw/scan_debug.rs`,
    `src/lib.rs`, `src/am/ec_hnsw/scan.rs`, `src/am/ec_hnsw/build_parallel.rs`,
    and `src/am/ec_ivf/page.rs`.
  - PG resource/pointer-pattern counts are highest in `src/lib.rs`,
    `src/am/ec_hnsw/scan_debug.rs`, `src/am/ec_ivf/page.rs`, remote-search
    tests, and AM scan/build/insert modules.
