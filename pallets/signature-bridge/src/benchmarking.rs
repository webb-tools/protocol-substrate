// This file is part of Webb.

// Copyright (C) 2022 Webb Technologies Inc.
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Signature pallet benchmarking.
use super::*;
use frame_benchmarking::{
	benchmarks_instance_pallet, impl_benchmark_test_suite, whitelisted_caller,
};
use frame_system::RawOrigin;
use hex_literal::hex;
use sp_core::{
	ecdsa::{self, Signature},
	keccak_256, Pair,
};
use webb_primitives::utils::{compute_chain_id_type, derive_resource_id};

fn assert_last_event<T: Config<I>, I: 'static>(generic_event: <T as Config<I>>::Event) {
	frame_system::Pallet::<T>::assert_last_event(generic_event.into());
}

/// Helper function to generate old/new maintainer signatures
/// Returns (old_maintainer, message, signature)
pub fn generate_maintainer_signatures<T: Config<I>, I: 'static>() -> (Vec<u8>, Vec<u8>, Vec<u8>) {
	let pair = ecdsa::Pair::from_string(
		"0x9d61b19deffd5a60ba844af492ec2cc44449c5697b326919703bac031cae7f60",
		None,
	)
	.unwrap();
	let new_maintainer = hex!("8db55b05db86c0b1786ca49f095d76344c9e6056b2f02701a7e7f3c20aabfd913ebbe148dd17c56551a52952371071a6c604b3f3abe8f2c8fa742158ea6dd7d4");
	let old_maintainer =
		libsecp256k1::PublicKey::parse_compressed(&pair.public().0).unwrap().serialize()[1..]
			.to_vec();
	Maintainer::<T, _>::put(old_maintainer.clone());
	let mut message = vec![];
	let nonce = 1u32.encode();
	message.extend_from_slice(&nonce);
	message.extend_from_slice(&new_maintainer);
	let msg = keccak_256(&message);
	let sig: Signature = pair.sign_prehashed(&msg).into();

	(old_maintainer, message, sig.encode())
}

/// Helper function to generate proposal data
fn make_proposal_data(encoded_r_id: Vec<u8>, nonce: [u8; 4], encoded_call: Vec<u8>) -> Vec<u8> {
	let mut prop_data = encoded_r_id;
	prop_data.extend_from_slice(&[0u8; 4]);
	prop_data.extend_from_slice(&nonce);
	prop_data.extend_from_slice(&encoded_call[..]);
	prop_data
}

benchmarks_instance_pallet! {

	where_clause {  where T::Proposal : From<frame_system::Call<T>> }

	set_maintainer {
		let caller: T::AccountId = whitelisted_caller();
		let (old_maintainer, message, signature) = generate_maintainer_signatures::<T, I>();
	}: _(RawOrigin::Signed(caller), message.clone(), signature)
	verify {
		assert_last_event::<T, I>(Event::MaintainerSet{old_maintainer: old_maintainer, new_maintainer: message}.into());
	}

	force_set_maintainer {
		let caller: T::AccountId = whitelisted_caller();
		let new_maintainer = hex!("8db55b05db86c0b1786ca49f095d76344c9e6056b2f02701a7e7f3c20aabfd913ebbe148dd17c56551a52952371071a6c604b3f3abe8f2c8fa742158ea6dd7d4");
	}: _(RawOrigin::Root, new_maintainer.into())
	verify {
		assert_last_event::<T, I>(Event::MaintainerSet{old_maintainer: Default::default(), new_maintainer: new_maintainer.into()}.into());
	}

	set_resource {
		let id: ResourceId = [1; 32];
		let method = "Pallet.do_something".as_bytes().to_vec();
	}: _(RawOrigin::Root, id, method.clone())
	verify {
	   assert_eq!(Resources::<T, I>::get(id).unwrap(), method);
	}

	remove_resource {
		let id: ResourceId = [1; 32];
		let method = "Pallet.do_something".as_bytes().to_vec();
		let _ = crate::Pallet::<T,I>::set_resource(RawOrigin::Root.into(), id, method);
	}: _(RawOrigin::Root, id)
	verify {
	   assert_eq!(Resources::<T, I>::get(id), None);
	}

	whitelist_chain {
	}: _(RawOrigin::Root, 0_u32.into())
	verify {
		assert_last_event::<T, I>(Event::ChainWhitelisted{chain_id : 0_u32.into()}.into());
	}

	execute_proposal {
		let caller: T::AccountId = whitelisted_caller();

		// set a new maintainer
		let new_maintainer = hex!("8db55b05db86c0b1786ca49f095d76344c9e6056b2f02701a7e7f3c20aabfd913ebbe148dd17c56551a52952371071a6c604b3f3abe8f2c8fa742158ea6dd7d4");
		let _ = crate::Pallet::<T,I>::force_set_maintainer(RawOrigin::Root.into(), new_maintainer.to_vec());

		// whitelist chain
		let chain_type = [2, 0];
		let src_id = compute_chain_id_type(1u32, chain_type);
		let _ = crate::Pallet::<T,I>::whitelist_chain(RawOrigin::Root.into(), src_id.into());

		// set resource
		let r_id : ResourceId = derive_resource_id(5u32, 1u32).into();
		let _ = crate::Pallet::<T,I>::set_resource(RawOrigin::Root.into(), r_id.into(), b"System.remark".to_vec());

		// prepare proposal
		let call : <T as pallet::Config<I>>::Proposal = frame_system::Call::<T>::remark { remark: vec![10] }.into();
		let call_encoded = call.encode();
		let nonce = [0u8, 0u8, 0u8, 1u8];
		let prop_data = make_proposal_data(r_id.encode(), nonce, call_encoded);
		let msg = keccak_256(&prop_data);
		let pair = ecdsa::Pair::from_string(
			"0x9d61b19deffd5a60ba844af492ec2cc44449c5697b326919703bac031cae7f60",
			None,
		)
		.unwrap();
		let sig: Signature = pair.sign_prehashed(&msg).into();
	}: _(RawOrigin::Signed(caller), src_id.into(), Box::new(call), prop_data, sig.0.to_vec())
	verify {}
}

impl_benchmark_test_suite!(Pallet, crate::mock::new_test_ext(), crate::mock::Test);
