#!/usr/bin/env python3
"""Derive latency stats + recall@k from sweep.sh output.

Reads `<sweep_dir>/<size>/_groundtruth.out` plus every other `*.out`
under that size, parses the `INFO: SAMPLE qid=N ms=F ids={...}` lines,
and writes per-cell `latency.log` + `recall.txt`. Emits an aggregate
Pareto table `<sweep_dir>/<size>/_pareto.tsv`.

Usage: compute_recall.py <sweep_dir> [<size> [<size> ...]]
       (defaults to every size subdirectory under <sweep_dir>)
"""
import re
import statistics
import sys
from pathlib import Path


SAMPLE_RE = re.compile(
    r"INFO:\s+(?:SAMPLE|GT)\s+qid=(\d+)\s+ms=([\d.]+)\s+ids=\{([\d,]+)\}"
)


def parse_out(path: Path):
    rows = []
    with path.open() as f:
        for line in f:
            m = SAMPLE_RE.search(line)
            if m:
                rows.append((int(m.group(1)),
                             float(m.group(2)),
                             [int(x) for x in m.group(3).split(",") if x]))
    return rows


def latency_stats(rows):
    if not rows:
        return None
    ms = sorted(r[1] for r in rows)
    def pct(p): return ms[max(0, int(round(p / 100.0 * (len(ms) - 1))))]
    return {"n": len(ms), "mean": statistics.mean(ms), "min": min(ms),
            "p50": pct(50), "p95": pct(95), "p99": pct(99), "max": max(ms)}


def recall_at_k(approx, gt, k=10):
    gt_by = {q: set(ids[:k]) for q, _, ids in gt if ids}
    if not gt_by: return None
    rs = [len(set(ids[:k]) & gt_by[q]) / len(gt_by[q])
          for q, _, ids in approx if q in gt_by and ids]
    return sum(rs) / len(rs) if rs else None


def write_latency_log(path: Path, s, k):
    path.write_text(
        f"# iterations: {s['n']}, k: {k}\n"
        f"# mean: {s['mean']:.3f} ms\n"
        f"# min: {s['min']:.3f} ms\n"
        f"# p50: {s['p50']:.3f} ms\n"
        f"# p95: {s['p95']:.3f} ms\n"
        f"# p99: {s['p99']:.3f} ms\n"
        f"# max: {s['max']:.3f} ms\n"
    )


def process_size(size_dir: Path, k: int = 10):
    gt_path = size_dir / "_groundtruth.out"
    if not gt_path.exists():
        print(f"[skip] {size_dir}: no _groundtruth.out", file=sys.stderr)
        return
    gt = parse_out(gt_path)
    if not gt:
        print(f"[skip] {size_dir}: empty ground truth", file=sys.stderr)
        return
    print(f"[gt {size_dir.name}] n={len(gt)} "
          f"p50={latency_stats(gt)['p50']:.1f}ms")

    pareto = [("system", "variant", "setting", "p50_ms", "p95_ms",
               "recall_at_10", "n")]
    for cell in sorted(size_dir.rglob("*.out")):
        if cell.name == "_groundtruth.out":
            continue
        # cell path: <size>/<system>/<variant>/<setting>.out
        rel = cell.relative_to(size_dir)
        parts = rel.parts
        if len(parts) != 3:
            continue
        system, variant, fname = parts
        setting = fname[:-4]

        rows = parse_out(cell)
        s = latency_stats(rows)
        if not s:
            print(f"[skip] {rel}: no SAMPLE rows", file=sys.stderr)
            continue
        write_latency_log(cell.with_name(f"{setting}.latency.log"), s, k)
        r = recall_at_k(rows, gt, k=k)
        recall_path = cell.with_name(f"{setting}.recall.txt")
        recall_path.write_text(
            f"recall@{k}: {r:.4f}\n" if r is not None else f"recall@{k}: n/a\n"
        )
        pareto.append((system, variant, setting,
                       f"{s['p50']:.3f}", f"{s['p95']:.3f}",
                       f"{r:.4f}" if r is not None else "nan",
                       str(s['n'])))
        print(f"  [{system}/{variant}/{setting}] "
              f"p50={s['p50']:.2f}ms p95={s['p95']:.2f}ms "
              f"recall@{k}={'%.4f' % r if r is not None else 'n/a'}")

    pareto_path = size_dir / "_pareto.tsv"
    pareto_path.write_text("\n".join("\t".join(row) for row in pareto) + "\n")
    print(f"  wrote {pareto_path}")


def main(argv):
    if len(argv) < 2:
        print(__doc__, file=sys.stderr)
        sys.exit(2)
    sweep_dir = Path(argv[1])
    sizes = argv[2:] or sorted(p.name for p in sweep_dir.iterdir()
                               if p.is_dir())
    for size in sizes:
        process_size(sweep_dir / size)


if __name__ == "__main__":
    main(sys.argv)
