use std::convert::TryInto;

use crate::mock::*;

use frame_support::{assert_err, assert_ok, error::BadOrigin};

use asset_registry::AssetType;

#[test]
fn should_wrap_token() {
	new_test_ext().execute_with(|| {
		let existential_balance: u32 = 1000;
		let first_token_id = AssetRegistry::register_asset(
			b"shib".to_vec().try_into().unwrap(),
			AssetType::Token,
			existential_balance.into(),
		)
		.unwrap();
		let second_token_id = AssetRegistry::register_asset(
			b"doge".to_vec().try_into().unwrap(),
			AssetType::Token,
			existential_balance.into(),
		)
		.unwrap();

		let pool_share_id = AssetRegistry::register_asset(
			b"meme".to_vec().try_into().unwrap(),
			AssetType::PoolShare(vec![second_token_id, first_token_id]),
			existential_balance.into(),
		)
		.unwrap();

		let recipient: u64 = 1;

		let balance: i128 = 100000;

		assert_ok!(Currencies::update_balance(
			Origin::root(),
			recipient,
			first_token_id,
			balance.into()
		));

		assert_ok!(TokenWrapper::set_wrapping_fee(Origin::root(), 5));

		assert_ok!(TokenWrapper::wrap(
			Origin::signed(recipient),
			first_token_id,
			pool_share_id,
			50000 as u128,
			recipient
		));

		assert_eq!(Tokens::total_issuance(pool_share_id), 50000);

		assert_eq!(TokenWrapper::get_balance(first_token_id, &recipient), 50000);

		assert_eq!(TokenWrapper::get_balance(pool_share_id, &recipient), 50000);
	})
}

#[test]
fn should_unwrap_token() {
	new_test_ext().execute_with(|| {
		let existential_balance: u32 = 1000;
		let first_token_id = AssetRegistry::register_asset(
			b"shib".to_vec().try_into().unwrap(),
			AssetType::Token,
			existential_balance.into(),
		)
		.unwrap();
		let second_token_id = AssetRegistry::register_asset(
			b"doge".to_vec().try_into().unwrap(),
			AssetType::Token,
			existential_balance.into(),
		)
		.unwrap();

		let pool_share_id = AssetRegistry::register_asset(
			b"meme".to_vec().try_into().unwrap(),
			AssetType::PoolShare(vec![second_token_id, first_token_id]),
			existential_balance.into(),
		)
		.unwrap();

		let recipient: u64 = 1;

		let balance: i128 = 100000;

		assert_ok!(Currencies::update_balance(
			Origin::root(),
			recipient,
			first_token_id,
			balance.into()
		));

		assert_ok!(TokenWrapper::set_wrapping_fee(Origin::root(), 5));

		assert_ok!(TokenWrapper::wrap(
			Origin::signed(recipient),
			first_token_id,
			pool_share_id,
			50000 as u128,
			recipient
		));

		assert_eq!(Tokens::total_issuance(pool_share_id), 50000);

		assert_eq!(TokenWrapper::get_balance(pool_share_id, &recipient), 50000);

		assert_eq!(TokenWrapper::get_balance(first_token_id, &recipient), 50000);

		assert_ok!(TokenWrapper::unwrap(
			Origin::signed(recipient),
			pool_share_id,
			first_token_id,
			50000 as u128,
			recipient
		));

		assert_eq!(Tokens::total_issuance(pool_share_id), Default::default());

		assert_eq!(TokenWrapper::get_balance(first_token_id, &recipient), 100000);
	})
}

#[test]
fn wrapping_should_fail_if_asset_is_not_in_pool() {
	new_test_ext().execute_with(|| {
		let existential_balance: u32 = 1000;
		let first_token_id = AssetRegistry::register_asset(
			b"shib".to_vec().try_into().unwrap(),
			AssetType::Token,
			existential_balance.into(),
		)
		.unwrap();
		let second_token_id = AssetRegistry::register_asset(
			b"doge".to_vec().try_into().unwrap(),
			AssetType::Token,
			existential_balance.into(),
		)
		.unwrap();

		let pool_share_id = AssetRegistry::register_asset(
			b"meme".to_vec().try_into().unwrap(),
			AssetType::PoolShare(vec![second_token_id]),
			existential_balance.into(),
		)
		.unwrap();

		let recipient: u64 = 1;

		let balance: i128 = 100000;

		assert_ok!(Currencies::update_balance(
			Origin::root(),
			recipient,
			first_token_id,
			balance.into()
		));

		assert_ok!(TokenWrapper::set_wrapping_fee(Origin::root(), 5));

		assert_err!(
			TokenWrapper::wrap(
				Origin::signed(recipient),
				first_token_id,
				pool_share_id,
				50000 as u128,
				recipient
			),
			crate::Error::<Test>::NotFoundInPool
		);
	})
}

#[test]
fn only_root_should_update_wrapping_fee() {
	new_test_ext().execute_with(|| {
		assert_err!(TokenWrapper::set_wrapping_fee(Origin::signed(1), 10), BadOrigin);

		assert_ok!(TokenWrapper::set_wrapping_fee(Origin::root(), 10));
	})
}

