    #[pg_test]
    fn test_ec_spire_concurrent_same_pk_delete_collision_sql() {
        let conninfo = current_pg_test_loopback_conninfo();
        let mut setup_client = postgres::Client::connect(&conninfo, postgres::NoTls)
            .expect("coordinator setup connection should succeed");
        setup_client
            .batch_execute(
                "DROP TABLE IF EXISTS ec_spire_dml_concurrent_delete_sql; \
                 CREATE TABLE ec_spire_dml_concurrent_delete_sql \
                     (id bigint primary key, title text not null, embedding ecvector); \
                 INSERT INTO ec_spire_dml_concurrent_delete_sql (id, title, embedding) VALUES \
                     (8808, 'delete collision target', \
                      encode_to_ecvector(ARRAY[1.0, 0.0], 4, 42)); \
                 CREATE INDEX ec_spire_dml_concurrent_delete_idx \
                     ON ec_spire_dml_concurrent_delete_sql USING ec_spire \
                     (embedding ecvector_spire_ip_ops) WITH (nlists = 1); \
                 SELECT ec_spire_dml_frontdoor_hook_status();",
            )
            .expect("coordinator concurrent delete fixture should be created");

        let index_oid = setup_client
            .query_one(
                "SELECT 'ec_spire_dml_concurrent_delete_idx'::regclass::oid",
                &[],
            )
            .expect("concurrent delete index oid query should succeed")
            .try_get::<_, u32>(0)
            .expect("concurrent delete index oid should decode");
        let prepared_prefix = format!("ec_spire_insert_{index_oid}_%");

        let (deleted_tx, deleted_rx) = std::sync::mpsc::channel::<()>();
        let first_conninfo = conninfo.clone();
        let first_delete = std::thread::spawn(move || -> Result<u64, String> {
            let mut client = postgres::Client::connect(&first_conninfo, postgres::NoTls)
                .map_err(|e| format!("first delete connection failed: {e}"))?;
            client
                .batch_execute("BEGIN; SELECT ec_spire_dml_frontdoor_hook_status();")
                .map_err(|e| format!("first delete setup failed: {e}"))?;
            let deleted = client
                .execute(
                    "DELETE FROM ec_spire_dml_concurrent_delete_sql WHERE id = 8808",
                    &[],
                )
                .map_err(|e| format!("first delete failed: {e}"))?;
            deleted_tx
                .send(())
                .map_err(|e| format!("first delete signal failed: {e}"))?;
            std::thread::sleep(std::time::Duration::from_millis(250));
            client
                .batch_execute("COMMIT")
                .map_err(|e| format!("first delete commit failed: {e}"))?;
            Ok(deleted)
        });

        deleted_rx
            .recv()
            .expect("first delete should reach the row-lock hold point");

        let second_delete = std::thread::spawn(move || -> Result<u64, String> {
            let mut client = postgres::Client::connect(&conninfo, postgres::NoTls)
                .map_err(|e| format!("second delete connection failed: {e}"))?;
            client
                .batch_execute("SELECT ec_spire_dml_frontdoor_hook_status();")
                .map_err(|e| format!("second delete setup failed: {e}"))?;
            client
                .execute(
                    "DELETE FROM ec_spire_dml_concurrent_delete_sql WHERE id = 8808",
                    &[],
                )
                .map_err(|e| format!("second delete failed: {e}"))
        });

        let first_deleted = first_delete
            .join()
            .expect("first delete thread should not panic")
            .expect("first delete should succeed");
        let second_deleted = second_delete
            .join()
            .expect("second delete thread should not panic")
            .expect("second delete should succeed");
        let mut delete_counts = [first_deleted, second_deleted];
        delete_counts.sort_unstable();

        let cleanup_summary = setup_client
            .query_one(
                "SELECT \
                    (SELECT count(*)::bigint \
                       FROM ec_spire_dml_concurrent_delete_sql \
                      WHERE id = 8808)::text \
                    || '|' || \
                    (SELECT count(*)::bigint \
                       FROM ec_spire_placement \
                      WHERE index_oid = 'ec_spire_dml_concurrent_delete_idx'::regclass \
                        AND pk_value = int8send(8808::bigint)::bytea)::text \
                    || '|' || \
                    (SELECT count(*)::bigint \
                       FROM pg_prepared_xacts \
                      WHERE gid LIKE $1)::text",
                &[&prepared_prefix],
            )
            .expect("concurrent delete cleanup query should succeed")
            .try_get::<_, String>(0)
            .expect("concurrent delete cleanup summary should decode");

        assert_eq!(delete_counts, [0, 1]);
        assert_eq!(cleanup_summary, "0|0|0");
    }
