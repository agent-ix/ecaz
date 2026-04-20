#!/usr/bin/env python3
"""Unit tests for the real-corpus loader's AM-generic plumbing.

These tests cover the naming, reloption, and profile-dispatch helpers behind
``scripts/load_real_corpus.py``. The loader supports coexisting storage
formats (``turboquant`` / ``pq_fastscan``) on ec_hnsw and is generic across
access-method profiles (ec_hnsw today, ec_diskann in progress).

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


def _hnsw_reloptions(m, ef_construction, storage_format):
    return load_real_corpus._format_hnsw_reloptions(
        m, ef_construction, storage_format, load_real_corpus.DEFAULT_BUILD_SOURCE_COLUMN
    )


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

    def test_hnsw_reloptions_include_storage_format_when_requested(self) -> None:
        self.assertEqual(
            load_real_corpus._expected_index_reloptions(_hnsw_reloptions(8, 128, None)),
            ["m=8", "ef_construction=128", "build_source_column=source"],
        )
        self.assertEqual(
            load_real_corpus._expected_index_reloptions(
                _hnsw_reloptions(8, 128, "pq_fastscan")
            ),
            [
                "m=8",
                "ef_construction=128",
                "build_source_column=source",
                "storage_format=pq_fastscan",
            ],
        )

    def test_build_index_sql_hnsw_only_emits_storage_format_when_requested(self) -> None:
        hnsw = load_real_corpus._resolve_index_profile("ec_hnsw")
        default_sql = load_real_corpus._build_index_sql(
            "ec_hnsw_real_50k_corpus",
            "ec_hnsw_real_50k_m8_idx",
            hnsw.access_method,
            hnsw.operator_class,
            _hnsw_reloptions(8, 128, None),
        )
        self.assertIn("USING ec_hnsw (embedding ecvector_ip_ops)", default_sql)
        self.assertIn("build_source_column = 'source'", default_sql)
        self.assertNotIn("storage_format =", default_sql)

        pq_fastscan_sql = load_real_corpus._build_index_sql(
            "ec_hnsw_real_50k_corpus",
            "ec_hnsw_real_50k_pq_fastscan_m8_idx",
            hnsw.access_method,
            hnsw.operator_class,
            _hnsw_reloptions(8, 128, "pq_fastscan"),
        )
        self.assertIn("storage_format = 'pq_fastscan'", pq_fastscan_sql)

    def test_build_index_sql_diskann_uses_diskann_opclass_and_no_default_reloptions(self) -> None:
        diskann = load_real_corpus._resolve_index_profile("ec_diskann")
        sql = load_real_corpus._build_index_sql(
            "ec_diskann_real_10k_corpus",
            "ec_diskann_real_10k_idx",
            diskann.access_method,
            diskann.operator_class,
            [],
        )
        self.assertIn(
            "USING ec_diskann (embedding ecvector_diskann_ip_ops)", sql
        )
        self.assertNotIn("WITH (", sql)

    def test_build_index_sql_quotes_string_reloptions_but_not_numerics(self) -> None:
        diskann = load_real_corpus._resolve_index_profile("ec_diskann")
        sql = load_real_corpus._build_index_sql(
            "ec_diskann_real_10k_corpus",
            "ec_diskann_real_10k_idx",
            diskann.access_method,
            diskann.operator_class,
            ["graph_degree=48", "alpha=1.2", "storage_format=pq_fastscan"],
        )
        self.assertIn("graph_degree = 48", sql)
        self.assertIn("alpha = 1.2", sql)
        self.assertIn("storage_format = 'pq_fastscan'", sql)

    def test_invalid_storage_format_is_rejected(self) -> None:
        with self.assertRaisesRegex(Exception, "must be one of"):
            load_real_corpus._validate_storage_format("grouped_v2")

    def test_invalid_index_profile_is_rejected(self) -> None:
        with self.assertRaisesRegex(Exception, "must be one of"):
            load_real_corpus._validate_index_profile("ec_bogus")

    def test_reloption_validator_requires_key_value(self) -> None:
        self.assertEqual(load_real_corpus._validate_reloption("graph_degree=48"), "graph_degree=48")
        with self.assertRaisesRegex(Exception, "key=value"):
            load_real_corpus._validate_reloption("graph_degree")
        with self.assertRaisesRegex(Exception, "key=value"):
            load_real_corpus._validate_reloption("=48")

    def test_profile_metadata_is_consistent_with_extension_sql(self) -> None:
        hnsw = load_real_corpus._resolve_index_profile("ec_hnsw")
        self.assertEqual(hnsw.access_method, "ec_hnsw")
        self.assertEqual(hnsw.operator_class, "ecvector_ip_ops")
        self.assertEqual(hnsw.embedding_type, "ecvector")
        self.assertEqual(hnsw.encoder_function, "encode_to_ecvector")
        self.assertTrue(hnsw.supports_build_source_column)
        self.assertTrue(hnsw.supports_m_sweep)

        diskann = load_real_corpus._resolve_index_profile("ec_diskann")
        self.assertEqual(diskann.access_method, "ec_diskann")
        self.assertEqual(diskann.operator_class, "ecvector_diskann_ip_ops")
        self.assertEqual(diskann.embedding_type, "ecvector")
        self.assertEqual(diskann.encoder_function, "encode_to_ecvector")
        self.assertFalse(diskann.supports_build_source_column)
        self.assertFalse(diskann.supports_m_sweep)


if __name__ == "__main__":
    unittest.main()
