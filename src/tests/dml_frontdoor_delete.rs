    #[pg_test]
    fn test_ec_spire_remote_delete_tuple_payload_idempotent_shape_sql() {
        Spi::run(
            "CREATE TABLE ec_spire_remote_delete_idempotent_sql \
             (id bigint primary key, title text not null, embedding ecvector)",
        )
        .expect("remote delete idempotent table creation should succeed");
        Spi::run(
            "INSERT INTO ec_spire_remote_delete_idempotent_sql (id, title, embedding) VALUES \
             (7001, 'delete once', encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42))",
        )
        .expect("remote delete idempotent seed insert should succeed");
        Spi::run(
            "CREATE INDEX ec_spire_remote_delete_idempotent_idx \
             ON ec_spire_remote_delete_idempotent_sql USING ec_spire \
             (embedding ecvector_spire_ip_ops) WITH (nlists = 1)",
        )
        .expect("remote delete idempotent ec_spire index creation should succeed");

        let first_delete = Spi::get_one::<String>(
            "SELECT deleted_count::text || '|' || status \
               FROM ec_spire_remote_delete_tuple_payload(\
                    'ec_spire_remote_delete_idempotent_idx'::regclass, \
                    'id', \
                    int8send(7001::bigint)::bytea)",
        )
        .expect("first remote delete idempotent query should succeed")
        .expect("first remote delete idempotent query should return a row");
        let second_delete = Spi::get_one::<String>(
            "SELECT deleted_count::text || '|' || status \
               FROM ec_spire_remote_delete_tuple_payload(\
                    'ec_spire_remote_delete_idempotent_idx'::regclass, \
                    'id', \
                    int8send(7001::bigint)::bytea)",
        )
        .expect("second remote delete idempotent query should succeed")
        .expect("second remote delete idempotent query should return a row");
        let remaining_rows = Spi::get_one::<i64>(
            "SELECT count(*) FROM ec_spire_remote_delete_idempotent_sql WHERE id = 7001",
        )
        .expect("remote delete idempotent remaining count query should succeed")
        .expect("remote delete idempotent remaining count should exist");

        assert_eq!(first_delete, "1|ready");
        assert_eq!(second_delete, "0|ready");
        assert_eq!(remaining_rows, 0);
    }
