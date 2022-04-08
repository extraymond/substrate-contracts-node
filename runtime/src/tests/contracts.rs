use std::str::FromStr;

use super::{ExtBuilder, ALICE};
use crate::{AccountId, Balance, Contracts, Event, Origin, Runtime, System};
use codec::Encode;
use frame_support::assert_ok;
use pallet_contracts::{Error as ContractError, Event as ContractEvent};
use sp_core::Bytes;
use sp_runtime::traits::Hash;

const GAS_LIMIT: u64 = 200_000_000_000;

/// find the latest intantiated contract addr from system events
fn find_new_contract_addr() -> AccountId {
	let evts = System::events();
	let flip_addr = evts
		.iter()
		.rev()
		.find_map(|evt| {
			if let Event::Contracts(ContractEvent::Instantiated { deployer: _, contract }) =
				&evt.event
			{
				Some(contract)
			} else {
				None
			}
		})
		.unwrap();

	flip_addr.clone()
}

/// convert raw hex-string output from jq to hex as bytes
fn raw_selector_to_vec(input: &str) -> Vec<u8> {
	serde_json::from_str::<String>(input)
		.map(|val| Bytes::from_str(&val).unwrap().to_vec())
		.unwrap()
}

type Salter = fn(Vec<u8>) -> Vec<u8>;

fn empty_salter() -> impl Iterator<Item = Salter> {
	(0..).map(|_| -> Salter { |_: Vec<u8>| vec![] })
}

fn nonce_salter() -> impl Iterator<Item = Salter> {
	(0..).map(|_| -> Salter { |_: Vec<u8>| System::account_nonce(ALICE).encode() })
}

/// mimic the behaviour of polkadot-js app, where salt is random u8_array as hex_string
fn rand_salter() -> impl Iterator<Item = Salter> {
	(0..).map(|_| -> Salter {
		|_: Vec<u8>| {
			let random_u8 = rand::random::<[u8; 32]>();
			Bytes::from(random_u8.to_vec()).encode()
		}
	})
}

type Hasher = <Runtime as frame_system::Config>::Hashing;

fn hashed_input_salter() -> impl Iterator<Item = Salter> {
	(0..).map(|_| -> Salter { |input: Vec<u8>| Hasher::hash(&input).encode() })
}

