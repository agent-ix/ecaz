# Review: Quantizer Cache Unbounded Growth

**File:** `src/quant/prod.rs:41-44`
**Severity:** Medium (memory leak under adversarial workloads)
**Category:** Memory safety / resource management

## Finding

The global quantizer cache is a `Mutex<HashMap<QuantizerKey, Arc<ProdQuantizer>>>` that grows without bound:

```rust
fn cache() -> &'static Mutex<HashMap<QuantizerKey, Arc<ProdQuantizer>>> {
    static CACHE: OnceLock<Mutex<HashMap<QuantizerKey, Arc<ProdQuantizer>>>> = OnceLock::new();
    CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}
```

Each unique `(dim, bits, seed)` triple creates a new entry that is never evicted. A `ProdQuantizer` for dim=1536 holds:
- `codebook`: `2^(bits-1)` f32 values (~128 floats = 512 bytes at bits=8)
- `signs`: `transform_dim` f32 values (2048 floats = 8KB at dim=1536)
- `qjl_signs`: another 8KB

Total ~17KB per entry. Under normal usage with a small number of index configurations, this is fine. However, in a shared Postgres backend that processes many different index configurations over its lifetime, the cache grows monotonically.

## Recommendation

This is acceptable for v0.1 given that the key space in practice is tiny (one or two configurations per database). Add a comment documenting the design assumption and consider an LRU or bounded cache before v1.0 if multi-tenant/multi-index workloads are expected.

## Action Required

No code change needed now. Document the assumption with a short inline comment.
