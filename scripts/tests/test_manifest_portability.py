#!/usr/bin/env python3
"""Fixture tests for portable `source_parquet_*` manifest fields.

These tests exercise the loader's manifest verifier in
``scripts/load_real_corpus.py`` against three synthetic manifest shapes:

1. A manifest WITH the new portable fields — accepted.
2. A manifest WITHOUT the new portable fields — still accepted (backwards
   compatible, older manifests must keep loading).
3. A manifest WITH the new portable fields pointing at absolute paths —
   rejected with a clear error, because the portable fields must be portable.

The tests stage tiny synthetic corpus/query TSV files under a temporary
directory so the row-count / SHA-256 / first-last-id checks that the
verifier performs unchanged continue to pass; the portable-field behavior
is the only axis under test.

Run with::

    python3 scripts/tests/test_manifest_portability.py
"""

from __future__ import annotations

import hashlib
import json
import os
import sys
import tempfile
import traceback
import unittest
from pathlib import Path

# Allow `import load_real_corpus` when this test file is invoked directly.
_SCRIPTS_DIR = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(_SCRIPTS_DIR))

import load_real_corpus  # noqa: E402 — sys.path tweak above


_PREFIX = "ec_hnsw_real_test"
_DIM = 4


def _write_tsv(path: Path, rows: list[tuple[int, list[float]]]) -> tuple[int, str, int, int]:
    sha = hashlib.sha256()
    first_id: int | None = None
    last_id: int | None = None
    with path.open("w", encoding="utf-8", newline="\n") as handle:
        for row_id, values in rows:
            line = f"{row_id}\t[" + ",".join(repr(float(v)) for v in values) + "]\n"
            if first_id is None:
                first_id = row_id
            last_id = row_id
            handle.write(line)
            sha.update(line.encode("utf-8"))
    assert first_id is not None and last_id is not None
    return len(rows), sha.hexdigest(), first_id, last_id


def _build_base_manifest(
    tmp_dir: Path,
    corpus_rows: list[tuple[int, list[float]]],
    query_rows: list[tuple[int, list[float]]],
) -> tuple[dict, Path, Path, Path]:
    corpus_path = tmp_dir / f"{_PREFIX}_corpus.tsv"
    queries_path = tmp_dir / f"{_PREFIX}_queries.tsv"
    manifest_path = tmp_dir / f"{_PREFIX}_manifest.json"

    corpus_count, corpus_sha, corpus_first, corpus_last = _write_tsv(corpus_path, corpus_rows)
    query_count, query_sha, query_first, query_last = _write_tsv(queries_path, query_rows)

    manifest = {
        "manifest_version": 1,
        "prefix": _PREFIX,
        "source_dataset": "synthetic test dataset",
        "dimension": _DIM,
        "id_column": "_id",
        "vector_column": "embedding",
        "selection_rule": {
            "sort_key": "_id ascending lexicographic",
            "corpus_start": 0,
            "corpus_rows": corpus_count,
            "query_start": corpus_count,
            "query_rows": query_count,
            "output_id_mode": "global_sorted_row_index",
        },
        "corpus": {
            "file": corpus_path.name,
            "rows": corpus_count,
            "sha256": corpus_sha,
            "first_id": corpus_first,
            "last_id": corpus_last,
            "first_source_id": "row-0",
            "last_source_id": f"row-{corpus_count - 1}",
        },
        "queries": {
            "file": queries_path.name,
            "rows": query_count,
            "sha256": query_sha,
            "first_id": query_first,
            "last_id": query_last,
            "first_source_id": f"row-{corpus_count}",
            "last_source_id": f"row-{corpus_count + query_count - 1}",
        },
        "generated_at_utc": "2026-04-10T00:00:00+00:00",
        "generated_by": "scripts/tests/test_manifest_portability.py",
    }
    return manifest, corpus_path, queries_path, manifest_path


def _write_manifest_json(manifest_path: Path, manifest: dict) -> None:
    with manifest_path.open("w", encoding="utf-8") as handle:
        json.dump(manifest, handle, indent=2, sort_keys=True)
        handle.write("\n")