#[test]
fn should_not_unwrap_if_no_liquidity_exists_for_selected_assets() {
	new_test_ext().execute_with(|| {
		let existential_balance: u32 = 1000;
		let first_token_id = AssetRegistry::register_asset(
			b"shib".to_vec().try_into().unwrap(),
			AssetType::Token,
			existential_balance.into(),
		)
		.unwrap();
		let second_token_id = AssetRegistry::register_asset(
			b"doge".to_vec().try_into().unwrap(),
			AssetType::Token,
			existential_balance.into(),
		)
		.unwrap();

		let pool_share_id = AssetRegistry::register_asset(
			b"meme".to_vec().try_into().unwrap(),
			AssetType::PoolShare(vec![second_token_id, first_token_id]),
			existential_balance.into(),
		)
		.unwrap();

		let recipient: u64 = 1;

		let balance: i128 = 100000;

		assert_ok!(Currencies::update_balance(
			Origin::root(),
			recipient,
			first_token_id,
			balance.into()
		));

		assert_ok!(TokenWrapper::set_wrapping_fee(Origin::root(), 5));

		assert_ok!(TokenWrapper::wrap(
			Origin::signed(recipient),
			first_token_id,
			pool_share_id,
			50000 as u128,
			recipient
		));

		assert_eq!(Tokens::total_issuance(pool_share_id), 50000);

		assert_eq!(TokenWrapper::get_balance(pool_share_id, &recipient), 50000);

		assert_eq!(TokenWrapper::get_balance(first_token_id, &recipient), 50000);

		assert_err!(
			TokenWrapper::unwrap(
				Origin::signed(recipient),
				pool_share_id,
				second_token_id,
				50000 as u128,
				recipient
			),
			orml_tokens::Error::<Test>::BalanceTooLow
		);
	})
}

#[test]
fn should_unwrap_when_liquidity_exists_for_selected_asset() {
	new_test_ext().execute_with(|| {
		let existential_balance: u32 = 1000;
		let first_token_id = AssetRegistry::register_asset(
			b"shib".to_vec().try_into().unwrap(),
			AssetType::Token,
			existential_balance.into(),
		)
		.unwrap();
		let second_token_id = AssetRegistry::register_asset(
			b"doge".to_vec().try_into().unwrap(),
			AssetType::Token,
			existential_balance.into(),
		)
		.unwrap();

		let pool_share_id = AssetRegistry::register_asset(
			b"meme".to_vec().try_into().unwrap(),
			AssetType::PoolShare(vec![second_token_id, first_token_id]),
			existential_balance.into(),
		)
		.unwrap();

		let recipient: u64 = 1;

		let balance: i128 = 100000;

		assert_ok!(Currencies::update_balance(
			Origin::root(),
			recipient,
			first_token_id,
			balance.into()
		));

		assert_ok!(Currencies::update_balance(
			Origin::root(),
			TokenWrapper::account_id(),
			second_token_id,
			balance.into()
		));

		assert_ok!(TokenWrapper::set_wrapping_fee(Origin::root(), 5));

		assert_ok!(TokenWrapper::wrap(
			Origin::signed(recipient),
			first_token_id,
			pool_share_id,
			50000 as u128,
			recipient
		));

		assert_eq!(Tokens::total_issuance(pool_share_id), 50000);

		assert_eq!(TokenWrapper::get_balance(first_token_id, &recipient), 50000);

		assert_eq!(
			TokenWrapper::get_balance(second_token_id, &recipient),
			Default::default()
		);

		assert_eq!(TokenWrapper::get_balance(pool_share_id, &recipient), 50000);

		assert_ok!(TokenWrapper::unwrap(
			Origin::signed(recipient),
			pool_share_id,
			second_token_id,
			50000 as u128,
			recipient
		));

		assert_eq!(Tokens::total_issuance(pool_share_id), Default::default());

		assert_eq!(TokenWrapper::get_balance(second_token_id, &recipient), 50000);
	})
}

#[test]
fn should_not_wrap_invalid_amount() {
	new_test_ext().execute_with(|| {
		let existential_balance: u32 = 1000;
		let first_token_id = AssetRegistry::register_asset(
			b"shib".to_vec().try_into().unwrap(),
			AssetType::Token,
			existential_balance.into(),
		)
		.unwrap();
		let second_token_id = AssetRegistry::register_asset(
			b"doge".to_vec().try_into().unwrap(),
			AssetType::Token,
			existential_balance.into(),
		)
		.unwrap();

		let pool_share_id = AssetRegistry::register_asset(
			b"meme".to_vec().try_into().unwrap(),
			AssetType::PoolShare(vec![second_token_id, first_token_id]),
			existential_balance.into(),
		)
		.unwrap();

		let recipient: u64 = 1;

		let balance: i128 = 100000;

		assert_ok!(Currencies::update_balance(
			Origin::root(),
			recipient,
			first_token_id,
			balance.into()
		));

		assert_ok!(TokenWrapper::set_wrapping_fee(Origin::root(), 5));

		assert_err!(
			TokenWrapper::wrap(
				Origin::signed(recipient),
				first_token_id,
				pool_share_id,
				0 as u128,
				recipient
			),
			crate::Error::<Test>::InvalidAmount
		);
	})
}