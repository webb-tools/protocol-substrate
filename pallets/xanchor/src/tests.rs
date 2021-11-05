use super::{
	mock::{parachain::*, *},
	test_utils::*,
	*,
};

use arkworks_gadgets::setup::common::Curve;
use codec::Encode;
use darkwebb_primitives::utils::encode_resource_id;
use frame_support::{assert_ok, traits::OnInitialize};
use xcm::latest::prelude::*;
use xcm_simulator::TestExt;

const SEED: u32 = 0;
const TREE_DEPTH: usize = 30;
const M: usize = 2;
const DEPOSIT_SIZE: u128 = 10_000;

fn setup_environment(curve: Curve) -> Vec<u8> {
	let params = match curve {
		Curve::Bn254 => get_hash_params::<ark_bn254::Fr>(curve),
		Curve::Bls381 => {
			todo!("Setup hash params for bls381")
		}
	};
	// 1. Setup The Hasher Pallet.
	assert_ok!(HasherPallet::force_set_parameters(Origin::root(), params.0));
	// 2. Initialize MerkleTree pallet.
	<MerkleTree as OnInitialize<u64>>::on_initialize(1);
	// 3. Setup the VerifierPallet
	//    but to do so, we need to have a VerifyingKey
	let mut verifier_key_bytes = Vec::new();
	let mut proving_key_bytes = Vec::new();

	get_keys(curve, &mut proving_key_bytes, &mut verifier_key_bytes);

	assert_ok!(VerifierPallet::force_set_parameters(Origin::root(), verifier_key_bytes));

	// finally return the provingkey bytes
	proving_key_bytes
}

// Helper function for forming buy execution message
fn buy_execution<C>(fees: impl Into<MultiAsset>) -> Instruction<C> {
	BuyExecution {
		fees: fees.into(),
		weight_limit: Unlimited,
	}
}

// sanity check that XCM is working
#[test]
fn dmp() {
	MockNet::reset();

	let remark =
		parachain::Call::System(frame_system::Call::<parachain::Runtime>::remark_with_event { remark: vec![1, 2, 3] });
	Relay::execute_with(|| {
		assert_ok!(RelayChainPalletXcm::send_xcm(
			Here,
			Parachain(PARAID_A),
			Xcm(vec![Transact {
				origin_type: OriginKind::SovereignAccount,
				require_weight_at_most: INITIAL_BALANCE as u64,
				call: remark.encode().into(),
			}]),
		));
	});

	ParaA::execute_with(|| {
		use parachain::{Event, System};
		assert!(System::events()
			.iter()
			.any(|r| matches!(r.event, Event::System(frame_system::Event::Remarked(_, _)))));
	});
}

#[test]
fn ump() {
	MockNet::reset();

	let remark = relay_chain::Call::System(frame_system::Call::<relay_chain::Runtime>::remark_with_event {
		remark: vec![1, 2, 3],
	});
	ParaA::execute_with(|| {
		assert_ok!(ParachainPalletXcm::send_xcm(
			Here,
			Parent,
			Xcm(vec![Transact {
				origin_type: OriginKind::SovereignAccount,
				require_weight_at_most: INITIAL_BALANCE as u64,
				call: remark.encode().into(),
			}]),
		));
	});

	Relay::execute_with(|| {
		use relay_chain::{Event, System};
		assert!(System::events()
			.iter()
			.any(|r| matches!(r.event, Event::System(frame_system::Event::Remarked(_, _)))));
	});
}

#[test]
fn xcmp() {
	MockNet::reset();

	let remark =
		parachain::Call::System(frame_system::Call::<parachain::Runtime>::remark_with_event { remark: vec![1, 2, 3] });
	ParaA::execute_with(|| {
		assert_ok!(ParachainPalletXcm::send_xcm(
			Here,
			(Parent, Parachain(PARAID_B)),
			Xcm(vec![Transact {
				origin_type: OriginKind::SovereignAccount,
				require_weight_at_most: INITIAL_BALANCE as u64,
				call: remark.encode().into(),
			}]),
		));
	});

	ParaB::execute_with(|| {
		use parachain::{Event, System};
		assert!(System::events()
			.iter()
			.any(|r| matches!(r.event, Event::System(frame_system::Event::Remarked(_, _)))));
	});
}

