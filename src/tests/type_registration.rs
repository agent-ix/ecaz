    #[pg_test]
    fn test_binary_send_matches_internal_layout() {
        let bytes = Spi::get_one::<Vec<u8>>(
            "SELECT tqvector_send('[dim=4,bits=4,seed=42,gamma=0.5]:112233'::tqvector)",
        )
        .expect("SPI query should succeed")
        .expect("query should return one row");

        assert_eq!(bytes, pack(4, 4, 42, 0.5, &[0x11, 0x22, 0x33]));
    }

    #[pg_test]
    fn test_encode_to_tqvector_round_trips_canonical_artifact_layout() {
        let expected = encode_embedding_to_tqvector(vec![1.0, 0.0, 0.5, -1.0], 4, 42)
            .expect("canonical tqvector artifact should encode");
        let actual = Spi::get_one::<Vec<u8>>(
            "SELECT tqvector_send(encode_to_tqvector(ARRAY[1.0, 0.0, 0.5, -1.0], 4, 42))",
        )
        .expect("SPI query should succeed")
        .expect("query should return one row");

        assert_eq!(actual, expected);
    }

    #[pg_test]
    fn test_ecvector_binary_send_matches_internal_layout() {
        let bytes = Spi::get_one::<Vec<u8>>("SELECT ecvector_send('[1,0,-0.5]'::ecvector)")
            .expect("SPI query should succeed")
            .expect("query should return one row");

        assert_eq!(
            bytes,
            pack_raw_f32(&[1.0_f32, 0.0, -0.5], "test ecvector")
                .expect("packing raw test vector should succeed")
        );
    }

    #[pg_test]
    fn test_ecvector_real_array_cast_round_trips() {
        let values = Spi::get_one::<Vec<f32>>("SELECT ('{1,2,-0.5}'::real[])::ecvector::real[]")
            .expect("SPI query should succeed")
            .expect("query should return one row");

        assert_eq!(values, vec![1.0_f32, 2.0, -0.5]);
    }

    #[pg_test]
    fn test_ecvector_typmod_cast_round_trips() {
        let values =
            Spi::get_one::<Vec<f32>>("SELECT CAST('{1,2,-0.5}'::real[] AS ecvector(3))::real[]")
                .expect("SPI query should succeed")
                .expect("query should return one row");

        assert_eq!(values, vec![1.0_f32, 2.0, -0.5]);
    }

    #[pg_test]
    #[should_panic(expected = "dimension mismatch")]
    fn test_ecvector_typmod_rejects_dimension_mismatch() {
        Spi::run("SELECT '[1,2]'::ecvector(3)").expect("query should fail");
    }

    #[pg_test]
    #[should_panic(expected = "dimension mismatch")]
    fn test_ecvector_assignment_enforces_typmod() {
        Spi::run(
            "CREATE TABLE ec_hnsw_ecvector_typmod_assignment (id bigint, embedding ecvector(2))",
        )
        .expect("table creation should succeed");
        Spi::run("INSERT INTO ec_hnsw_ecvector_typmod_assignment VALUES (1, '[1,2,3]'::ecvector)")
            .expect("insert should fail");
    }

    #[pg_test]
    #[should_panic(expected = "must be finite")]
    fn test_ecvector_rejects_non_finite_text() {
        Spi::run("SELECT '[1,NaN]'::ecvector").expect("query should fail");
    }

    #[pg_test]
    fn test_access_method_is_registered() {
        let amname =
            Spi::get_one::<String>("SELECT amname::text FROM pg_am WHERE amname = 'ec_hnsw'")
                .expect("SPI query should succeed")
                .expect("access method should exist");
        assert_eq!(amname, "ec_hnsw");
    }

    #[pg_test]
    fn test_operator_class_is_registered() {
        let opcname = Spi::get_one::<String>(
            "SELECT opcname::text FROM pg_opclass WHERE opcname = 'ecvector_ip_ops'",
        )
        .expect("SPI query should succeed")
        .expect("operator class should exist");
        assert_eq!(opcname, "ecvector_ip_ops");
    }

    #[pg_test]
    fn test_ec_ivf_access_method_is_registered() {
        let amname =
            Spi::get_one::<String>("SELECT amname::text FROM pg_am WHERE amname = 'ec_ivf'")
                .expect("SPI query should succeed")
                .expect("access method should exist");
        assert_eq!(amname, "ec_ivf");
    }

    #[pg_test]
    fn test_ec_ivf_operator_classes_are_registered() {
        let opclasses = Spi::get_one::<i64>(
            "SELECT count(*) FROM pg_opclass opc \
             JOIN pg_am am ON am.oid = opc.opcmethod \
             WHERE am.amname = 'ec_ivf' \
             AND opc.opcname IN ('tqvector_ip_ops', 'ecvector_ip_ops')",
        )
        .expect("SPI query should succeed")
        .expect("operator class count should exist");
        assert_eq!(opclasses, 2);
    }
