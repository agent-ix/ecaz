#!/usr/bin/env python3
"""Generate synthetic vector datasets for tqvector benchmarks.

Usage:
    python3 gen_synthetic_data.py --n 50000 --dim 1536 --seed 42 > corpus.csv
    python3 gen_synthetic_data.py --n 100  --dim 1536 --seed 999 --format query > queries.csv
"""

import argparse
import sys

import numpy as np


def generate_unit_vectors(n: int, dim: int, seed: int) -> np.ndarray:
    rng = np.random.default_rng(seed)
    vectors = rng.standard_normal((n, dim)).astype(np.float32)
    norms = np.linalg.norm(vectors, axis=1, keepdims=True)
    return vectors / np.maximum(norms, 1e-10)


def main():
    parser = argparse.ArgumentParser(description="Generate synthetic vector data")
    parser.add_argument("--n", type=int, required=True, help="Number of vectors")
    parser.add_argument("--dim", type=int, default=1536, help="Vector dimension")
    parser.add_argument("--seed", type=int, default=42, help="Random seed")
    parser.add_argument(
        "--format",
        choices=["corpus", "query"],
        default="corpus",
        help="Output format: corpus (id,embedding CSV) or query (embedding-only CSV)",
    )
    args = parser.parse_args()

    vectors = generate_unit_vectors(args.n, args.dim, args.seed)

    for i, vec in enumerate(vectors):
        arr_str = "{" + ",".join(f"{v:.6f}" for v in vec) + "}"
        if args.format == "corpus":
            print(f"{i},{arr_str}")
        else:
            print(",".join(f"{v:.6f}" for v in vec))


if __name__ == "__main__":
    main()