class ManifestPortabilityTests(unittest.TestCase):
    def setUp(self) -> None:
        self._tmp = tempfile.TemporaryDirectory()
        self.tmp_dir = Path(self._tmp.name)
        self.corpus_rows = [
            (0, [0.1, 0.2, 0.3, 0.4]),
            (1, [0.5, 0.6, 0.7, 0.8]),
            (2, [0.9, 1.0, 1.1, 1.2]),
        ]
        self.query_rows = [
            (3, [1.3, 1.4, 1.5, 1.6]),
            (4, [1.7, 1.8, 1.9, 2.0]),
        ]

    def tearDown(self) -> None:
        self._tmp.cleanup()

    def _verify(self, manifest_path: Path, corpus_path: Path, queries_path: Path) -> None:
        load_real_corpus._verify_manifest(
            str(manifest_path),
            _PREFIX,
            str(corpus_path),
            str(queries_path),
            _DIM,
            allow_mismatch=False,
        )

    def test_manifest_with_portable_fields_is_accepted(self) -> None:
        manifest, corpus_path, queries_path, manifest_path = _build_base_manifest(
            self.tmp_dir, self.corpus_rows, self.query_rows
        )
        # The absolute `source_parquet` path intentionally references a file
        # that does not exist on disk — the verifier must not try to open it.
        manifest["source_parquet"] = "/does/not/exist/dbpedia_1M.parquet"
        manifest["source_parquet_basename"] = "dbpedia_1M.parquet"
        manifest["source_parquet_shard_basenames"] = [
            "shard-00000.parquet",
            "shard-00001.parquet",
        ]
        _write_manifest_json(manifest_path, manifest)
        self._verify(manifest_path, corpus_path, queries_path)

    def test_manifest_without_portable_fields_is_accepted(self) -> None:
        manifest, corpus_path, queries_path, manifest_path = _build_base_manifest(
            self.tmp_dir, self.corpus_rows, self.query_rows
        )
        # Older manifests omit the new fields entirely. The verifier must
        # still accept them so already-staged fixtures keep loading.
        manifest["source_parquet"] = "/legacy/absolute/path.parquet"
        _write_manifest_json(manifest_path, manifest)
        self._verify(manifest_path, corpus_path, queries_path)

    def test_manifest_with_absolute_basename_is_rejected(self) -> None:
        manifest, corpus_path, queries_path, manifest_path = _build_base_manifest(
            self.tmp_dir, self.corpus_rows, self.query_rows
        )
        manifest["source_parquet"] = "/home/dev/datasets/dbpedia_1M.parquet"
        manifest["source_parquet_basename"] = "/home/dev/datasets/dbpedia_1M.parquet"
        manifest["source_parquet_shard_basenames"] = ["shard-00000.parquet"]
        _write_manifest_json(manifest_path, manifest)
        with self.assertRaises(ValueError) as ctx:
            self._verify(manifest_path, corpus_path, queries_path)
        self.assertIn("source_parquet_basename", str(ctx.exception))

    def test_manifest_with_absolute_shard_is_rejected(self) -> None:
        manifest, corpus_path, queries_path, manifest_path = _build_base_manifest(
            self.tmp_dir, self.corpus_rows, self.query_rows
        )
        manifest["source_parquet"] = "/home/dev/datasets/dbpedia_1M"
        manifest["source_parquet_basename"] = "dbpedia_1M"
        manifest["source_parquet_shard_basenames"] = [
            "/home/dev/datasets/dbpedia_1M/shard-00000.parquet",
        ]
        _write_manifest_json(manifest_path, manifest)
        with self.assertRaises(ValueError) as ctx:
            self._verify(manifest_path, corpus_path, queries_path)
        self.assertIn("source_parquet_shard_basenames", str(ctx.exception))

    def test_manifest_with_non_string_basename_is_rejected(self) -> None:
        manifest, corpus_path, queries_path, manifest_path = _build_base_manifest(
            self.tmp_dir, self.corpus_rows, self.query_rows
        )
        manifest["source_parquet"] = "/home/dev/datasets/dbpedia_1M.parquet"
        manifest["source_parquet_basename"] = 42  # type: ignore[assignment]
        _write_manifest_json(manifest_path, manifest)
        with self.assertRaises(ValueError) as ctx:
            self._verify(manifest_path, corpus_path, queries_path)
        self.assertIn("source_parquet_basename", str(ctx.exception))

    def test_manifest_with_non_list_shard_basenames_is_rejected(self) -> None:
        manifest, corpus_path, queries_path, manifest_path = _build_base_manifest(
            self.tmp_dir, self.corpus_rows, self.query_rows
        )
        manifest["source_parquet"] = "/home/dev/datasets/dbpedia_1M"
        manifest["source_parquet_basename"] = "dbpedia_1M"
        manifest["source_parquet_shard_basenames"] = "shard-00000.parquet"  # type: ignore[assignment]
        _write_manifest_json(manifest_path, manifest)
        with self.assertRaises(ValueError) as ctx:
            self._verify(manifest_path, corpus_path, queries_path)
        self.assertIn("source_parquet_shard_basenames", str(ctx.exception))

    def test_writer_emits_portable_fields_for_file_input(self) -> None:
        # Also exercise the writer side so the round-trip (writer ->
        # verifier) is covered end-to-end. We stub the parquet file list and
        # drive `_write_manifest` directly, bypassing the parquet reader.
        import qdrant_dbpedia_to_tsv as writer

        parquet_dir = self.tmp_dir / "dbpedia_1M"
        parquet_dir.mkdir()
        shard_a = parquet_dir / "shard-00001.parquet"
        shard_b = parquet_dir / "shard-00000.parquet"
        shard_a.write_bytes(b"")
        shard_b.write_bytes(b"")

        profile = writer.PROFILES["ec_hnsw_real_10k"]
        corpus_fm = writer.FileManifest(
            file="x_corpus.tsv", rows=0, sha256="", first_id=None, last_id=None
        )
        query_fm = writer.FileManifest(
            file="x_queries.tsv", rows=0, sha256="", first_id=None, last_id=None
        )

        manifest_path = self.tmp_dir / "out_manifest.json"
        writer._write_manifest(
            manifest_path,
            profile=profile,
            source_parquet=str(parquet_dir.resolve()),
            parquet_files=[shard_a, shard_b],
            source_dataset="synthetic test dataset",
            id_column="_id",
            vector_column="embedding",
            dim=_DIM,
            corpus_manifest=corpus_fm,
            query_manifest=query_fm,
        )

        with manifest_path.open("r", encoding="utf-8") as handle:
            data = json.load(handle)
        self.assertEqual(data["source_parquet_basename"], "dbpedia_1M")
        # Shard basenames must be sorted and stripped of their directory part.
        self.assertEqual(
            data["source_parquet_shard_basenames"],
            ["shard-00000.parquet", "shard-00001.parquet"],
        )
        # The absolute path field is still present as a debug hint.
        self.assertTrue(os.path.isabs(data["source_parquet"]))

    def test_writer_basename_handles_trailing_slash(self) -> None:
        import qdrant_dbpedia_to_tsv as writer

        self.assertEqual(writer._source_parquet_basename("/a/b/c"), "c")
        self.assertEqual(writer._source_parquet_basename("/a/b/c/"), "c")
        self.assertEqual(writer._source_parquet_basename("/a/b/c.parquet"), "c.parquet")


def main() -> int:
    suite = unittest.defaultTestLoader.loadTestsFromTestCase(ManifestPortabilityTests)
    runner = unittest.TextTestRunner(verbosity=2)
    try:
        result = runner.run(suite)
    except Exception:  # pragma: no cover — defensive for direct invocation
        traceback.print_exc()
        return 1
    return 0 if result.wasSuccessful() else 1


if __name__ == "__main__":
    sys.exit(main())
