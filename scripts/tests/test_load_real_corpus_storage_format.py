#!/usr/bin/env python3
"""Unit tests for real-corpus loader storage-format coexistence helpers.

These tests cover the naming and reloption helpers behind
``scripts/load_real_corpus.py --storage-format``. The loader now needs to keep
coexisting ``turboquant`` and ``pq_fastscan`` index families on one staged
corpus without changing the underlying table names.

Run with::

    python3 scripts/tests/test_load_real_corpus_storage_format.py
"""

from __future__ import annotations

import sys
import unittest
from pathlib import Path

_SCRIPTS_DIR = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(_SCRIPTS_DIR))

import load_real_corpus  # noqa: E402 — sys.path tweak above


class LoadRealCorpusStorageFormatTests(unittest.TestCase):
    def test_default_index_prefix_preserves_legacy_names(self) -> None:
        prefix = load_real_corpus._index_prefix("ec_hnsw_real_50k", None)
        self.assertEqual(prefix, "ec_hnsw_real_50k")
        self.assertEqual(load_real_corpus._index_name(prefix, 8), "ec_hnsw_real_50k_m8_idx")

    def test_explicit_storage_format_gets_coexisting_index_prefix(self) -> None:
        turboquant_prefix = load_real_corpus._index_prefix("ec_hnsw_real_50k", "turboquant")
        pq_fastscan_prefix = load_real_corpus._index_prefix(
            "ec_hnsw_real_50k", "pq_fastscan"
        )
        self.assertEqual(turboquant_prefix, "ec_hnsw_real_50k_turboquant")
        self.assertEqual(pq_fastscan_prefix, "ec_hnsw_real_50k_pq_fastscan")
        self.assertEqual(
            load_real_corpus._index_name(pq_fastscan_prefix, 16),
            "ec_hnsw_real_50k_pq_fastscan_m16_idx",
        )

    def test_expected_reloptions_include_storage_format_when_requested(self) -> None:
        self.assertEqual(
            load_real_corpus._expected_index_reloptions(8, 128, None),
            ["m=8", "ef_construction=128", "build_source_column=source"],
        )
        self.assertEqual(
            load_real_corpus._expected_index_reloptions(8, 128, "pq_fastscan"),
            [
                "m=8",
                "ef_construction=128",
                "build_source_column=source",
                "storage_format=pq_fastscan",
            ],
        )

    def test_build_index_sql_only_emits_storage_format_reloption_when_requested(self) -> None:
        default_sql = load_real_corpus._build_index_sql(
            "ec_hnsw_real_50k_corpus",
            "ec_hnsw_real_50k_m8_idx",
            8,
            128,
            None,
        )
        self.assertIn("build_source_column = 'source'", default_sql)
        self.assertNotIn("storage_format =", default_sql)

        pq_fastscan_sql = load_real_corpus._build_index_sql(
            "ec_hnsw_real_50k_corpus",
            "ec_hnsw_real_50k_pq_fastscan_m8_idx",
            8,
            128,
            "pq_fastscan",
        )
        self.assertIn("storage_format = 'pq_fastscan'", pq_fastscan_sql)

    def test_invalid_storage_format_is_rejected(self) -> None:
        with self.assertRaisesRegex(Exception, "must be one of"):
            load_real_corpus._validate_storage_format("grouped_v2")


if __name__ == "__main__":
    unittest.main()
