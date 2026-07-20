// Task 163 D8 runtime transport coverage. Keep this separate from the Task 179
// control/registration fixture so the two review lanes can evolve independently.

#[pg_test]
fn test_ec_distann_d8_multiblock_buffile_spool() {
    let row_count = (2 * pg_sys::BLCKSZ as usize / 8) + 1;

    Spi::run(
        "CREATE TABLE ec_distann_d8_multiblock (
             id bigint PRIMARY KEY,
             embedding ecvector NOT NULL
         )",
    )
    .expect("D8 multiblock table should create");
    Spi::run(&format!(
        "INSERT INTO ec_distann_d8_multiblock
         SELECT g,
                encode_to_ecvector(
                    ARRAY[
                        (sin(g::double precision) / 1.4142135623730951)::real,
                        (cos(g::double precision) / 1.4142135623730951)::real,
                        (sin(g::double precision * 0.37) / 1.4142135623730951)::real,
                        (cos(g::double precision * 0.37) / 1.4142135623730951)::real
                    ],
                    4,
                    42
                )
         FROM generate_series(1, {row_count}) AS g"
    ))
    .expect("D8 multiblock rows should insert");

    // There are enough required eight-byte membership headers to exceed two
    // BLCKSZ buffers across exactly two shard files. By pigeonhole, at least
    // one real BufFile must therefore cross a block boundary even before its
    // adjacency payload or closure duplicates are counted. The build NOTICE
    // captured by the focused pg_test records the exact total spill bytes.
    Spi::run(
        "CREATE INDEX ec_distann_d8_multiblock_idx
         ON ec_distann_d8_multiblock
         USING ec_distann (embedding ecvector_distann_ip_ops)
         WITH (
             graph_degree = 16,
             build_list_size = 32,
             head_index_cap = 64,
             build_shards = 2,
             closure_epsilon = 1.0,
             neighbor_code_format = 'rabitq'
         )",
    )
    .expect("D8 multiblock index should build through runtime BufFiles");

    let (metadata, _) = distann_materialized_index("ec_distann_d8_multiblock_idx");
    assert_eq!(
        metadata.node_count, row_count as u64,
        "multi-block stitch must emit every global vec_id exactly once"
    );
    assert_ne!(
        metadata.entry_point,
        crate::storage::page::ItemPointer::INVALID,
        "multi-block stitch must retain a valid global medoid"
    );
}
