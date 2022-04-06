use crate::{AccountId, Balance, Contracts, Event, Runtime, System};
use frame_support::traits::GenesisBuild;
use pallet_contracts::{Error as ContractError, Event as ContractEvent};

mod contracts;

pub const ALICE: AccountId = AccountId::new([1u8; 32]);
pub const BOB: AccountId = AccountId::new([2u8; 32]);
pub struct ExtBuilder {
	balances: Vec<(AccountId, Balance)>,
	sudo: Option<AccountId>,
}

impl Default for ExtBuilder {
	fn default() -> Self {
		Self { balances: vec![], sudo: None }
	}
}

impl ExtBuilder {
	pub fn balances(mut self, balances: Vec<(AccountId, Balance)>) -> Self {
		self.balances = balances;
		self
	}

	pub fn sudo(mut self, sudo: AccountId) -> Self {
		self.sudo.replace(sudo);
		self
	}

	pub fn build(self) -> sp_io::TestExternalities {
		// construct test storage for the mock runtime
		let mut t = frame_system::GenesisConfig::default().build_storage::<Runtime>().unwrap();

		// prefund blances for tester accounts
		pallet_balances::GenesisConfig::<Runtime> {
			balances: self.balances.clone().into_iter().collect::<Vec<_>>(),
		}
		.assimilate_storage(&mut t)
		.unwrap();

		// setup sudo account
		if let Some(key) = self.sudo {
			pallet_sudo::GenesisConfig::<Runtime> { key: Some(key) }
				.assimilate_storage(&mut t)
				.unwrap();
		}

		let mut ext = sp_io::TestExternalities::new(t);
		ext.execute_with(|| System::set_block_number(1));

		ext
	}
}
