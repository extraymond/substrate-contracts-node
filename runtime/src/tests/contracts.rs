use super::{ExtBuilder, ALICE};

#[test]
fn test_basic() {
	ExtBuilder::default()
		.balances(vec![(ALICE, 100_000_000_000_000_000)])
		.sudo(ALICE)
		.build()
		.execute_with(|| {});
}
