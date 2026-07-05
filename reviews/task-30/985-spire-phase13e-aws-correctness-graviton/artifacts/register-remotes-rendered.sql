\set ON_ERROR_STOP on

SELECT ec_spire_register_remote_node_descriptor('ec_spire_aws_synth_10k_idx'::regclass::oid, 2, 1, 'ecaz-spire-aws-90599215-remote-1-20260525221947626500000001', decode('f9bdf3053894d43d', 'hex'), 'ec_spire_aws_synth_10k_remote_idx', 'active', 1, 1, '0.1.1', 'none') AS registered_node_2;

SELECT ec_spire_register_remote_node_descriptor('ec_spire_aws_synth_10k_idx'::regclass::oid, 3, 1, 'ecaz-spire-aws-90599215-remote-2-20260525221947626700000005', decode('299ba12ef04afe12', 'hex'), 'ec_spire_aws_synth_10k_remote_idx', 'active', 1, 1, '0.1.1', 'none') AS registered_node_3;

SELECT ec_spire_register_remote_node_descriptor('ec_spire_aws_synth_10k_idx'::regclass::oid, 4, 1, 'ecaz-spire-aws-90599215-remote-3-20260525221947626600000003', decode('064f0898178b2a6f', 'hex'), 'ec_spire_aws_synth_10k_remote_idx', 'active', 1, 1, '0.1.1', 'none') AS registered_node_4;