#[test]
fn link_two_anchors() {
	MockNet::reset();
	let mut para_a_tree_id = 0;
	let mut para_b_tree_id = 0;

	ParaA::execute_with(|| {
		setup_environment(Curve::Bn254);
		let max_edges = M as _;
		let depth = TREE_DEPTH as u8;
		let asset_id = 0;
		assert_ok!(Anchor::create(Origin::root(), DEPOSIT_SIZE, max_edges, depth, asset_id));
		para_a_tree_id = MerkleTree::next_tree_id() - 1;
	});

	ParaB::execute_with(|| {
		setup_environment(Curve::Bn254);
		let max_edges = M as _;
		let depth = TREE_DEPTH as u8;
		let asset_id = 0;
		assert_ok!(Anchor::create(Origin::root(), DEPOSIT_SIZE, max_edges, depth, asset_id));
		para_b_tree_id = MerkleTree::next_tree_id() - 1;
	});

	// The caller her is one of the Parachain B operators.
	// it will try to link the para_a_tree_id to para_b_tree_id
	// we tell ParaA to link the `para_a_tree_id` to `para_b_tree_id` on the ParaB.
	ParaA::execute_with(|| {
		// the resource id reads as following
		// we need to link para_a_tree_id to another anchor defined on ParaB
		let r_id = encode_resource_id(para_a_tree_id, PARAID_B);
		// then, on the call here, we tell it which tree we are going to link to.
		// (para_b_tree_id).
		assert_ok!(XAnchor::force_register_resource_id(
			Origin::root(),
			r_id,
			para_b_tree_id
		));
	});
	// Here, the same as above, but the only difference is that
	// the caller is one of the Parachain A operators.
	ParaB::execute_with(|| {
		// we need to link para_b_tree_id to another anchor defined on ParaA
		let r_id = encode_resource_id(para_b_tree_id, PARAID_A);
		// then, when we are sending the call we tell it which tree we are going to link
		// to. (para_a_tree_id).
		assert_ok!(XAnchor::force_register_resource_id(
			Origin::root(),
			r_id,
			para_a_tree_id
		));
	});

	// now we assume both of them are linked, let's check that.
	ParaA::execute_with(|| {
		let exists =
			crate::LinkedAnchors::<parachain::Runtime, _>::iter().any(|(chain_id, tree_id, target_tree_id)| {
				chain_id == PARAID_B && tree_id == para_a_tree_id && target_tree_id == para_b_tree_id
			});
		assert!(exists, "ParaA does not have link to ParaB");
	});

	ParaB::execute_with(|| {
		let exists =
			crate::LinkedAnchors::<parachain::Runtime, _>::iter().any(|(chain_id, tree_id, target_tree_id)| {
				chain_id == PARAID_A && tree_id == para_b_tree_id && target_tree_id == para_a_tree_id
			});
		assert!(exists, "ParaB does not have link to ParaB");
	});
}

#[test]
fn bridge_anchors_using_xcm() {
	MockNet::reset();
	let mut para_a_tree_id = 0;
	let mut para_b_tree_id = 0;

	ParaA::execute_with(|| {
		setup_environment(Curve::Bn254);
		let max_edges = M as _;
		let depth = TREE_DEPTH as u8;
		let asset_id = 0;
		assert_ok!(Anchor::create(Origin::root(), DEPOSIT_SIZE, max_edges, depth, asset_id));
		para_a_tree_id = MerkleTree::next_tree_id() - 1;
	});

	ParaB::execute_with(|| {
		setup_environment(Curve::Bn254);
		let max_edges = M as _;
		let depth = TREE_DEPTH as u8;
		let asset_id = 0;
		assert_ok!(Anchor::create(Origin::root(), DEPOSIT_SIZE, max_edges, depth, asset_id));
		para_b_tree_id = MerkleTree::next_tree_id() - 1;
	});

	ParaA::execute_with(|| {
		let r_id = encode_resource_id(para_a_tree_id, PARAID_B);
		assert_ok!(XAnchor::force_register_resource_id(
			Origin::root(),
			r_id,
			para_b_tree_id
		));
	});

	ParaB::execute_with(|| {
		let r_id = encode_resource_id(para_b_tree_id, PARAID_A);
		assert_ok!(XAnchor::force_register_resource_id(
			Origin::root(),
			r_id,
			para_a_tree_id
		));
	});

	// now we do a deposit on one chain (ParaA) for example
	// and check the edges on the other chain (ParaB).
	let mut para_a_root = Element::from_bytes(&[0u8; 32]);
	ParaA::execute_with(|| {
		let account_id = ALICE;
		let leaf = Element::from_bytes(&[1u8; 32]);
		// check the balance before the deposit.
		let balance_before = Balances::free_balance(account_id.clone());
		// and we do the deposit
		assert_ok!(Anchor::deposit(
			Origin::signed(account_id.clone()),
			para_a_tree_id,
			leaf
		));
		// now we check the balance after the deposit.
		let balance_after = Balances::free_balance(account_id);
		// the balance should be less now with `deposit_size`
		assert_eq!(balance_after, balance_before - DEPOSIT_SIZE);
		// now we need also to check if the state got updated.
		let tree = MerkleTree::trees(para_a_tree_id);
		assert_eq!(tree.leaf_count, 1);
		para_a_root = tree.root;
	});

	// ok now we go to ParaB and check the edges.
	// we should expect that the edge for ParaA is there, and the merkle root equal
	// to the one we got from ParaA.
	ParaB::execute_with(|| {
		let edge = Anchor::edge_list(para_b_tree_id, PARAID_A);
		assert_eq!(edge.root, para_a_root);
	});
}