fn call_sol_from_sol(mut salters: impl Iterator<Item = fn(Vec<u8>) -> Vec<u8>>) {
	let flip_contract = std::fs::read_to_string("./contracts/Flip.contract").unwrap();
	let inc_contract = std::fs::read_to_string("./contracts/Inc.contract").unwrap();

	let mut find_wasm = jq_rs::compile(".source.wasm").unwrap();
	let blob_rs = find_wasm.run(&flip_contract).unwrap();

	let flip_blob: String = serde_json::from_str(&blob_rs).unwrap();

	let mut find_constructor =
		jq_rs::compile(r#".spec.constructors[] | select(.name | contains("new")) | .selector"#)
			.unwrap();

	let flip_constructors_raw = find_constructor.run(&flip_contract).unwrap();
	let flip_new_selector = raw_selector_to_vec(&flip_constructors_raw);

	assert_ok!(Contracts::instantiate_with_code(
		Origin::signed(ALICE),
		0,
		GAS_LIMIT,
		None,
		Bytes::from_str(&flip_blob).unwrap().to_vec(),
		flip_new_selector.clone(),
		salters.next().unwrap()(flip_new_selector),
	));

	let flip_addr = find_new_contract_addr();

	let mut find_flip =
		jq_rs::compile(r#".spec.messages[] | select(.name | contains("flip")) | .selector"#)
			.unwrap();

	let flip_selector_raw = find_flip.run(&flip_contract).unwrap();
	let flip_selector = raw_selector_to_vec(&flip_selector_raw);

	assert_ok!(Contracts::call(
		Origin::signed(ALICE),
		flip_addr.clone().into(),
		0,
		GAS_LIMIT,
		None,
		flip_selector,
	));

	let blob_rs = find_wasm.run(&inc_contract).unwrap();
	let inc_blob: String = serde_json::from_str(&blob_rs).unwrap();

	let inc_constructors_raw = find_constructor.run(&inc_contract).unwrap();
	let mut inc_new_selector = raw_selector_to_vec(&inc_constructors_raw);
	inc_new_selector.append(&mut flip_addr.encode());

	assert_ok!(Contracts::instantiate_with_code(
		Origin::signed(ALICE),
		0,
		GAS_LIMIT,
		None,
		Bytes::from_str(&inc_blob).unwrap().to_vec(),
		inc_new_selector.clone(),
		salters.next().unwrap()(inc_new_selector),
	));

	let inc_addr = find_new_contract_addr();

	let mut find_flip =
		jq_rs::compile(r#".spec.messages[] | select(.name | contains("flip")) | .selector"#)
			.unwrap();

	let flip_selector_raw = find_flip.run(&flip_contract).unwrap();
	let flip_selector = raw_selector_to_vec(&flip_selector_raw);

	assert_ok!(Contracts::call(
		Origin::signed(ALICE),
		flip_addr.clone().into(),
		0,
		GAS_LIMIT,
		None,
		flip_selector,
	));

	let mut find_superflip =
		jq_rs::compile(r#".spec.messages[] | select(.name | contains("superFlip")) | .selector"#)
			.unwrap();

	let superflip_selector_raw = find_superflip.run(&inc_contract).unwrap();
	let superflip_selector = raw_selector_to_vec(&superflip_selector_raw);

	assert_ok!(Contracts::call(
		Origin::signed(ALICE),
		inc_addr.clone().into(),
		0,
		GAS_LIMIT,
		None,
		superflip_selector,
	));
}

fn call_ink_from_ink(mut salter: impl Iterator<Item = Salter>) {
	let flip_contract =
		std::fs::read_to_string("./contracts/flipper/target/ink/flipper.contract").unwrap();

	let inc_contract = std::fs::read_to_string("./contracts/inc/target/ink/inc.contract").unwrap();

	let mut find_blob = jq_rs::compile(".source.wasm").unwrap();
	let mut find_constructor = jq_rs::compile("[.V3.spec.constructors[] | .selector]").unwrap();

	let flip_blob = find_blob
		.run(&flip_contract)
		.ok()
		.map(|val| serde_json::from_str::<String>(&val).ok())
		.flatten()
		.unwrap();

	let flip_constructor = find_constructor
		.run(&flip_contract)
		.ok()
		.and_then(|val| serde_json::from_str::<Vec<String>>(&val).ok())
		.map(|vals| vals[0].clone())
		.and_then(|val| Bytes::from_str(&val).ok().map(|v| v.to_vec()))
		.unwrap();

	assert_ok!(Contracts::instantiate_with_code(
		Origin::signed(ALICE),
		0,
		GAS_LIMIT,
		None,
		Bytes::from_str(&flip_blob).map(|v| v.to_vec()).unwrap(),
		flip_constructor.clone(),
		salter.next().unwrap()(flip_constructor),
	));

	let flip_addr = find_new_contract_addr();

	let mut find_flip =
		jq_rs::compile(r#".V3.spec.messages[] | select(.label | contains("flip")) | .selector"#)
			.unwrap();

	let flip_selector_raw = find_flip.run(&flip_contract).unwrap();

	let flip_selector = raw_selector_to_vec(&flip_selector_raw);

	assert_ok!(Contracts::call(
		Origin::signed(ALICE),
		flip_addr.clone().into(),
		0,
		GAS_LIMIT,
		None,
		flip_selector
	));

	let inc_blob = find_blob
		.run(&inc_contract)
		.ok()
		.map(|val| serde_json::from_str::<String>(&val).ok())
		.flatten()
		.unwrap();

	let mut inc_constructor = find_constructor
		.run(&inc_contract)
		.ok()
		.and_then(|val| serde_json::from_str::<Vec<String>>(&val).ok())
		.and_then(|val| Bytes::from_str(&val[0]).ok())
		.map(|val| val.to_vec())
		.unwrap();

	inc_constructor.append(&mut flip_addr.encode());

	assert_ok!(Contracts::instantiate_with_code(
		Origin::signed(ALICE),
		0,
		GAS_LIMIT,
		None,
		Bytes::from_str(&inc_blob).map(|v| v.to_vec()).unwrap(),
		inc_constructor.clone(),
		salter.next().unwrap()(inc_constructor)
	));

	let inc_addr = find_new_contract_addr();

	let mut find_superflip = jq_rs::compile(
		r#".V3.spec.messages[] | select(.label | contains("super_flip")) | .selector"#,
	)
	.unwrap();

	let superflip_selector_raw = find_superflip.run(&inc_contract).unwrap();

	let superflip_selector = raw_selector_to_vec(&superflip_selector_raw);

	assert_ok!(Contracts::call(
		Origin::signed(ALICE),
		inc_addr.clone().into(),
		0,
		GAS_LIMIT,
		None,
		superflip_selector
	));
}

fn call_sol_from_ink(mut salter: impl Iterator<Item = Salter>) {
	let flip_contract = std::fs::read_to_string("./contracts/Flip.contract").unwrap();

	let inc_contract = std::fs::read_to_string("./contracts/inc/target/ink/inc.contract").unwrap();

	let mut find_blob = jq_rs::compile(".source.wasm").unwrap();
	let mut find_ink_constructor = jq_rs::compile("[.V3.spec.constructors[] | .selector]").unwrap();

	let mut find_sol_constructor = jq_rs::compile("[.spec.constructors[] | .selector]").unwrap();

	let flip_blob = find_blob
		.run(&flip_contract)
		.ok()
		.map(|val| serde_json::from_str::<String>(&val).ok())
		.flatten()
		.unwrap();

	let flip_constructor = find_sol_constructor
		.run(&flip_contract)
		.ok()
		.and_then(|val| serde_json::from_str::<Vec<String>>(&val).ok())
		.map(|vals| vals[0].clone())
		.and_then(|val| Bytes::from_str(&val).ok().map(|v| v.to_vec()))
		.unwrap();

	assert_ok!(Contracts::instantiate_with_code(
		Origin::signed(ALICE),
		0,
		GAS_LIMIT,
		None,
		Bytes::from_str(&flip_blob).map(|v| v.to_vec()).unwrap(),
		flip_constructor.clone(),
		salter.next().unwrap()(flip_constructor),
	));

	let flip_addr = find_new_contract_addr();

	let mut find_flip =
		jq_rs::compile(r#".spec.messages[] | select(.name | contains("flip")) | .selector"#)
			.unwrap();

	let flip_selector_raw = find_flip.run(&flip_contract).unwrap();

	let flip_selector = raw_selector_to_vec(&flip_selector_raw);

	assert_ok!(Contracts::call(
		Origin::signed(ALICE),
		flip_addr.clone().into(),
		0,
		GAS_LIMIT,
		None,
		flip_selector
	));

	let inc_blob = find_blob
		.run(&inc_contract)
		.ok()
		.map(|val| serde_json::from_str::<String>(&val).ok())
		.flatten()
		.unwrap();

	let mut inc_constructor = find_ink_constructor
		.run(&inc_contract)
		.ok()
		.and_then(|val| serde_json::from_str::<Vec<String>>(&val).ok())
		.and_then(|val| Bytes::from_str(&val[0]).ok())
		.map(|val| val.to_vec())
		.unwrap();

	inc_constructor.append(&mut flip_addr.encode());

	assert_ok!(Contracts::instantiate_with_code(
		Origin::signed(ALICE),
		0,
		GAS_LIMIT,
		None,
		Bytes::from_str(&inc_blob).map(|v| v.to_vec()).unwrap(),
		inc_constructor.clone(),
		salter.next().unwrap()(inc_constructor)
	));

	let inc_addr = find_new_contract_addr();

	let mut find_superflip = jq_rs::compile(
		r#".V3.spec.messages[] | select(.label | contains("super_flip")) | .selector"#,
	)
	.unwrap();

	let superflip_selector_raw = find_superflip.run(&inc_contract).unwrap();

	let superflip_selector = raw_selector_to_vec(&superflip_selector_raw);

	assert_ok!(Contracts::call(
		Origin::signed(ALICE),
		inc_addr.clone().into(),
		0,
		GAS_LIMIT,
		None,
		superflip_selector
	));
}

fn call_ink_from_sol(mut salter: impl Iterator<Item = Salter>) {
	let flip_contract =
		std::fs::read_to_string("./contracts/flipper/target/ink/flipper.contract").unwrap();

	let inc_contract = std::fs::read_to_string("./contracts/Inc.contract").unwrap();

	let mut find_blob = jq_rs::compile(".source.wasm").unwrap();
	let mut find_ink_constructor = jq_rs::compile("[.V3.spec.constructors[] | .selector]").unwrap();

	let mut find_sol_constructor = jq_rs::compile("[.spec.constructors[] | .selector]").unwrap();

	let flip_blob = find_blob
		.run(&flip_contract)
		.ok()
		.map(|val| serde_json::from_str::<String>(&val).ok())
		.flatten()
		.unwrap();

	let flip_constructor = find_ink_constructor
		.run(&flip_contract)
		.ok()
		.and_then(|val| serde_json::from_str::<Vec<String>>(&val).ok())
		.map(|vals| vals[0].clone())
		.and_then(|val| Bytes::from_str(&val).ok().map(|v| v.to_vec()))
		.unwrap();

	assert_ok!(Contracts::instantiate_with_code(
		Origin::signed(ALICE),
		0,
		GAS_LIMIT,
		None,
		Bytes::from_str(&flip_blob).map(|v| v.to_vec()).unwrap(),
		flip_constructor.clone(),
		salter.next().unwrap()(flip_constructor),
	));

	let flip_addr = find_new_contract_addr();

	let mut find_flip =
		jq_rs::compile(r#".V3.spec.messages[] | select(.label | contains("flip")) | .selector"#)
			.unwrap();

	let flip_selector_raw = find_flip.run(&flip_contract).unwrap();

	let flip_selector = raw_selector_to_vec(&flip_selector_raw);

	assert_ok!(Contracts::call(
		Origin::signed(ALICE),
		flip_addr.clone().into(),
		0,
		GAS_LIMIT,
		None,
		flip_selector
	));

	let inc_blob = find_blob
		.run(&inc_contract)
		.ok()
		.map(|val| serde_json::from_str::<String>(&val).ok())
		.flatten()
		.unwrap();

	let mut inc_constructor = find_sol_constructor
		.run(&inc_contract)
		.ok()
		.and_then(|val| serde_json::from_str::<Vec<String>>(&val).ok())
		.and_then(|val| Bytes::from_str(&val[0]).ok())
		.map(|val| val.to_vec())
		.unwrap();

	inc_constructor.append(&mut flip_addr.encode());

	assert_ok!(Contracts::instantiate_with_code(
		Origin::signed(ALICE),
		0,
		GAS_LIMIT,
		None,
		Bytes::from_str(&inc_blob).map(|v| v.to_vec()).unwrap(),
		inc_constructor.clone(),
		salter.next().unwrap()(inc_constructor)
	));

	let inc_addr = find_new_contract_addr();

	let mut find_superflip =
		jq_rs::compile(r#".spec.messages[] | select(.name | contains("superFlip")) | .selector"#)
			.unwrap();

	let superflip_selector_raw = find_superflip.run(&inc_contract).unwrap();

	let superflip_selector = raw_selector_to_vec(&superflip_selector_raw);

	assert_ok!(Contracts::call(
		Origin::signed(ALICE),
		inc_addr.clone().into(),
		0,
		GAS_LIMIT,
		None,
		superflip_selector
	));
}

#[test]
fn test_sol_to_sol() {
	ExtBuilder::default()
		.balances(vec![(ALICE, 100_000_000_000_000_000)])
		.sudo(ALICE)
		.build()
		.execute_with(|| {
			// test against empty salts
			call_sol_from_sol(empty_salter());

			// test against salt as account nonce
			call_sol_from_sol(nonce_salter());

			// test against salt as random [u8; 32]
			call_sol_from_sol(rand_salter());

			// test against salt value prior to pr #7482
			call_sol_from_sol(hashed_input_salter());
		});
}

#[test]
fn test_ink_to_ink() {
	ExtBuilder::default()
		.balances(vec![(ALICE, 100_000_000_000_000_000)])
		.sudo(ALICE)
		.build()
		.execute_with(|| {
			// test against empty salts
			call_ink_from_ink(empty_salter());

			// test against salt as account nonce
			call_ink_from_ink(nonce_salter());

			// test against salt as random [u8; 32]
			call_ink_from_ink(rand_salter());

			// test against salt value prior to pr #7482
			call_ink_from_ink(hashed_input_salter());
		});
}

#[test]
fn test_ink_to_sol() {
	ExtBuilder::default()
		.balances(vec![(ALICE, 100_000_000_000_000_000)])
		.sudo(ALICE)
		.build()
		.execute_with(|| {
			// test against empty salts
			call_sol_from_ink(empty_salter());

			// test against salt as account nonce
			call_sol_from_ink(nonce_salter());

			// test against salt as random [u8; 32]
			call_sol_from_ink(rand_salter());

			// test against salt value prior to pr #7482
			call_sol_from_ink(hashed_input_salter());
		});
}

#[test]
fn test_sol_to_ink() {
	ExtBuilder::default()
		.balances(vec![(ALICE, 100_000_000_000_000_000)])
		.sudo(ALICE)
		.build()
		.execute_with(|| {
			// test against empty salts
			call_ink_from_sol(empty_salter());

			// test against salt as account nonce
			call_ink_from_sol(nonce_salter());

			// test against salt as random [u8; 32]
			call_ink_from_sol(rand_salter());

			// test against salt value prior to pr #7482
			call_ink_from_sol(hashed_input_salter());
		});
}
