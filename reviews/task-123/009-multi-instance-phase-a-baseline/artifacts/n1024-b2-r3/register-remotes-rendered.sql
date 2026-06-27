\set ON_ERROR_STOP on

SELECT ec_spire_register_remote_node_descriptor('t123_p9_mi_100k_n1024_b2_coord_idx'::regclass::oid, 2, 1, 'spire/remote/aws-local/node2', decode('2e0c4a26bdc5827d', 'hex'), 't123_p9_mi_100k_n1024_b2_remote_idx', 'active', 1, 1, '0.1.1', 'none') AS registered_node_2;

SELECT ec_spire_register_remote_node_descriptor('t123_p9_mi_100k_n1024_b2_coord_idx'::regclass::oid, 3, 1, 'spire/remote/aws-local/node3', decode('19f3be29cccc415a', 'hex'), 't123_p9_mi_100k_n1024_b2_remote_idx', 'active', 1, 1, '0.1.1', 'none') AS registered_node_3;

SELECT ec_spire_register_remote_node_descriptor('t123_p9_mi_100k_n1024_b2_coord_idx'::regclass::oid, 4, 1, 'spire/remote/aws-local/node4', decode('fb36f82d0c144e37', 'hex'), 't123_p9_mi_100k_n1024_b2_remote_idx', 'active', 1, 1, '0.1.1', 'none') AS registered_node_4;

