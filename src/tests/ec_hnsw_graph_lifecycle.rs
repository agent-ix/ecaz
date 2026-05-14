    #[pg_test]
    fn test_ech_empty_index_insert_initializes_shape_metadata() {
        Spi::run("CREATE TABLE ec_hnsw_empty_insert (id bigint primary key, embedding ecvector)")
            .expect("table creation should succeed");
        Spi::run(
            "CREATE INDEX ec_hnsw_empty_insert_idx ON ec_hnsw_empty_insert USING ec_hnsw \
             (embedding ecvector_ip_ops)",
        )
        .expect("index creation should succeed");
        Spi::run(
            "INSERT INTO ec_hnsw_empty_insert VALUES
             (1, encode_to_ecvector(ARRAY[1.0, 0.0, 0.5, -1.0], 4, 42))",
        )
        .expect("insert should succeed");

        let index_oid =
            Spi::get_one::<pg_sys::Oid>("SELECT 'ec_hnsw_empty_insert_idx'::regclass::oid")
                .expect("SPI query should succeed")
                .expect("index oid should exist");
        let (metadata, elements, neighbors) =
            decode_index_elements_and_neighbors(index_oid, code_len(4, 4));
        assert_eq!(metadata.dimensions, 4);
        assert_eq!(metadata.bits, 4);
        assert_eq!(metadata.seed, 42);
        assert_ne!(metadata.entry_point, am::page::ItemPointer::INVALID);
        assert_eq!(elements.len(), 1);

        let (entry_tid, entry_element) = elements
            .iter()
            .find(|(tid, _)| *tid == metadata.entry_point)
            .expect("entry point should identify the inserted element");
        assert_eq!(metadata.max_level, entry_element.level);
        assert_eq!(entry_element.heaptids.len(), 1);

        let neighbor = neighbors
            .get(&entry_element.neighbortid)
            .expect("entry element neighbor tuple should exist");
        assert_eq!(neighbor.count as usize, neighbor.tids.len());
        assert_eq!(
            neighbor.tids.len(),
            am::page::neighbor_slots(entry_element.level, metadata.m)
        );

        let tuple_count = elements.len() + neighbors.len();
        assert_eq!(
            tuple_count, 2,
            "aminsert should append one neighbor and one element tuple"
        );
        assert_eq!(metadata.entry_point, *entry_tid);
    }

    #[pg_test]
    fn test_ech_empty_index_reuses_initialized_metadata() {
        Spi::run(
            "CREATE TABLE ec_hnsw_empty_insert_reuse (id bigint primary key, embedding ecvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "CREATE INDEX ec_hnsw_empty_insert_reuse_idx ON ec_hnsw_empty_insert_reuse USING ec_hnsw \
             (embedding ecvector_ip_ops)",
        )
        .expect("index creation should succeed");
        Spi::run(
            "INSERT INTO ec_hnsw_empty_insert_reuse VALUES
             (1, encode_to_ecvector(ARRAY[1.0, 0.0, 0.5, -1.0], 4, 42)),
             (2, encode_to_ecvector(ARRAY[0.0, 1.0, 0.25, -0.5], 4, 42))",
        )
        .expect("sequential inserts should succeed");

        let index_oid =
            Spi::get_one::<pg_sys::Oid>("SELECT 'ec_hnsw_empty_insert_reuse_idx'::regclass::oid")
                .expect("SPI query should succeed")
                .expect("index oid should exist");
        let (metadata, elements, _neighbors) =
            decode_index_elements_and_neighbors(index_oid, code_len(4, 4));
        assert_eq!(metadata.dimensions, 4);
        assert_eq!(metadata.bits, 4);
        assert_eq!(metadata.seed, 42);
        assert_ne!(metadata.entry_point, am::page::ItemPointer::INVALID);
        let entry_element = elements
            .iter()
            .find(|(tid, _)| *tid == metadata.entry_point)
            .expect("entry point should identify a live element tuple");
        assert_eq!(entry_element.1.level, metadata.max_level);

        let element_count = elements.len();
        assert_eq!(
            element_count, 2,
            "second insert into an initially empty index should validate against persisted shape metadata"
        );
    }

    #[pg_test]
    fn test_ech_insert_repairs_invalid_entry_point_after_shape_init() {
        Spi::run(
            "CREATE TABLE ec_hnsw_insert_entry_point_repair (id bigint primary key, embedding ecvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "CREATE INDEX ec_hnsw_insert_entry_point_repair_idx ON ec_hnsw_insert_entry_point_repair USING ec_hnsw \
             (embedding ecvector_ip_ops)",
        )
        .expect("index creation should succeed");
        Spi::run(
            "INSERT INTO ec_hnsw_insert_entry_point_repair VALUES
             (1, encode_to_ecvector(ARRAY[1.0, 0.0, 0.5, -1.0], 4, 42))",
        )
        .expect("seed insert should succeed");

        let index_oid = Spi::get_one::<pg_sys::Oid>(
            "SELECT 'ec_hnsw_insert_entry_point_repair_idx'::regclass::oid",
        )
        .expect("SPI query should succeed")
        .expect("index oid should exist");
        let (_block_count, _m, _ef_construction, mut metadata) =
            unsafe { am::debug_index_metadata(index_oid) };
        assert_eq!(metadata.dimensions, 4);
        assert_eq!(metadata.bits, 4);
        assert_eq!(metadata.seed, 42);
        assert_ne!(metadata.entry_point, am::page::ItemPointer::INVALID);

        metadata.entry_point = am::page::ItemPointer::INVALID;
        unsafe {
            am::debug_update_index_metadata(index_oid, metadata);
        }

        let (_block_count, _m, _ef_construction, metadata) =
            unsafe { am::debug_index_metadata(index_oid) };
        assert_eq!(metadata.entry_point, am::page::ItemPointer::INVALID);

        Spi::run(
            "INSERT INTO ec_hnsw_insert_entry_point_repair VALUES
             (2, encode_to_ecvector(ARRAY[0.0, 1.0, 0.25, -0.5], 4, 42))",
        )
        .expect("repairing insert should succeed");

        let (metadata, elements, _neighbors) =
            decode_index_elements_and_neighbors(index_oid, code_len(4, 4));
        assert_ne!(metadata.entry_point, am::page::ItemPointer::INVALID);
        assert_eq!(elements.len(), 2);
        let (entry_tid, entry_element) = elements
            .iter()
            .find(|(tid, _)| *tid == metadata.entry_point)
            .expect("repairing insert should repoint metadata at a live element tuple");
        assert_eq!(entry_element.level, metadata.max_level);
        assert_eq!(metadata.entry_point, *entry_tid);
    }

    #[pg_test]
    fn test_ech_insert_neighbor_tuple_sizing_matches_levels() {
        Spi::run(
            "CREATE TABLE ec_hnsw_insert_level_shape (id bigint primary key, embedding ecvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "CREATE INDEX ec_hnsw_insert_level_shape_idx ON ec_hnsw_insert_level_shape USING ec_hnsw \
             (embedding ecvector_ip_ops) WITH (m = 2)",
        )
        .expect("index creation should succeed");

        let index_oid =
            Spi::get_one::<pg_sys::Oid>("SELECT 'ec_hnsw_insert_level_shape_idx'::regclass::oid")
                .expect("SPI query should succeed")
                .expect("index oid should exist");

        let mut inserted_rows = 0_i64;
        let mut found_upper_level = false;
        while inserted_rows < 128 && !found_upper_level {
            inserted_rows += 1;
            Spi::run(&format!(
                "INSERT INTO ec_hnsw_insert_level_shape VALUES (
                    {id},
                    encode_to_ecvector(ARRAY[
                        {id}.0,
                        {two}.0,
                        {three}.0,
                        {four}.0
                    ], 4, 42)
                )",
                id = inserted_rows,
                two = inserted_rows * 2,
                three = inserted_rows * 3,
                four = inserted_rows * 4,
            ))
            .expect("insert should succeed");

            let heap_tid = heap_tid_for_row("ec_hnsw_insert_level_shape", inserted_rows);
            let expected_level =
                am::debug_insert_level_for_heap_tid(2, 42, heap_tid, code_len(4, 4));
            found_upper_level |= expected_level > 0;
        }
        assert!(
            found_upper_level,
            "deterministic insert level assignment should produce an upper-layer node within 128 inserts"
        );

        let (metadata, elements, neighbors) =
            decode_index_elements_and_neighbors(index_oid, code_len(4, 4));
        assert_eq!(elements.len(), inserted_rows as usize);
        assert!(
            elements.iter().any(|(_, element)| element.level > 0),
            "test fixture should contain at least one inserted upper-layer node"
        );

        for (_, element) in &elements {
            let neighbor = neighbors
                .get(&element.neighbortid)
                .expect("neighbor tuple should exist for each inserted element");
            assert_eq!(neighbor.count as usize, neighbor.tids.len());
            assert_eq!(
                neighbor.tids.len(),
                am::page::neighbor_slots(element.level, metadata.m),
                "neighbor tuple sizing should match the inserted element level",
            );
        }
    }

    #[pg_test]
    fn test_ech_insert_promotes_entry_point_on_level_up() {
        Spi::run(
            "CREATE TABLE ec_hnsw_insert_level_promotion (id bigint primary key, embedding ecvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "CREATE INDEX ec_hnsw_insert_level_promotion_idx ON ec_hnsw_insert_level_promotion USING ec_hnsw \
             (embedding ecvector_ip_ops) WITH (m = 2)",
        )
        .expect("index creation should succeed");

        Spi::run(
            "INSERT INTO ec_hnsw_insert_level_promotion VALUES
             (1, encode_to_ecvector(ARRAY[1.0, 2.0, 3.0, 4.0], 4, 42))",
        )
        .expect("seed insert should succeed");

        let index_oid = Spi::get_one::<pg_sys::Oid>(
            "SELECT 'ec_hnsw_insert_level_promotion_idx'::regclass::oid",
        )
        .expect("SPI query should succeed")
        .expect("index oid should exist");

        let mut previous_metadata = unsafe { am::debug_index_metadata(index_oid) }.3;
        for id in 2_i64..=128_i64 {
            Spi::run(&format!(
                "INSERT INTO ec_hnsw_insert_level_promotion VALUES (
                    {id},
                    encode_to_ecvector(ARRAY[
                        {id}.0,
                        {two}.0,
                        {three}.0,
                        {four}.0
                    ], 4, 42)
                )",
                id = id,
                two = id * 2,
                three = id * 3,
                four = id * 4,
            ))
            .expect("insert should succeed");

            let heap_tid = heap_tid_for_row("ec_hnsw_insert_level_promotion", id);
            let expected_level =
                am::debug_insert_level_for_heap_tid(2, 42, heap_tid, code_len(4, 4));
            let (metadata, elements, _neighbors) =
                decode_index_elements_and_neighbors(index_oid, code_len(4, 4));
            if expected_level > previous_metadata.max_level {
                let (promoted_tid, promoted_element) = elements
                    .iter()
                    .find(|(_, element)| element.heaptids.contains(&heap_tid))
                    .expect("promoted element should be discoverable by heap tid");
                assert_eq!(promoted_element.level, expected_level);
                assert_eq!(metadata.max_level, expected_level);
                assert_eq!(metadata.entry_point, *promoted_tid);
                return;
            }
            previous_metadata = metadata;
        }

        panic!("expected a higher-level insert to promote metadata within 128 inserts");
    }

    #[pg_test]
    fn test_ech_insert_populates_forward_links_from_live_entry_seed() {
        Spi::run(
            "CREATE TABLE ec_hnsw_insert_live_forward_links (id bigint primary key, embedding ecvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "CREATE INDEX ec_hnsw_insert_live_forward_links_idx ON ec_hnsw_insert_live_forward_links USING ec_hnsw \
             (embedding ecvector_ip_ops) WITH (m = 2)",
        )
        .expect("index creation should succeed");
        Spi::run(
            "INSERT INTO ec_hnsw_insert_live_forward_links VALUES
             (1, encode_to_ecvector(ARRAY[1.0, 0.0, 0.5, -1.0], 4, 42))",
        )
        .expect("seed insert should succeed");

        let index_oid = Spi::get_one::<pg_sys::Oid>(
            "SELECT 'ec_hnsw_insert_live_forward_links_idx'::regclass::oid",
        )
        .expect("SPI query should succeed")
        .expect("index oid should exist");
        let row1_heap_tid = heap_tid_for_row("ec_hnsw_insert_live_forward_links", 1);
        let (_metadata, elements, _neighbors) =
            decode_index_elements_and_neighbors(index_oid, code_len(4, 4));
        let (row1_element_tid, _) = find_element_for_heap_tid(&elements, row1_heap_tid);

        Spi::run(
            "INSERT INTO ec_hnsw_insert_live_forward_links VALUES
             (2, encode_to_ecvector(ARRAY[0.9, 0.1, 0.25, -0.9], 4, 42))",
        )
        .expect("second insert should succeed");

        let row2_heap_tid = heap_tid_for_row("ec_hnsw_insert_live_forward_links", 2);
        let (metadata, elements, neighbors) =
            decode_index_elements_and_neighbors(index_oid, code_len(4, 4));
        let (row1_element_tid_after, row1_element) =
            find_element_for_heap_tid(&elements, row1_heap_tid);
        let (row2_element_tid, row2_element) = find_element_for_heap_tid(&elements, row2_heap_tid);
        let row1_neighbors = neighbors
            .get(&row1_element.neighbortid)
            .expect("seed element neighbor tuple should exist");
        let row2_neighbors = neighbors
            .get(&row2_element.neighbortid)
            .expect("second insert neighbor tuple should exist");
        let row1_layer0 = layer_neighbor_slice(&row1_neighbors.tids, usize::from(metadata.m), 0);
        let row2_layer0 = layer_neighbor_slice(&row2_neighbors.tids, usize::from(metadata.m), 0);
        let populated_layer0_slots = row2_layer0
            .iter()
            .take(usize::from(metadata.m))
            .copied()
            .filter(|tid| *tid != am::page::ItemPointer::INVALID)
            .collect::<Vec<_>>();

        assert_eq!(
            populated_layer0_slots,
            vec![row1_element_tid],
            "the second live insert should seed its forward links from the existing entry element",
        );
        assert_eq!(
            row1_element_tid_after, row1_element_tid,
            "the seed element tid should remain stable after the live insert",
        );
        assert!(
            row1_layer0.contains(&row2_element_tid),
            "the seeded element should receive a layer-0 backlink to the newly inserted node",
        );
        assert!(
            row2_layer0
                .iter()
                .skip(usize::from(metadata.m))
                .all(|tid| *tid == am::page::ItemPointer::INVALID),
            "the upper-layer checkpoint still leaves the second half of the layer-0 forward window invalid on the new node",
        );
    }

    #[pg_test]
    fn test_ech_insert_populates_forward_links_against_built_graph() {
        Spi::run(
            "CREATE TABLE ec_hnsw_insert_built_forward_links (id bigint primary key, embedding ecvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO ec_hnsw_insert_built_forward_links VALUES
             (1, encode_to_ecvector(ARRAY[1.0, 0.0, 0.0, 0.0], 4, 42)),
             (2, encode_to_ecvector(ARRAY[0.0, 1.0, 0.0, 0.0], 4, 42)),
             (3, encode_to_ecvector(ARRAY[0.0, 0.0, 1.0, 0.0], 4, 42)),
             (4, encode_to_ecvector(ARRAY[0.0, 0.0, 0.0, 1.0], 4, 42))",
        )
        .expect("seed inserts should succeed");
        Spi::run(
            "CREATE INDEX ec_hnsw_insert_built_forward_links_idx ON ec_hnsw_insert_built_forward_links USING ec_hnsw \
             (embedding ecvector_ip_ops) WITH (m = 2)",
        )
        .expect("index creation should succeed");

        let index_oid = Spi::get_one::<pg_sys::Oid>(
            "SELECT 'ec_hnsw_insert_built_forward_links_idx'::regclass::oid",
        )
        .expect("SPI query should succeed")
        .expect("index oid should exist");
        let (_metadata, before_elements, _before_neighbors) =
            decode_index_elements_and_neighbors(index_oid, code_len(4, 4));
        let existing_element_tids = before_elements
            .iter()
            .map(|(tid, _)| *tid)
            .collect::<HashSet<_>>();

        Spi::run(
            "INSERT INTO ec_hnsw_insert_built_forward_links VALUES
             (5, encode_to_ecvector(ARRAY[1.0, 0.2, 0.1, 0.0], 4, 42))",
        )
        .expect("live insert should succeed");

        let row5_heap_tid = heap_tid_for_row("ec_hnsw_insert_built_forward_links", 5);
        let (metadata, elements, neighbors) =
            decode_index_elements_and_neighbors(index_oid, code_len(4, 4));
        let (row5_element_tid, row5_element) = find_element_for_heap_tid(&elements, row5_heap_tid);
        let row5_neighbors = neighbors
            .get(&row5_element.neighbortid)
            .expect("inserted element neighbor tuple should exist");
        let row5_layer0 = layer_neighbor_slice(&row5_neighbors.tids, usize::from(metadata.m), 0);
        let populated_layer0_slots = row5_layer0
            .iter()
            .take(usize::from(metadata.m))
            .copied()
            .filter(|tid| *tid != am::page::ItemPointer::INVALID)
            .collect::<Vec<_>>();

        assert!(
            !populated_layer0_slots.is_empty(),
            "live insert into a built graph should materialize at least one forward link",
        );
        assert!(
            populated_layer0_slots
                .iter()
                .all(|tid| existing_element_tids.contains(tid)),
            "forward links should target pre-existing graph elements in this one-way slice",
        );
        assert!(
            populated_layer0_slots
                .iter()
                .all(|tid| *tid != row5_element_tid),
            "forward links must not self-reference the newly inserted element",
        );
        assert!(
            row5_layer0
                .iter()
                .skip(usize::from(metadata.m))
                .all(|tid| *tid == am::page::ItemPointer::INVALID),
            "the second half of the layer-0 forward window stays invalid even after upper-layer links land",
        );
    }

    #[pg_test]
    fn test_ech_insert_populates_upper_layer_links_when_available() {
        Spi::run(
            "CREATE TABLE ec_hnsw_insert_upper_layer_links (id bigint primary key, embedding ecvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "CREATE INDEX ec_hnsw_insert_upper_layer_links_idx ON ec_hnsw_insert_upper_layer_links USING ec_hnsw \
             (embedding ecvector_ip_ops) WITH (m = 2)",
        )
        .expect("index creation should succeed");

        let index_oid = Spi::get_one::<pg_sys::Oid>(
            "SELECT 'ec_hnsw_insert_upper_layer_links_idx'::regclass::oid",
        )
        .expect("SPI query should succeed")
        .expect("index oid should exist");

        let mut chosen_insert = None;
        for id in 1_i64..=192_i64 {
            let previous_metadata = unsafe { am::debug_index_metadata(index_oid) }.3;
            Spi::run(&format!(
                "INSERT INTO ec_hnsw_insert_upper_layer_links VALUES (
                    {id},
                    encode_to_ecvector(ARRAY[
                        {id}.0,
                        {two}.0,
                        {three}.0,
                        {four}.0
                    ], 4, 42)
                )",
                id = id,
                two = id * 2,
                three = id * 3,
                four = id * 4,
            ))
            .expect("live insert should succeed");

            let heap_tid = heap_tid_for_row("ec_hnsw_insert_upper_layer_links", id);
            let level = am::debug_insert_level_for_heap_tid(2, 42, heap_tid, code_len(4, 4));
            if previous_metadata.max_level > 0 && level > 0 {
                chosen_insert = Some((id, level));
                break;
            }
        }

        let (inserted_id, expected_level) = chosen_insert
            .expect("deterministic insert levels should produce an upper-layer live insert");
        let inserted_heap_tid = heap_tid_for_row("ec_hnsw_insert_upper_layer_links", inserted_id);
        let (metadata, elements, neighbors) =
            decode_index_elements_and_neighbors(index_oid, code_len(4, 4));
        let (inserted_element_tid, inserted_element) =
            find_element_for_heap_tid(&elements, inserted_heap_tid);
        let inserted_neighbors = neighbors
            .get(&inserted_element.neighbortid)
            .expect("inserted upper-layer neighbor tuple should exist");
        assert_eq!(inserted_element.level, expected_level);
        assert!(
            inserted_element.level > 0,
            "the chosen live insert must participate in at least one upper layer",
        );

        let layer1_forward_tids =
            layer_neighbor_slice(&inserted_neighbors.tids, usize::from(metadata.m), 1)
                .iter()
                .copied()
                .filter(|tid| *tid != am::page::ItemPointer::INVALID)
                .collect::<Vec<_>>();
        assert!(
            !layer1_forward_tids.is_empty(),
            "an upper-layer live insert should populate at least one layer-1 forward link",
        );

        let mut layer1_backlink_targets = 0_usize;
        for forward_tid in layer1_forward_tids {
            let (_, forward_element) = elements
                .iter()
                .find(|(tid, _)| *tid == forward_tid)
                .expect("upper-layer forward link should target an existing element");
            assert!(
                forward_element.level >= 1,
                "upper-layer forward links must target elements that participate in layer 1",
            );

            let forward_neighbors = neighbors
                .get(&forward_element.neighbortid)
                .expect("upper-layer forward target neighbor tuple should exist");
            if layer_neighbor_slice(&forward_neighbors.tids, usize::from(metadata.m), 1)
                .contains(&inserted_element_tid)
            {
                layer1_backlink_targets += 1;
            }
        }
        assert!(
            layer1_backlink_targets > 0,
            "at least one sparse layer-1 forward target should receive a matching layer-1 backlink to the new element",
        );
        assert!(
            layer_neighbor_slice(&inserted_neighbors.tids, usize::from(metadata.m), 0)
                .iter()
                .skip(usize::from(metadata.m))
                .all(|tid| *tid == am::page::ItemPointer::INVALID),
            "upper-layer link coverage should not change the second half of the layer-0 forward window",
        );
    }

    #[pg_test]
    fn test_ech_insert_rewrites_full_layer0_backlink_slice() {
        Spi::run(
            "CREATE TABLE ec_hnsw_insert_full_layer0_backlink (id bigint primary key, embedding ecvector)",
        )
        .expect("table creation should succeed");
        for id in 1_i64..=12_i64 {
            let delta = id as f32 * 0.08;
            let z = if id % 2 == 0 { 0.25 } else { -0.25 };
            Spi::run(&format!(
                "INSERT INTO ec_hnsw_insert_full_layer0_backlink VALUES
                 ({id}, encode_to_ecvector(ARRAY[1.0, {delta}, {z}, 0.0], 4, 42))",
            ))
            .expect("seed insert should succeed");
        }
        Spi::run(
            "CREATE INDEX ec_hnsw_insert_full_layer0_backlink_idx ON ec_hnsw_insert_full_layer0_backlink USING ec_hnsw \
             (embedding ecvector_ip_ops) WITH (m = 2)",
        )
        .expect("index creation should succeed");

        let index_oid = Spi::get_one::<pg_sys::Oid>(
            "SELECT 'ec_hnsw_insert_full_layer0_backlink_idx'::regclass::oid",
        )
        .expect("SPI query should succeed")
        .expect("index oid should exist");
        for id in 13_i64..=40_i64 {
            let (_metadata_before, elements_before, neighbors_before) =
                decode_index_elements_and_neighbors(index_oid, code_len(4, 4));
            let delta = (id - 12) as f32 * 0.04;
            let w = (id - 12) as f32 * 0.02;
            let z = if id % 2 == 0 { 0.25 } else { -0.25 };
            Spi::run(&format!(
                "INSERT INTO ec_hnsw_insert_full_layer0_backlink VALUES
                 ({id}, encode_to_ecvector(ARRAY[1.0, {delta}, {z}, {w}], 4, 42))",
            ))
            .expect("live insert should succeed");

            let inserted_heap_tid = heap_tid_for_row("ec_hnsw_insert_full_layer0_backlink", id);
            let (metadata, elements_after, neighbors_after) =
                decode_index_elements_and_neighbors(index_oid, code_len(4, 4));
            let (inserted_element_tid, inserted_element) =
                find_element_for_heap_tid(&elements_after, inserted_heap_tid);
            let inserted_neighbors = neighbors_after
                .get(&inserted_element.neighbortid)
                .expect("inserted element neighbor tuple should exist");
            let inserted_layer0_forward_tids =
                layer_neighbor_slice(&inserted_neighbors.tids, usize::from(metadata.m), 0)
                    .iter()
                    .take(usize::from(metadata.m))
                    .copied()
                    .filter(|tid| *tid != am::page::ItemPointer::INVALID)
                    .collect::<Vec<_>>();

            for forward_tid in inserted_layer0_forward_tids {
                let (_, before_element) = elements_before
                    .iter()
                    .find(|(tid, _)| *tid == forward_tid)
                    .expect("forward target should exist before the live insert");
                let before_neighbors = neighbors_before
                    .get(&before_element.neighbortid)
                    .expect("forward target neighbor tuple should exist before the live insert");
                let before_layer0 =
                    layer_neighbor_slice(&before_neighbors.tids, usize::from(metadata.m), 0);
                if before_layer0.contains(&am::page::ItemPointer::INVALID) {
                    continue;
                }

                let (_, after_element) = elements_after
                    .iter()
                    .find(|(tid, _)| *tid == forward_tid)
                    .expect("forward target should still exist after the live insert");
                let after_neighbors = neighbors_after
                    .get(&after_element.neighbortid)
                    .expect("forward target neighbor tuple should exist after the live insert");
                let after_layer0 =
                    layer_neighbor_slice(&after_neighbors.tids, usize::from(metadata.m), 0);
                if !after_layer0.contains(&inserted_element_tid) {
                    continue;
                }

                assert!(
                    after_layer0
                        .iter()
                        .all(|tid| *tid != am::page::ItemPointer::INVALID),
                    "overflow rewrite should preserve the full 2M layer-0 capacity on selected targets",
                );
                assert_ne!(
                    after_layer0, before_layer0,
                    "admitting the new element into a full layer-0 target should evict at least one prior neighbor",
                );
                return;
            }
        }

        panic!("expected a bounded live-insert search to rewrite at least one full layer-0 target");
    }

    #[pg_test]
    fn test_ech_live_insert_is_graph_reachable_via_backlinks() {
        Spi::run(
            "CREATE TABLE ec_hnsw_insert_graph_reachable (id bigint primary key, embedding ecvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO ec_hnsw_insert_graph_reachable VALUES
             (1, encode_to_ecvector(ARRAY[1.0, 0.0, 0.0, 0.0], 4, 42)),
             (2, encode_to_ecvector(ARRAY[0.0, 1.0, 0.0, 0.0], 4, 42)),
             (3, encode_to_ecvector(ARRAY[0.0, 0.0, 1.0, 0.0], 4, 42)),
             (4, encode_to_ecvector(ARRAY[0.0, 0.0, 0.0, 1.0], 4, 42))",
        )
        .expect("seed inserts should succeed");
        Spi::run(
            "CREATE INDEX ec_hnsw_insert_graph_reachable_idx ON ec_hnsw_insert_graph_reachable USING ec_hnsw \
             (embedding ecvector_ip_ops) WITH (m = 8, ef_search = 8)",
        )
        .expect("index creation should succeed");

        let index_oid = Spi::get_one::<pg_sys::Oid>(
            "SELECT 'ec_hnsw_insert_graph_reachable_idx'::regclass::oid",
        )
        .expect("SPI query should succeed")
        .expect("index oid should exist");

        let mut chosen_insert = None;
        for id in 5_i64..=32_i64 {
            let delta = (id - 4) as f32 * 0.02;
            Spi::run(&format!(
                "INSERT INTO ec_hnsw_insert_graph_reachable VALUES
                 ({id}, encode_to_ecvector(ARRAY[1.0, {delta}, 0.1, 0.0], 4, 42))",
            ))
            .expect("live insert should succeed");

            let heap_tid = heap_tid_for_row("ec_hnsw_insert_graph_reachable", id);
            let level = am::debug_insert_level_for_heap_tid(8, 42, heap_tid, code_len(4, 4));
            if level == 0 {
                chosen_insert = Some((id, vec![1.0, delta, 0.1, 0.0]));
                break;
            }
        }

        let (inserted_id, query) = chosen_insert
            .expect("deterministic insert levels should produce a level-0 live insert");
        let inserted_heap_tid = heap_tid_for_row("ec_hnsw_insert_graph_reachable", inserted_id);
        let (metadata, elements, _neighbors) =
            decode_index_elements_and_neighbors(index_oid, code_len(4, 4));
        let (inserted_element_tid, _inserted_element) =
            find_element_for_heap_tid(&elements, inserted_heap_tid);
        assert_ne!(
            metadata.entry_point, inserted_element_tid,
            "the reachability check should exercise backlinks rather than an entry-point promotion",
        );

        let (_head, frontier, frontier_slots, _frontier_provenance, _expanded_sources) =
            unsafe { am::debug_rescan_candidate_frontier(index_oid, query) };
        let frontier_tids = frontier_slots
            .iter()
            .filter_map(|(valid, tid, _)| valid.then_some(*tid))
            .collect::<Vec<_>>();

        assert!(
            !frontier.is_empty(),
            "reachable live inserts should contribute to a non-empty graph frontier",
        );
        assert!(
            frontier_tids.contains(&(inserted_element_tid.block_number, inserted_element_tid.offset_number)),
            "the graph-seeded runtime frontier should reach the live-inserted element before any linear fallback",
        );
    }

    #[pg_test]
    fn test_ech_vacuum_callbacks_are_benign_noops() {
        Spi::run("CREATE TABLE ec_hnsw_vacuum_noop (id bigint primary key, embedding ecvector)")
            .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO ec_hnsw_vacuum_noop VALUES
             (1, encode_to_ecvector(ARRAY[1.0, 0.0, 0.5, -1.0], 4, 42)),
             (2, encode_to_ecvector(ARRAY[0.5, 1.0, -0.5, 0.25], 4, 42)),
             (3, encode_to_ecvector(ARRAY[-1.0, 0.5, 0.25, 0.75], 4, 42))",
        )
        .expect("seed inserts should succeed");
        Spi::run(
            "CREATE INDEX ec_hnsw_vacuum_noop_idx ON ec_hnsw_vacuum_noop USING ec_hnsw \
             (embedding ecvector_ip_ops)",
        )
        .expect("index creation should succeed");
        Spi::run("DELETE FROM ec_hnsw_vacuum_noop WHERE id = 2").expect("delete should succeed");

        let index_oid =
            Spi::get_one::<pg_sys::Oid>("SELECT 'ec_hnsw_vacuum_noop_idx'::regclass::oid")
                .expect("SPI query should succeed")
                .expect("index oid should exist");
        let (block_count, metadata, data_pages) = unsafe { am::debug_index_pages(index_oid) };
        let element_tuple_count =
            decode_turboquant_elements_from_pages(&metadata, &data_pages, code_len(4, 4)).len();

        let stats = unsafe { am::debug_vacuum_stats(index_oid) };
        assert_eq!(stats.num_pages, block_count);
        assert!(
            !stats.estimated_count,
            "vacuum stats should report exact tuple counts"
        );
        assert_eq!(stats.num_index_tuples, element_tuple_count as f64);
        assert_eq!(stats.tuples_removed, 0.0);
        assert_eq!(stats.pages_newly_deleted, 0);
        assert_eq!(stats.pages_deleted, 0);
        assert_eq!(stats.pages_free, 0);
    }

    #[pg_test]
    fn test_ech_vacuum_callbacks_handle_empty_index() {
        Spi::run("CREATE TABLE ec_hnsw_vacuum_empty (id bigint primary key, embedding ecvector)")
            .expect("table creation should succeed");
        Spi::run(
            "CREATE INDEX ec_hnsw_vacuum_empty_idx ON ec_hnsw_vacuum_empty USING ec_hnsw \
             (embedding ecvector_ip_ops)",
        )
        .expect("index creation should succeed");

        let index_oid =
            Spi::get_one::<pg_sys::Oid>("SELECT 'ec_hnsw_vacuum_empty_idx'::regclass::oid")
                .expect("SPI query should succeed")
                .expect("index oid should exist");
        let (block_count, _metadata, data_pages) = unsafe { am::debug_index_pages(index_oid) };
        assert_eq!(data_pages.len(), 0, "empty index should have no data pages");

        let stats = unsafe { am::debug_vacuum_stats(index_oid) };
        assert_eq!(stats.num_pages, block_count);
        assert!(
            !stats.estimated_count,
            "vacuum stats should report exact tuple counts"
        );
        assert_eq!(stats.num_index_tuples, 0.0);
        assert_eq!(stats.tuples_removed, 0.0);
        assert_eq!(stats.pages_newly_deleted, 0);
        assert_eq!(stats.pages_deleted, 0);
        assert_eq!(stats.pages_free, 0);
    }

    #[pg_test]
    fn test_ech_vacuum_callbacks_are_stable_across_repeated_calls() {
        Spi::run("CREATE TABLE ec_hnsw_vacuum_repeat (id bigint primary key, embedding ecvector)")
            .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO ec_hnsw_vacuum_repeat VALUES
             (1, encode_to_ecvector(ARRAY[1.0, 0.0, 0.5, -1.0], 4, 42)),
             (2, encode_to_ecvector(ARRAY[0.5, 1.0, -0.5, 0.25], 4, 42)),
             (3, encode_to_ecvector(ARRAY[-1.0, 0.5, 0.25, 0.75], 4, 42))",
        )
        .expect("seed inserts should succeed");
        Spi::run(
            "CREATE INDEX ec_hnsw_vacuum_repeat_idx ON ec_hnsw_vacuum_repeat USING ec_hnsw \
             (embedding ecvector_ip_ops)",
        )
        .expect("index creation should succeed");
        Spi::run("DELETE FROM ec_hnsw_vacuum_repeat WHERE id = 2").expect("delete should succeed");

        let index_oid =
            Spi::get_one::<pg_sys::Oid>("SELECT 'ec_hnsw_vacuum_repeat_idx'::regclass::oid")
                .expect("SPI query should succeed")
                .expect("index oid should exist");
        let first_stats = unsafe { am::debug_vacuum_stats(index_oid) };
        let second_stats = unsafe { am::debug_vacuum_stats(index_oid) };

        assert_eq!(second_stats.num_pages, first_stats.num_pages);
        assert_eq!(second_stats.estimated_count, first_stats.estimated_count);
        assert_eq!(second_stats.num_index_tuples, first_stats.num_index_tuples);
        assert_eq!(second_stats.tuples_removed, first_stats.tuples_removed);
        assert_eq!(
            second_stats.pages_newly_deleted,
            first_stats.pages_newly_deleted
        );
        assert_eq!(second_stats.pages_deleted, first_stats.pages_deleted);
        assert_eq!(second_stats.pages_free, first_stats.pages_free);
    }

    #[pg_test]
    fn test_ech_vacuum_pass1_compacts_duplicate_heaptids() {
        Spi::run(
            "CREATE TABLE ec_hnsw_vacuum_pass1_duplicates (id bigint primary key, embedding ecvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO ec_hnsw_vacuum_pass1_duplicates VALUES
             (1, encode_to_ecvector(ARRAY[1.0, 0.0, 0.5, -1.0], 4, 42)),
             (2, encode_to_ecvector(ARRAY[1.0, 0.0, 0.5, -1.0], 4, 42)),
             (3, encode_to_ecvector(ARRAY[-1.0, 0.5, 0.25, 0.75], 4, 42))",
        )
        .expect("seed inserts should succeed");
        Spi::run(
            "CREATE INDEX ec_hnsw_vacuum_pass1_duplicates_idx ON ec_hnsw_vacuum_pass1_duplicates USING ec_hnsw \
             (embedding ecvector_ip_ops)",
        )
        .expect("index creation should succeed");

        let survivor_heap_tid = heap_tid_for_row("ec_hnsw_vacuum_pass1_duplicates", 1);
        let deleted_heap_tid = heap_tid_for_row("ec_hnsw_vacuum_pass1_duplicates", 2);
        Spi::run("DELETE FROM ec_hnsw_vacuum_pass1_duplicates WHERE id = 2")
            .expect("delete should succeed");

        let index_oid = Spi::get_one::<pg_sys::Oid>(
            "SELECT 'ec_hnsw_vacuum_pass1_duplicates_idx'::regclass::oid",
        )
        .expect("SPI query should succeed")
        .expect("index oid should exist");
        let stats = unsafe { am::debug_vacuum_remove_heap_tids(index_oid, &[deleted_heap_tid]) };
        let (_metadata, elements, _neighbors) =
            decode_index_elements_and_neighbors(index_oid, code_len(4, 4));
        let (_element_tid, duplicate_element) =
            find_element_for_heap_tid(&elements, survivor_heap_tid);

        assert_eq!(
            duplicate_element.heaptids,
            vec![survivor_heap_tid],
            "pass 1 should compact the duplicate element to the surviving heap tid",
        );
        assert!(
            elements
                .iter()
                .all(|(_, element)| !element.heaptids.contains(&deleted_heap_tid)),
            "pass 1 should remove the deleted heap tid from every element payload",
        );
        assert_eq!(stats.tuples_removed, 1.0);
        assert_eq!(
            stats.num_index_tuples, 2.0,
            "amvacuumcleanup should report the remaining live element count",
        );
    }

    #[pg_test]
    fn test_ech_vacuum_pass1_makes_deleted_row_unreachable() {
        Spi::run(
            "CREATE TABLE ec_hnsw_vacuum_pass1_scan (id bigint primary key, embedding ecvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO ec_hnsw_vacuum_pass1_scan VALUES
             (1, encode_to_ecvector(ARRAY[1.0, 0.0, 0.5, -1.0], 4, 42)),
             (2, encode_to_ecvector(ARRAY[0.5, 1.0, -0.5, 0.25], 4, 42)),
             (3, encode_to_ecvector(ARRAY[-1.0, 0.5, 0.25, 0.75], 4, 42))",
        )
        .expect("seed inserts should succeed");
        Spi::run(
            "CREATE INDEX ec_hnsw_vacuum_pass1_scan_idx ON ec_hnsw_vacuum_pass1_scan USING ec_hnsw \
             (embedding ecvector_ip_ops)",
        )
        .expect("index creation should succeed");

        let deleted_heap_tid = heap_tid_for_row("ec_hnsw_vacuum_pass1_scan", 2);
        Spi::run("DELETE FROM ec_hnsw_vacuum_pass1_scan WHERE id = 2")
            .expect("delete should succeed");

        let index_oid =
            Spi::get_one::<pg_sys::Oid>("SELECT 'ec_hnsw_vacuum_pass1_scan_idx'::regclass::oid")
                .expect("SPI query should succeed")
                .expect("index oid should exist");
        let stats = unsafe { am::debug_vacuum_remove_heap_tids(index_oid, &[deleted_heap_tid]) };
        let (_metadata, elements, _neighbors) =
            decode_index_elements_and_neighbors(index_oid, code_len(4, 4));
        let deleted_element = elements
            .iter()
            .find(|(_, element)| {
                element.code
                    == encoded_code_bytes(
                        ProdQuantizer::new(4, 4, 42).encode(&[0.5, 1.0, -0.5, 0.25]),
                    )
            })
            .expect("deleted element should still be present after vacuum finalization");

        assert!(
            deleted_element.1.heaptids.is_empty(),
            "pass 1 should clear the last heap tid from a fully dead element",
        );
        assert!(
            deleted_element.1.deleted,
            "vacuum should finalize a fully dead element once pass 1 strips its last heap tid",
        );

        let returned =
            unsafe { am::debug_gettuple_scan_heap_tids(index_oid, vec![0.5, 1.0, -0.5, 0.25]) };
        assert!(
            !returned.contains(&(
                deleted_heap_tid.block_number,
                deleted_heap_tid.offset_number
            )),
            "graph/runtime scans should skip elements whose heap tid array is empty after pass 1",
        );
        assert_eq!(stats.tuples_removed, 1.0);
        assert_eq!(stats.num_index_tuples, 2.0);
    }

    #[pg_test]
    fn test_ech_vacuum_repairs_deleted_entry_point_metadata() {
        let table_name = "ec_hnsw_vacuum_entry_repair";
        let index_name = "ec_hnsw_vacuum_entry_repair_idx";
        Spi::run(&format!(
            "CREATE TABLE {table_name} (id bigint primary key, embedding ecvector)"
        ))
        .expect("table creation should succeed");
        Spi::run(&format!(
            "INSERT INTO {table_name} VALUES
             (1, encode_to_ecvector(ARRAY[1.0, 0.0, 0.5, -1.0], 4, 42)),
             (2, encode_to_ecvector(ARRAY[0.5, 1.0, -0.5, 0.25], 4, 42)),
             (3, encode_to_ecvector(ARRAY[-1.0, 0.5, 0.25, 0.75], 4, 42)),
             (4, encode_to_ecvector(ARRAY[0.25, -0.75, 1.0, 0.5], 4, 42)),
             (5, encode_to_ecvector(ARRAY[-0.5, -1.0, 0.75, 0.25], 4, 42))"
        ))
        .expect("seed inserts should succeed");
        Spi::run(&format!(
            "CREATE INDEX {index_name} ON {table_name} USING ec_hnsw \
             (embedding ecvector_ip_ops)"
        ))
        .expect("index creation should succeed");

        let index_oid =
            Spi::get_one::<pg_sys::Oid>(&format!("SELECT '{index_name}'::regclass::oid"))
                .expect("SPI query should succeed")
                .expect("index oid should exist");
        let (metadata_before, elements_before, _neighbors_before) =
            decode_index_elements_and_neighbors(index_oid, code_len(4, 4));
        let deleted_element_before = elements_before
            .iter()
            .find(|(tid, _)| *tid == metadata_before.entry_point)
            .expect("metadata entry point should identify a live element before vacuum");
        let deleted_heap_tid = *deleted_element_before
            .1
            .heaptids
            .first()
            .expect("entry-point element should carry a heap tid");

        let ctid_to_id = ctid_id_map(table_name);
        let deleted_row_id = *ctid_to_id
            .get(&(
                deleted_heap_tid.block_number,
                deleted_heap_tid.offset_number,
            ))
            .expect("entry-point heap tid should map back to a table row");
        Spi::run(&format!(
            "DELETE FROM {table_name} WHERE id = {deleted_row_id}"
        ))
        .expect("delete should succeed");

        unsafe { am::debug_vacuum_remove_heap_tids(index_oid, &[deleted_heap_tid]) };

        let (metadata_after, elements_after, _neighbors_after) =
            decode_index_elements_and_neighbors(index_oid, code_len(4, 4));
        assert_ne!(
            metadata_after.entry_point, deleted_element_before.0,
            "vacuum should replace a deleted metadata entry point instead of leaving it behind",
        );
        assert_ne!(
            metadata_after.entry_point,
            am::page::ItemPointer::INVALID,
            "vacuum should keep advertising a live entry point while live elements remain",
        );
        let repaired_entry = elements_after
            .iter()
            .find(|(tid, _)| *tid == metadata_after.entry_point)
            .expect("repaired metadata entry point should identify an on-disk element");
        assert!(
            !repaired_entry.1.deleted && !repaired_entry.1.heaptids.is_empty(),
            "repaired metadata entry point should identify a live element",
        );
        assert_eq!(
            repaired_entry.1.level, metadata_after.max_level,
            "vacuum should keep metadata.max_level aligned with the repaired live entry point",
        );
    }

    #[pg_test]
    fn test_ech_scan_falls_back_from_stale_entry_metadata() {
        fn fixture_query(id: usize) -> Vec<f32> {
            match id {
                1 => vec![1.0, 0.0, 0.5, -1.0],
                2 => vec![0.5, 1.0, -0.5, 0.25],
                3 => vec![-1.0, 0.5, 0.25, 0.75],
                4 => vec![0.25, -0.75, 1.0, 0.5],
                5 => vec![-0.5, -1.0, 0.75, 0.25],
                other => panic!("unexpected fixture row id {other}"),
            }
        }

        let table_name = "ec_hnsw_scan_stale_entry_fallback";
        let index_name = "ec_hnsw_scan_stale_entry_fallback_idx";
        Spi::run(&format!(
            "CREATE TABLE {table_name} (id bigint primary key, embedding ecvector)"
        ))
        .expect("table creation should succeed");
        Spi::run(&format!(
            "INSERT INTO {table_name} VALUES
             (1, encode_to_ecvector(ARRAY[1.0, 0.0, 0.5, -1.0], 4, 42)),
             (2, encode_to_ecvector(ARRAY[0.5, 1.0, -0.5, 0.25], 4, 42)),
             (3, encode_to_ecvector(ARRAY[-1.0, 0.5, 0.25, 0.75], 4, 42)),
             (4, encode_to_ecvector(ARRAY[0.25, -0.75, 1.0, 0.5], 4, 42)),
             (5, encode_to_ecvector(ARRAY[-0.5, -1.0, 0.75, 0.25], 4, 42))"
        ))
        .expect("seed inserts should succeed");
        Spi::run(&format!(
            "CREATE INDEX {index_name} ON {table_name} USING ec_hnsw \
             (embedding ecvector_ip_ops)"
        ))
        .expect("index creation should succeed");

        let index_oid =
            Spi::get_one::<pg_sys::Oid>(&format!("SELECT '{index_name}'::regclass::oid"))
                .expect("SPI query should succeed")
                .expect("index oid should exist");
        let (metadata_before, elements_before, _neighbors_before) =
            decode_index_elements_and_neighbors(index_oid, code_len(4, 4));
        let deleted_element_before = elements_before
            .iter()
            .find(|(tid, _)| *tid == metadata_before.entry_point)
            .expect("metadata entry point should identify a live element before vacuum");
        let deleted_heap_tid = *deleted_element_before
            .1
            .heaptids
            .first()
            .expect("entry-point element should carry a heap tid");
        let deleted_level = deleted_element_before.1.level;

        let ctid_to_id = ctid_id_map(table_name);
        let deleted_row_id = *ctid_to_id
            .get(&(
                deleted_heap_tid.block_number,
                deleted_heap_tid.offset_number,
            ))
            .expect("entry-point heap tid should map back to a table row");
        Spi::run(&format!(
            "DELETE FROM {table_name} WHERE id = {deleted_row_id}"
        ))
        .expect("delete should succeed");

        unsafe { am::debug_vacuum_remove_heap_tids(index_oid, &[deleted_heap_tid]) };

        let (_block_count, _m, _ef_construction, mut stale_metadata) =
            unsafe { am::debug_index_metadata(index_oid) };
        stale_metadata.entry_point = deleted_element_before.0;
        stale_metadata.max_level = deleted_level;
        unsafe { am::debug_update_index_metadata(index_oid, stale_metadata) };

        let returned =
            unsafe { am::debug_gettuple_scan_heap_tids(index_oid, fixture_query(deleted_row_id)) };
        assert!(
            !returned.is_empty(),
            "scan should fall back to another live seed when metadata.entry_point is stale and deleted",
        );
        assert!(
            !returned.contains(&(
                deleted_heap_tid.block_number,
                deleted_heap_tid.offset_number
            )),
            "stale entry-point fallback should not re-emit the deleted heap row",
        );
    }

    #[pg_test]
    fn test_ech_vacuum_pass2_unlinks_deleted_neighbor_refs() {
        Spi::run(
            "CREATE TABLE ec_hnsw_vacuum_pass2_unlink (id bigint primary key, embedding ecvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO ec_hnsw_vacuum_pass2_unlink VALUES
             (1, encode_to_ecvector(ARRAY[1.0, 0.0, 0.5, -1.0], 4, 42)),
             (2, encode_to_ecvector(ARRAY[0.5, 1.0, -0.5, 0.25], 4, 42)),
             (3, encode_to_ecvector(ARRAY[-1.0, 0.5, 0.25, 0.75], 4, 42)),
             (4, encode_to_ecvector(ARRAY[0.25, -0.75, 1.0, 0.5], 4, 42)),
             (5, encode_to_ecvector(ARRAY[-0.5, -1.0, 0.75, 0.25], 4, 42))",
        )
        .expect("seed inserts should succeed");
        Spi::run(
            "CREATE INDEX ec_hnsw_vacuum_pass2_unlink_idx ON ec_hnsw_vacuum_pass2_unlink USING ec_hnsw \
             (embedding ecvector_ip_ops)",
        )
        .expect("index creation should succeed");

        let deleted_heap_tid = heap_tid_for_row("ec_hnsw_vacuum_pass2_unlink", 2);
        let index_oid =
            Spi::get_one::<pg_sys::Oid>("SELECT 'ec_hnsw_vacuum_pass2_unlink_idx'::regclass::oid")
                .expect("SPI query should succeed")
                .expect("index oid should exist");
        let (_metadata_before, elements_before, neighbors_before) =
            decode_index_elements_and_neighbors(index_oid, code_len(4, 4));
        let (deleted_element_tid, _deleted_element) =
            find_element_for_heap_tid(&elements_before, deleted_heap_tid);
        assert!(
            count_neighbor_refs(&neighbors_before, deleted_element_tid) > 0,
            "fixture should start with at least one persisted neighbor ref to the soon-to-be-deleted node",
        );

        Spi::run("DELETE FROM ec_hnsw_vacuum_pass2_unlink WHERE id = 2")
            .expect("delete should succeed");

        unsafe { am::debug_vacuum_remove_heap_tids(index_oid, &[deleted_heap_tid]) };

        let (_metadata_after, elements_after, neighbors_after) =
            decode_index_elements_and_neighbors(index_oid, code_len(4, 4));
        let (_, deleted_element_after) = elements_after
            .iter()
            .find(|(tid, _)| *tid == deleted_element_tid)
            .expect("deleted element tuple should remain on disk after vacuum");

        assert!(
            deleted_element_after.deleted,
            "vacuum should still finalize the fully dead element after pass-2 repair",
        );
        assert_eq!(
            count_neighbor_refs(&neighbors_after, deleted_element_tid),
            0,
            "pass 2 should remove every persisted neighbor ref to the deleted element tid",
        );
    }

    #[pg_test]
    fn test_ech_vacuum_pass2_layer0_replacement_fills_broken_edges() {
        Spi::run(
            "CREATE TABLE ec_hnsw_vacuum_pass2_replace (id bigint primary key, embedding ecvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO ec_hnsw_vacuum_pass2_replace VALUES
             (1, encode_to_ecvector(ARRAY[1.0, 0.0, 0.5, -1.0], 4, 42)),
             (2, encode_to_ecvector(ARRAY[0.9, 0.1, 0.45, -0.9], 4, 42)),
             (3, encode_to_ecvector(ARRAY[0.8, 0.2, 0.4, -0.8], 4, 42)),
             (4, encode_to_ecvector(ARRAY[0.7, 0.3, 0.35, -0.7], 4, 42)),
             (5, encode_to_ecvector(ARRAY[0.6, 0.4, 0.3, -0.6], 4, 42)),
             (6, encode_to_ecvector(ARRAY[0.5, 0.5, 0.25, -0.5], 4, 42)),
             (7, encode_to_ecvector(ARRAY[0.4, 0.6, 0.2, -0.4], 4, 42)),
             (8, encode_to_ecvector(ARRAY[0.3, 0.7, 0.15, -0.3], 4, 42))",
        )
        .expect("seed inserts should succeed");
        Spi::run(
            "CREATE INDEX ec_hnsw_vacuum_pass2_replace_idx ON ec_hnsw_vacuum_pass2_replace USING ec_hnsw \
             (embedding ecvector_ip_ops) WITH (m = 2)",
        )
        .expect("index creation should succeed");

        let index_oid =
            Spi::get_one::<pg_sys::Oid>("SELECT 'ec_hnsw_vacuum_pass2_replace_idx'::regclass::oid")
                .expect("SPI query should succeed")
                .expect("index oid should exist");
        let (metadata_before, elements_before, neighbors_before) =
            decode_index_elements_and_neighbors(index_oid, code_len(4, 4));
        let (deleted_row_id, deleted_heap_tid, deleted_element_tid, affected_before) =
            (1_i64..=8)
                .find_map(|id| {
                    let deleted_heap_tid = heap_tid_for_row("ec_hnsw_vacuum_pass2_replace", id);
                    let (deleted_element_tid, _) =
                        find_element_for_heap_tid(&elements_before, deleted_heap_tid);
                    let affected_before = elements_before
                        .iter()
                        .filter_map(|(element_tid, element)| {
                            if *element_tid == deleted_element_tid
                                || element.deleted
                                || element.heaptids.is_empty()
                            {
                                return None;
                            }

                            let neighbor = neighbors_before
                                .get(&element.neighbortid)
                                .expect("live element should have a persisted neighbor tuple");
                            let layer0 = layer_neighbor_slice(
                                &neighbor.tids,
                                usize::from(metadata_before.m),
                                0,
                            );
                            layer0.contains(&deleted_element_tid).then(|| {
                                (
                                    *element_tid,
                                    layer0
                                        .iter()
                                        .copied()
                                        .filter(|tid| {
                                            *tid != am::page::ItemPointer::INVALID
                                                && *tid != deleted_element_tid
                                        })
                                        .collect::<Vec<_>>(),
                                )
                            })
                        })
                        .collect::<Vec<_>>();

                    (!affected_before.is_empty())
                        .then_some((id, deleted_heap_tid, deleted_element_tid, affected_before))
                })
                .expect(
                    "fixture should provide at least one deletable row with a live inbound layer-0 edge",
                );

        Spi::run(&format!(
            "DELETE FROM ec_hnsw_vacuum_pass2_replace WHERE id = {deleted_row_id}"
        ))
        .expect("delete should succeed");
        unsafe { am::debug_vacuum_remove_heap_tids(index_oid, &[deleted_heap_tid]) };

        let (metadata_after, elements_after, neighbors_after) =
            decode_index_elements_and_neighbors(index_oid, code_len(4, 4));
        let mut replacement_filled = false;

        for (affected_tid, surviving_before) in affected_before {
            let (_, element_after) = elements_after
                .iter()
                .find(|(tid, _)| *tid == affected_tid)
                .expect("affected live element should remain on disk after vacuum");
            let neighbor_after = neighbors_after
                .get(&element_after.neighbortid)
                .expect("affected live element should keep a persisted neighbor tuple");
            let layer0_after =
                layer_neighbor_slice(&neighbor_after.tids, usize::from(metadata_after.m), 0);
            let surviving_after = layer0_after
                .iter()
                .copied()
                .filter(|tid| *tid != am::page::ItemPointer::INVALID)
                .collect::<Vec<_>>();

            if surviving_after
                .iter()
                .any(|tid| *tid != deleted_element_tid && !surviving_before.contains(tid))
            {
                replacement_filled = true;
                break;
            }
        }

        assert_eq!(
            count_neighbor_refs(&neighbors_after, deleted_element_tid),
            0,
            "vacuum replacement should still leave no persisted refs to the deleted element tid",
        );
        assert!(
            replacement_filled,
            "vacuum replacement search should fill at least one broken layer-0 edge with a new live candidate",
        );
    }

    #[pg_test]
    fn test_ech_vacuum_pass2_upper_replacement_fills_broken_edges() {
        Spi::run(
            "CREATE TABLE ec_hnsw_vacuum_pass2_upper_replace (id bigint primary key, embedding ecvector)",
        )
        .expect("table creation should succeed");
        for id in 1_i64..=192_i64 {
            Spi::run(&format!(
                "INSERT INTO ec_hnsw_vacuum_pass2_upper_replace VALUES (
                    {id},
                    encode_to_ecvector(ARRAY[
                        {id}.0,
                        {two}.0,
                        {three}.0,
                        {four}.0
                    ], 4, 42)
                )",
                id = id,
                two = id * 2,
                three = id * 3,
                four = id * 4,
            ))
            .expect("seed insert should succeed");
        }
        Spi::run(
            "CREATE INDEX ec_hnsw_vacuum_pass2_upper_replace_idx ON ec_hnsw_vacuum_pass2_upper_replace USING ec_hnsw \
             (embedding ecvector_ip_ops) WITH (m = 2)",
        )
        .expect("index creation should succeed");

        let index_oid = Spi::get_one::<pg_sys::Oid>(
            "SELECT 'ec_hnsw_vacuum_pass2_upper_replace_idx'::regclass::oid",
        )
        .expect("SPI query should succeed")
        .expect("index oid should exist");
        let (metadata_before, elements_before, neighbors_before) =
            decode_index_elements_and_neighbors(index_oid, code_len(4, 4));
        assert!(
            metadata_before.max_level > 0,
            "fixture should build at least one upper layer before vacuum repair runs",
        );

        let (deleted_row_id, deleted_heap_tid, deleted_element_tid, affected_before) =
            (1_i64..=192_i64)
                .find_map(|id| {
                    let deleted_heap_tid =
                        heap_tid_for_row("ec_hnsw_vacuum_pass2_upper_replace", id);
                    let (deleted_element_tid, _) =
                        find_element_for_heap_tid(&elements_before, deleted_heap_tid);
                    let affected_before = elements_before
                        .iter()
                        .filter_map(|(element_tid, element)| {
                            if *element_tid == deleted_element_tid
                                || element.deleted
                                || element.heaptids.is_empty()
                                || element.level < 1
                            {
                                return None;
                            }

                            let neighbor = neighbors_before
                                .get(&element.neighbortid)
                                .expect("live upper-layer element should have a persisted neighbor tuple");
                            let layer1 = layer_neighbor_slice(
                                &neighbor.tids,
                                usize::from(metadata_before.m),
                                1,
                            );
                            layer1.contains(&deleted_element_tid).then(|| {
                                (
                                    *element_tid,
                                    layer1
                                        .iter()
                                        .copied()
                                        .filter(|tid| {
                                            *tid != am::page::ItemPointer::INVALID
                                                && *tid != deleted_element_tid
                                        })
                                        .collect::<Vec<_>>(),
                                )
                            })
                        })
                        .collect::<Vec<_>>();

                    (!affected_before.is_empty())
                        .then_some((id, deleted_heap_tid, deleted_element_tid, affected_before))
                })
                .expect(
                    "fixture should provide at least one deletable row with a live inbound layer-1 edge",
                );

        Spi::run(&format!(
            "DELETE FROM ec_hnsw_vacuum_pass2_upper_replace WHERE id = {deleted_row_id}"
        ))
        .expect("delete should succeed");
        unsafe { am::debug_vacuum_remove_heap_tids(index_oid, &[deleted_heap_tid]) };

        let (metadata_after, elements_after, neighbors_after) =
            decode_index_elements_and_neighbors(index_oid, code_len(4, 4));
        let mut replacement_filled = false;

        for (affected_tid, surviving_before) in affected_before {
            let (_, element_after) = elements_after
                .iter()
                .find(|(tid, _)| *tid == affected_tid)
                .expect("affected live upper-layer element should remain on disk after vacuum");
            let neighbor_after = neighbors_after
                .get(&element_after.neighbortid)
                .expect("affected live upper-layer element should keep a persisted neighbor tuple");
            let layer1_after =
                layer_neighbor_slice(&neighbor_after.tids, usize::from(metadata_after.m), 1);
            let surviving_after = layer1_after
                .iter()
                .copied()
                .filter(|tid| *tid != am::page::ItemPointer::INVALID)
                .collect::<Vec<_>>();

            if surviving_after
                .iter()
                .any(|tid| *tid != deleted_element_tid && !surviving_before.contains(tid))
            {
                replacement_filled = true;
                break;
            }
        }

        assert_eq!(
            count_neighbor_refs(&neighbors_after, deleted_element_tid),
            0,
            "upper-layer vacuum replacement should still leave no persisted refs to the deleted element tid",
        );
        assert!(
            replacement_filled,
            "vacuum replacement search should fill at least one broken upper-layer edge with a new live candidate",
        );
    }

    #[pg_test]
    fn test_ech_vacuum_pass1_is_stable_across_repeated_replays() {
        Spi::run(
            "CREATE TABLE ec_hnsw_vacuum_pass1_repeat (id bigint primary key, embedding ecvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO ec_hnsw_vacuum_pass1_repeat VALUES
             (1, encode_to_ecvector(ARRAY[1.0, 0.0, 0.5, -1.0], 4, 42)),
             (2, encode_to_ecvector(ARRAY[0.5, 1.0, -0.5, 0.25], 4, 42)),
             (3, encode_to_ecvector(ARRAY[-1.0, 0.5, 0.25, 0.75], 4, 42))",
        )
        .expect("seed inserts should succeed");
        Spi::run(
            "CREATE INDEX ec_hnsw_vacuum_pass1_repeat_idx ON ec_hnsw_vacuum_pass1_repeat USING ec_hnsw \
             (embedding ecvector_ip_ops)",
        )
        .expect("index creation should succeed");

        let deleted_heap_tid = heap_tid_for_row("ec_hnsw_vacuum_pass1_repeat", 2);
        let index_oid =
            Spi::get_one::<pg_sys::Oid>("SELECT 'ec_hnsw_vacuum_pass1_repeat_idx'::regclass::oid")
                .expect("SPI query should succeed")
                .expect("index oid should exist");
        let (_metadata_before, elements_before, neighbors_before) =
            decode_index_elements_and_neighbors(index_oid, code_len(4, 4));
        let (deleted_element_tid, _deleted_element) =
            find_element_for_heap_tid(&elements_before, deleted_heap_tid);
        assert!(
            count_neighbor_refs(&neighbors_before, deleted_element_tid) > 0,
            "fixture should start with at least one persisted neighbor ref to the deleted element",
        );

        Spi::run("DELETE FROM ec_hnsw_vacuum_pass1_repeat WHERE id = 2")
            .expect("delete should succeed");

        let first_stats =
            unsafe { am::debug_vacuum_remove_heap_tids(index_oid, &[deleted_heap_tid]) };
        let second_stats =
            unsafe { am::debug_vacuum_remove_heap_tids(index_oid, &[deleted_heap_tid]) };
        let (_metadata, elements, neighbors) =
            decode_index_elements_and_neighbors(index_oid, code_len(4, 4));

        assert_eq!(first_stats.tuples_removed, 1.0);
        assert_eq!(second_stats.tuples_removed, 0.0);
        assert_eq!(second_stats.num_index_tuples, first_stats.num_index_tuples);
        assert_eq!(
            elements
                .iter()
                .filter(|(_, element)| element.heaptids.is_empty() && element.deleted)
                .count(),
            1,
            "the second pass should observe the already-finalized fully dead element without rewriting it again",
        );
        assert_eq!(
            count_neighbor_refs(&neighbors, deleted_element_tid),
            0,
            "replaying the same vacuum delete-set should keep the deleted element tid fully unlinked from persisted neighbor tuples",
        );
    }

    #[pg_test]
    fn test_ech_vacuum_finalized_nodes_skip_duplicate_coalesce() {
        Spi::run(
            "CREATE TABLE ec_hnsw_vacuum_reinsert (id bigint primary key, embedding ecvector)",
        )
        .expect("table creation should succeed");
        Spi::run(
            "INSERT INTO ec_hnsw_vacuum_reinsert VALUES
             (1, encode_to_ecvector(ARRAY[0.5, 1.0, -0.5, 0.25], 4, 42))",
        )
        .expect("seed insert should succeed");
        Spi::run(
            "CREATE INDEX ec_hnsw_vacuum_reinsert_idx ON ec_hnsw_vacuum_reinsert USING ec_hnsw \
             (embedding ecvector_ip_ops)",
        )
        .expect("index creation should succeed");

        let deleted_heap_tid = heap_tid_for_row("ec_hnsw_vacuum_reinsert", 1);
        let index_oid =
            Spi::get_one::<pg_sys::Oid>("SELECT 'ec_hnsw_vacuum_reinsert_idx'::regclass::oid")
                .expect("SPI query should succeed")
                .expect("index oid should exist");

        Spi::run("DELETE FROM ec_hnsw_vacuum_reinsert WHERE id = 1")
            .expect("delete should succeed");
        unsafe { am::debug_vacuum_remove_heap_tids(index_oid, &[deleted_heap_tid]) };

        Spi::run(
            "INSERT INTO ec_hnsw_vacuum_reinsert VALUES
             (2, encode_to_ecvector(ARRAY[0.5, 1.0, -0.5, 0.25], 4, 42))",
        )
        .expect("replacement insert should succeed");

        let replacement_heap_tid = heap_tid_for_row("ec_hnsw_vacuum_reinsert", 2);
        let (_metadata, elements, _neighbors) =
            decode_index_elements_and_neighbors(index_oid, code_len(4, 4));
        let (_replacement_tid, replacement_element) =
            find_element_for_heap_tid(&elements, replacement_heap_tid);

        assert!(
            !replacement_element.deleted,
            "duplicate insert should append or coalesce into a live element, not a finalized tombstone",
        );
        assert_eq!(replacement_element.heaptids, vec![replacement_heap_tid]);
        assert_eq!(
            elements
                .iter()
                .filter(|(_, element)| element.deleted)
                .count(),
            1,
            "the finalized vacuum tombstone should remain on disk until page compaction lands",
        );

        let returned =
            unsafe { am::debug_gettuple_scan_heap_tids(index_oid, vec![0.5, 1.0, -0.5, 0.25]) };
        assert!(
            returned.contains(&(
                replacement_heap_tid.block_number,
                replacement_heap_tid.offset_number
            )),
            "the replacement row should stay reachable after reinserting the same encoded vector",
        );
    }

