use crate::*;
use sp_std::vec::Vec;
use sp_std::marker::PhantomData;
use ark_ff::{PrimeField, BigInteger};
use ark_crypto_primitives::{Error};
use ark_groth16::{Proof, VerifyingKey};
use arkworks_gadgets::{
	utils::{to_field_elements},
	common::mixer::{get_public_inputs},
};

pub struct ArkworksMixerVerifierGroth16<E: PairingEngine>(PhantomData<E>);

impl<E: PairingEngine> InstanceVerifier for ArkworksMixerVerifierGroth16<E> {
	fn verify(public_inp_bytes: &[u8], proof_bytes: &[u8], vk_bytes: &[u8]) -> Result<bool, Error> {
		let public_input_field_elts = to_field_elements::<E::Fr>(public_inp_bytes);
		let public_inputs = get_public_inputs::<E::Fr>(
			public_input_field_elts[0], // nullifier_hash
			public_input_field_elts[1], // root
			public_input_field_elts[2], // recipient
			public_input_field_elts[3], // relayer
		);
		let vk = VerifyingKey::<E>::deserialize(&vk_bytes.unwrap()[..])?;
		let proof = Proof::<E>::deserialize(&proof_bytes[..])?;
		let res = verify_groth16::<E>(&vk, &public_inputs, &proof);
		Ok(res)
	}
}

use ark_bls12_381::{Fr as Bls381};
pub type ArkworksBls381Verifier = ArkworksMixerVerifierGroth16<Bls381>;
