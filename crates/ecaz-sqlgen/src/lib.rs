//! ECAZ-grammar SQL generator.
//!
//! Implements the five generation templates Task 46 §Approach 3
//! names by line:
//!
//!   1. `SELECT … ORDER BY embedding <op> $1 LIMIT n` (with op
//!      drawn from `<->`, `<#>`, `<=>`)
//!   2. `CREATE INDEX … USING <am>` followed by random
//!      INSERT / SELECT / VACUUM
//!   3. prepared statement with bound vector parameters of
//!      varying dim
//!   4. partial / expression indexes over the vector column
//!   5. `REINDEX CONCURRENTLY` interleaved with queries
//!
//! Each generator is deterministic in its seed; the same
//! `--seed` reproduces the same SQL stream, which is what makes
//! the lane reviewable against a committed seed corpus.

use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;

/// Access methods this generator targets. Names match the ECAZ
/// extension's `CREATE INDEX … USING` keywords.
pub const ACCESS_METHODS: &[&str] = &["ec_diskann", "ec_hnsw", "ec_ivf"];

/// Vector-distance operators registered by the ECAZ extension.
pub const VECTOR_OPS: &[&str] = &["<->", "<#>", "<=>"];

/// Typical embedding dimensions the production cluster sees.
pub const DIM_LADDER: &[usize] = &[16, 64, 128, 256, 384, 512, 768, 1024, 1536, 3072];

/// Standard top-k limits a production workload tends to ask for.
pub const TOPK_LADDER: &[usize] = &[1, 5, 10, 25, 50, 100];

pub struct Generator {
    rng: ChaCha8Rng,
}

impl Generator {
    pub fn from_seed(seed: u64) -> Self {
        Self {
            rng: ChaCha8Rng::seed_from_u64(seed),
        }
    }

