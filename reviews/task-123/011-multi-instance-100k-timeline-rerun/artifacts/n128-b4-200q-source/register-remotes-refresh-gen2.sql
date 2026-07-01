\set ON_ERROR_STOP on

SELECT ec_spire_register_remote_node_descriptor('t123_p11_mi_100k_n128_b4_200q_source_coord_idx'::regclass::oid, 2, 2, 'spire/remote/aws-local/node2', decode('583b7ca39d65586b', 'hex'), 't123_p11_mi_100k_n128_b4_200q_source_remote_idx', 'active', 1, 1, '0.1.1', 'none') AS registered_node_2;

SELECT ec_spire_register_remote_node_descriptor('t123_p11_mi_100k_n128_b4_200q_source_coord_idx'::regclass::oid, 3, 2, 'spire/remote/aws-local/node3', decode('df081d6eac66fcfa', 'hex'), 't123_p11_mi_100k_n128_b4_200q_source_remote_idx', 'active', 1, 1, '0.1.1', 'none') AS registered_node_3;

SELECT ec_spire_register_remote_node_descriptor('t123_p11_mi_100k_n128_b4_200q_source_coord_idx'::regclass::oid, 4, 2, 'spire/remote/aws-local/node4', decode('3ae2af1703d9b548', 'hex'), 't123_p11_mi_100k_n128_b4_200q_source_remote_idx', 'active', 1, 1, '0.1.1', 'none') AS registered_node_4;
