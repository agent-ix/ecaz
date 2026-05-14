    #[pg_test]
    fn test_ec_spire_text_projection_nul_byte_rejected_sql() {
        Spi::run(
            "CREATE TABLE ec_spire_text_nul_projection_sql \
             (id bigint primary key, title text not null, embedding ecvector)",
        )
        .expect("text-NUL projection table creation should succeed");
        Spi::run(
            "CREATE INDEX ec_spire_text_nul_projection_idx \
             ON ec_spire_text_nul_projection_sql USING ec_spire \
             (embedding ecvector_spire_ip_ops) WITH (nlists = 1)",
        )
        .expect("text-NUL projection ec_spire index creation should succeed");

        let error = pg_sys::PgTryBuilder::new(|| {
            Spi::run(
                "INSERT INTO ec_spire_text_nul_projection_sql (id, title, embedding) \
                 VALUES ( \
                     1, \
                     convert_from(decode('72656d6f74650074657874', 'hex'), 'UTF8'), \
                     encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42) \
                 )",
            )
            .expect("text projection with embedded NUL should fail before CustomScan");
            "no_error".to_owned()
        })
        .catch_others(|cause| match cause {
            pg_sys::panic::CaughtError::ErrorReport(report)
            | pg_sys::panic::CaughtError::PostgresError(report) => report.message().to_owned(),
            pg_sys::panic::CaughtError::RustPanic { ereport, .. } => {
                ereport.message().to_owned()
            }
        })
        .execute();
        let row_count = Spi::get_one::<i64>(
            "SELECT count(*) FROM ec_spire_text_nul_projection_sql",
        )
        .expect("text-NUL projection row count should succeed")
        .expect("text-NUL projection row count should exist");

        assert!(
            error.contains("invalid byte sequence for encoding")
                || error.contains("null character not permitted"),
            "expected PostgreSQL text NUL rejection, got: {error}"
        );
        assert_eq!(row_count, 0);
    }
