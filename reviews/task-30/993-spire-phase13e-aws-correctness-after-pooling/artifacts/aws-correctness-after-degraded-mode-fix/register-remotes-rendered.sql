\set ON_ERROR_STOP on

SELECT ec_spire_register_remote_node_descriptor('ec_spire_aws_synth_10k_idx'::regclass::oid, 2, 1, 'ecaz-spire-aws-bec2ac76-remote-1-20260526235247960800000003', decode('a98e2f31a1a480a6', 'hex'), 'ec_spire_aws_synth_10k_remote_idx', 'active', 1, 1, '0.1.1', 'none') AS registered_node_2;

SELECT ec_spire_register_remote_node_descriptor('ec_spire_aws_synth_10k_idx'::regclass::oid, 3, 1, 'ecaz-spire-aws-bec2ac76-remote-2-20260526235247960700000001', decode('49c37d28de6f7855', 'hex'), 'ec_spire_aws_synth_10k_remote_idx', 'active', 1, 1, '0.1.1', 'none') AS registered_node_3;

SELECT ec_spire_register_remote_node_descriptor('ec_spire_aws_synth_10k_idx'::regclass::oid, 4, 1, 'ecaz-spire-aws-bec2ac76-remote-3-20260526235247961100000005', decode('31b04d4307aa1d50', 'hex'), 'ec_spire_aws_synth_10k_remote_idx', 'active', 1, 1, '0.1.1', 'none') AS registered_node_4;

