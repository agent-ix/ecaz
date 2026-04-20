#!/usr/bin/env python3
"""Full NFR-003 recall benchmark suite via SQL.

Requires: running PostgreSQL with tqvector extension installed.
Usage: PGDATABASE=tqvector_bench python3 scripts/bench_recall.py
"""

import os
import subprocess
import sys

import numpy as np


PGDATABASE = os.environ.get("PGDATABASE", "tqvector_bench")
N = int(os.environ.get("N", "50000"))
DIM = int(os.environ.get("DIM", "1536"))
BITS = int(os.environ.get("BITS", "4"))
SEED = int(os.environ.get("SEED", "42"))
N_QUERIES = int(os.environ.get("N_QUERIES", "100"))
K = 10


def psql(query: str) -> str:
    result = subprocess.run(
        ["psql", PGDATABASE, "-t", "-A", "-c", query],
        capture_output=True, text=True, check=True,
    )
    return result.stdout.strip()


def generate_unit_vectors(n: int, dim: int, seed: int) -> np.ndarray:
    rng = np.random.default_rng(seed)
    vectors = rng.standard_normal((n, dim)).astype(np.float32)
    norms = np.linalg.norm(vectors, axis=1, keepdims=True)
    return vectors / np.maximum(norms, 1e-10)


def brute_force_top_k(corpus: np.ndarray, queries: np.ndarray, k: int):
    scores = queries @ corpus.T
    indices = np.argsort(-scores, axis=1)[:, :k]
    top_scores = np.take_along_axis(scores, indices, axis=1)
    return indices, top_scores


def recall_at_k(true_indices: np.ndarray, pred_indices: np.ndarray, k: int) -> float:
    hits = sum(
        len(set(true[:k]) & set(pred[:k]))
        for true, pred in zip(true_indices, pred_indices)
    )
    return hits / (len(true_indices) * k)


def ndcg_at_k(true_scores: np.ndarray, pred_indices: np.ndarray,
              all_scores: np.ndarray, k: int) -> float:
    """NDCG@k using true IP scores as relevance."""
    ndcg_sum = 0.0
    for q in range(len(pred_indices)):
        # DCG of predicted
        dcg = 0.0
        for rank in range(min(k, len(pred_indices[q]))):
            idx = pred_indices[q][rank]
            rel = max(0.0, all_scores[q][idx])
            dcg += rel / np.log2(rank + 2)
        # Ideal DCG
        idcg = 0.0
        for rank in range(min(k, len(true_scores[q]))):
            rel = max(0.0, true_scores[q][rank])
            idcg += rel / np.log2(rank + 2)
        ndcg_sum += dcg / max(idcg, 1e-10)
    return ndcg_sum / len(pred_indices)


def query_tqvector(query_vec: np.ndarray, m: int, ef_search: int, k: int) -> list:
    """Run a single k-NN query via SQL and return ordered result IDs."""
    arr_str = ",".join(f"{v:.6f}" for v in query_vec)
    result = psql(f"""
        SET ec_hnsw.ef_search = {ef_search};
        SELECT id FROM bench_encoded
        ORDER BY vec <#> ARRAY[{arr_str}]::real[]
        LIMIT {k};
    """)
    return [int(x) for x in result.split("\n") if x.strip()]


def main():
    print(f"=== NFR-003 Recall Benchmark ===")
    print(f"Corpus: {N} x {DIM}, {BITS}-bit, seed={SEED}")
    print(f"Queries: {N_QUERIES}")
    print()

    # Generate data
    corpus = generate_unit_vectors(N, DIM, SEED)
    queries = generate_unit_vectors(N_QUERIES, DIM, SEED + 1_000_000)

    # Ground truth
    print("Computing ground truth...")
    true_indices, true_scores = brute_force_top_k(corpus, queries, 100)

    # Test configurations
    configs = [
        (8, 128, 0.89),
        (8, 200, 0.93),
        (16, 200, 0.97),
    ]

    print(f"\n{'m':>3} {'ef':>5} {'Recall@10':>10} {'Target':>8} {'Pass':>5}")
    print("-" * 40)

    for m, ef_search, target in configs:
        pred_indices = np.zeros((N_QUERIES, K), dtype=int)
        for q_idx in range(N_QUERIES):
            result_ids = query_tqvector(queries[q_idx], m, ef_search, K)
            for r_idx, rid in enumerate(result_ids[:K]):
                pred_indices[q_idx, r_idx] = rid

        r10 = recall_at_k(true_indices, pred_indices, K)
        passed = "YES" if r10 >= target else "NO"
        print(f"{m:>3} {ef_search:>5} {r10:>9.2%} {target:>7.0%} {passed:>5}")

    print("\nDone.")


if __name__ == "__main__":
    main()
