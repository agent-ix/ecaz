\set ON_ERROR_STOP on

SELECT ec_spire_register_remote_node_descriptor('t142_release_100k_n1024_b0_coord_idx'::regclass::oid, 2, 1, 'spire/remote/aws-local/node2', decode('8846caaecc6d60ae', 'hex'), 't142_release_100k_n1024_b0_remote_idx', 'active', 1, 1, '0.1.1', 'none') AS registered_node_2;

SELECT ec_spire_register_remote_node_descriptor('t142_release_100k_n1024_b0_coord_idx'::regclass::oid, 3, 1, 'spire/remote/aws-local/node3', decode('898ad59caebc54b3', 'hex'), 't142_release_100k_n1024_b0_remote_idx', 'active', 1, 1, '0.1.1', 'none') AS registered_node_3;

SELECT ec_spire_register_remote_node_descriptor('t142_release_100k_n1024_b0_coord_idx'::regclass::oid, 4, 1, 'spire/remote/aws-local/node4', decode('3a53960f7386dcc4', 'hex'), 't142_release_100k_n1024_b0_remote_idx', 'active', 1, 1, '0.1.1', 'none') AS registered_node_4;