    fn pick<'a, T: ?Sized>(&mut self, options: &'a [&'a T]) -> &'a T {
        let idx = self.rng.gen_range(0..options.len());
        options[idx]
    }

    fn pick_usize(&mut self, options: &[usize]) -> usize {
        let idx = self.rng.gen_range(0..options.len());
        options[idx]
    }

    fn random_vector(&mut self, dim: usize) -> String {
        let mut s = String::with_capacity(2 + dim * 12);
        s.push('[');
        for i in 0..dim {
            if i > 0 {
                s.push(',');
            }
            let v: f32 = self.rng.gen_range(-1.0..=1.0);
            s.push_str(&format!("{v:.4}"));
        }
        s.push(']');
        s
    }

    /// Template 1: vector ORDER BY top-k.
    pub fn select_order_by_vector(&mut self, table: &str, column: &str) -> String {
        let dim = self.pick_usize(DIM_LADDER);
        let op = self.pick(VECTOR_OPS);
        let k = self.pick_usize(TOPK_LADDER);
        let q = self.random_vector(dim);
        format!("SELECT id FROM {table} ORDER BY {column} {op} '{q}'::vector LIMIT {k};")
    }

    /// Template 2: CREATE INDEX followed by INSERT / SELECT /
    /// VACUUM. Returns a Vec of statements so the caller can
    /// run them in sequence inside one session.
    pub fn create_index_and_workload(&mut self, table: &str, column: &str) -> Vec<String> {
        let am = self.pick(ACCESS_METHODS);
        let dim = self.pick_usize(DIM_LADDER);
        let index_name = format!("{table}_{am}_idx_{}", self.rng.gen_range(0..u32::MAX));
        let mut out = Vec::with_capacity(4);
        out.push(format!(
            "CREATE INDEX IF NOT EXISTS {index_name} ON {table} USING {am} ({column});"
        ));
        let q = self.random_vector(dim);
        out.push(format!(
            "INSERT INTO {table} ({column}) VALUES ('{q}'::vector);"
        ));
        let op = self.pick(VECTOR_OPS);
        let q2 = self.random_vector(dim);
        out.push(format!(
            "SELECT id FROM {table} ORDER BY {column} {op} '{q2}'::vector LIMIT 10;"
        ));
        out.push(format!("VACUUM {table};"));
        out
    }

    /// Template 3: prepared statement with bound vector parameter.
    pub fn prepared_vector_query(&mut self, table: &str, column: &str) -> Vec<String> {
        let stmt = format!("ecaz_sqlgen_p_{}", self.rng.gen_range(0..u32::MAX));
        let op = self.pick(VECTOR_OPS);
        let k = self.pick_usize(TOPK_LADDER);
        let dim = self.pick_usize(DIM_LADDER);
        let q = self.random_vector(dim);
        vec![
            format!(
                "PREPARE {stmt}(vector) AS SELECT id FROM {table} ORDER BY {column} {op} $1 LIMIT {k};"
            ),
            format!("EXECUTE {stmt}('{q}'::vector);"),
            format!("DEALLOCATE {stmt};"),
        ]
    }

    /// Template 4: partial / expression indexes over the vector
    /// column.
    pub fn partial_or_expression_index(&mut self, table: &str, column: &str) -> String {
        let am = self.pick(ACCESS_METHODS);
        let index_name = format!("{table}_{am}_pidx_{}", self.rng.gen_range(0..u32::MAX));
        let kind = self.rng.gen_range(0..2u8);
        if kind == 0 {
            // Partial: only index rows whose id is above a threshold.
            let threshold: u32 = self.rng.gen_range(0..1_000_000);
            format!(
                "CREATE INDEX IF NOT EXISTS {index_name} ON {table} \
                 USING {am} ({column}) WHERE id > {threshold};"
            )
        } else {
            // Expression: index a tagged column.
            format!(
                "CREATE INDEX IF NOT EXISTS {index_name} ON {table} \
                 USING {am} (({column}::vector));"
            )
        }
    }

    /// Template 5: REINDEX CONCURRENTLY interleaved with a query.
    pub fn reindex_interleaved(&mut self, table: &str, column: &str) -> Vec<String> {
        let am = self.pick(ACCESS_METHODS);
        let index_name = format!("{table}_{am}_rdx_{}", self.rng.gen_range(0..u32::MAX));
        let op = self.pick(VECTOR_OPS);
        let dim = self.pick_usize(DIM_LADDER);
        let q = self.random_vector(dim);
        vec![
            format!(
                "CREATE INDEX IF NOT EXISTS {index_name} ON {table} USING {am} ({column});"
            ),
            format!("REINDEX INDEX CONCURRENTLY {index_name};"),
            format!(
                "SELECT id FROM {table} ORDER BY {column} {op} '{q}'::vector LIMIT 5;"
            ),
        ]
    }

    /// Draw one statement using a uniformly-random template.
    pub fn one_statement(&mut self, table: &str, column: &str) -> Vec<String> {
        let template = self.rng.gen_range(0..5u8);
        match template {
            0 => vec![self.select_order_by_vector(table, column)],
            1 => self.create_index_and_workload(table, column),
            2 => self.prepared_vector_query(table, column),
            3 => vec![self.partial_or_expression_index(table, column)],
            _ => self.reindex_interleaved(table, column),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deterministic_for_same_seed() {
        let mut a = Generator::from_seed(42);
        let mut b = Generator::from_seed(42);
        for _ in 0..16 {
            assert_eq!(a.one_statement("t", "v"), b.one_statement("t", "v"));
        }
    }

    #[test]
    fn each_template_emits_at_least_one_statement() {
        let mut g = Generator::from_seed(1);
        assert!(!g.select_order_by_vector("t", "v").is_empty());
        assert!(!g.create_index_and_workload("t", "v").is_empty());
        assert!(!g.prepared_vector_query("t", "v").is_empty());
        assert!(!g.partial_or_expression_index("t", "v").is_empty());
        assert!(!g.reindex_interleaved("t", "v").is_empty());
    }

    #[test]
    fn statements_terminate_with_semicolon() {
        let mut g = Generator::from_seed(7);
        for _ in 0..32 {
            for s in g.one_statement("t", "v") {
                assert!(s.trim_end().ends_with(';'), "statement: {s}");
            }
        }
    }
}
