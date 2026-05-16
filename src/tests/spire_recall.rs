    fn spire_recall_set(ids: Vec<i64>) -> std::collections::BTreeSet<i64> {
        ids.into_iter().collect()
    }

    fn spire_recall_at_k(table_name: &str, query: &str, k: i64) -> usize {
        let predicted = spire_recall_set(spire_scan_top_ids(table_name, query, k));
        let exact = spire_recall_set(spire_scan_exact_top_ids(table_name, query, k));

        assert_eq!(predicted.len(), k as usize, "predicted top-k ids must be unique");
        assert_eq!(exact.len(), k as usize, "exact top-k ids must be unique");
        predicted.intersection(&exact).count()
    }

    #[pg_test]
    fn test_ec_spire_recall_at_10_matches_exact_on_full_probe() {
        Spi::run("CREATE TABLE ec_spire_recall_full_probe (id bigint primary key, embedding ecvector)")
            .expect("recall full-probe table creation should succeed");
        Spi::run(
            "INSERT INTO ec_spire_recall_full_probe (id, embedding) \
             SELECT id, encode_to_ecvector(\
                    ARRAY[(id::real / 64.0)::real, ((65 - id)::real / 64.0)::real], 4, 42) \
               FROM generate_series(1, 64) AS id",
        )
        .expect("recall full-probe corpus insert should succeed");
        Spi::run(
            "CREATE INDEX ec_spire_recall_full_probe_idx \
             ON ec_spire_recall_full_probe USING ec_spire \
             (embedding ecvector_spire_ip_ops) \
             WITH (nlists = 8, nprobe = 8, rerank_width = 64, training_sample_rows = 64)",
        )
        .expect("recall full-probe ec_spire index creation should succeed");

        Spi::run("SET LOCAL enable_seqscan = off").expect("SET should succeed");
        let query = "ARRAY[1.0, 0.0]";
        let predicted = spire_scan_top_ids("ec_spire_recall_full_probe", query, 10);
        let exact = spire_scan_exact_top_ids("ec_spire_recall_full_probe", query, 10);

        assert_eq!(predicted, exact, "full-probe recall@10 should be 1.0");
        assert_eq!(
            spire_recall_set(predicted).len(),
            10,
            "full-probe top-k ids must be unique"
        );
    }

    #[pg_test]
    fn test_ec_spire_nprobe_sweep_recall_is_monotonic() {
        Spi::run("CREATE TABLE ec_spire_recall_nprobe_sweep (id bigint primary key, embedding ecvector)")
            .expect("recall nprobe-sweep table creation should succeed");
        Spi::run(
            "INSERT INTO ec_spire_recall_nprobe_sweep (id, embedding) \
             SELECT id, encode_to_ecvector(\
                    ARRAY[(id::real / 64.0)::real, ((65 - id)::real / 64.0)::real], 4, 42) \
               FROM generate_series(1, 64) AS id",
        )
        .expect("recall nprobe-sweep corpus insert should succeed");
        Spi::run(
            "CREATE INDEX ec_spire_recall_nprobe_sweep_idx \
             ON ec_spire_recall_nprobe_sweep USING ec_spire \
             (embedding ecvector_spire_ip_ops) \
             WITH (nlists = 16, nprobe = 1, rerank_width = 64, training_sample_rows = 64)",
        )
        .expect("recall nprobe-sweep ec_spire index creation should succeed");

        Spi::run("SET LOCAL enable_seqscan = off").expect("SET should succeed");
        let query = "ARRAY[1.0, 0.0]";
        let mut previous_recall_count = 0;
        for nprobe in [1, 4, 8, 16] {
            Spi::run(&format!("SET LOCAL ec_spire.nprobe = {nprobe}"))
                .expect("nprobe override should succeed");
            let recall_count = spire_recall_at_k("ec_spire_recall_nprobe_sweep", query, 10);
            assert!(
                recall_count >= previous_recall_count,
                "recall@10 should be monotonic across nprobe sweep; nprobe={nprobe}, \
                 previous={previous_recall_count}, current={recall_count}"
            );
            previous_recall_count = recall_count;
        }
    }
