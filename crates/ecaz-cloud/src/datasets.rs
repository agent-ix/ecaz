//! Dataset registry (FR-046).
//!
//! Each entry maps a short name to a source location, dimension, distance
//! metric, and the third-party benchmarks it is comparable against. The
//! registry is the single source of truth for `ecaz cloud corpus stage`.

use serde::Serialize;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub enum Format {
    /// Native parquet (Hugging Face). Streamed directly through the
    /// existing `ecaz corpus prepare` + `ecaz corpus load` pipeline.
    Parquet,
    /// BIGANN/DEEP1B fixed-width binary (`.fbin` for f32, `.u8bin` for
    /// uint8). Converted to parquet during `corpus stage`.
    BigAnnFbin,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub enum Distance {
    Cosine,
    InnerProduct,
    L2,
}

#[derive(Clone, Debug, Serialize)]
pub struct Dataset {
    pub name: &'static str,
    /// Free-text source identifier (HF repo path or canonical mirror URL).
    pub source: &'static str,
    /// Glob or directory under the source that points at the actual data.
    pub source_path: &'static str,
    pub rows: u64,
    pub dim: u32,
    pub distance: Distance,
    pub format: Format,
    pub comparable_to: &'static [&'static str],
}

pub const REGISTRY: &[Dataset] = &[
    Dataset {
        name: "dbpedia-1m",
        source: "Qdrant/dbpedia-entities-openai3-embedding-3-large-1536-1M",
        source_path: "*.parquet",
        rows: 1_000_000,
        dim: 1536,
        distance: Distance::Cosine,
        format: Format::Parquet,
        comparable_to: &["Qdrant blog benches"],
    },
    Dataset {
        name: "dbpedia-ada-1m",
        source: "KShivendu/dbpedia-entities-openai-1M",
        source_path: "*.parquet",
        rows: 1_000_000,
        dim: 1536,
        distance: Distance::Cosine,
        format: Format::Parquet,
        comparable_to: &["pgvector posts", "Qdrant blog benches"],
    },
    Dataset {
        name: "cohere-wiki-10m",
        source: "Cohere/wikipedia-22-12-en-embeddings",
        source_path: "data/*.parquet",
        rows: 10_000_000,
        dim: 768,
        distance: Distance::Cosine,
        format: Format::Parquet,
        comparable_to: &["Qdrant", "Weaviate"],
    },
    Dataset {
        name: "cohere-wiki-35m",
        source: "Cohere/wikipedia-22-12-en-embeddings",
        source_path: "data/*.parquet",
        rows: 35_167_920,
        dim: 768,
        distance: Distance::Cosine,
        format: Format::Parquet,
        comparable_to: &["Qdrant", "Weaviate"],
    },
    Dataset {
        name: "laion-100m",
        source: "laion/laion2B-en-vit-l-14-embeddings",
        source_path: "*.parquet",
        rows: 100_000_000,
        dim: 768,
        distance: Distance::Cosine,
        format: Format::Parquet,
        comparable_to: &["LAION research"],
    },
    Dataset {
        name: "bigann-1b",
        source: "https://big-ann-benchmarks.com/datasets/bigann",
        source_path: "base.1B.u8bin",
        rows: 1_000_000_000,
        dim: 128,
        distance: Distance::L2,
        format: Format::BigAnnFbin,
        comparable_to: &["NeurIPS Big-ANN", "ann-benchmarks.com"],
    },
];

pub fn lookup(name: &str) -> Option<&'static Dataset> {
    REGISTRY.iter().find(|d| d.name == name)
}
