use crate::{Module, Trait};
use frame_support::{impl_outer_origin, parameter_types, weights::Weight};
use frame_system as system;
use sp_core::H256;
use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, IdentityLookup},
	Perbill,
};

impl_outer_origin! {
	pub enum Origin for Test {}
}

// Configure a mock runtime to test the pallet.

#[derive(Clone, Eq, PartialEq)]
pub struct Test;
parameter_types! {
	pub const BlockHashCount: u64 = 250;
	pub const MaximumBlockWeight: Weight = 1024;
	pub const MaximumBlockLength: u32 = 2 * 1024;
	pub const AvailableBlockRatio: Perbill = Perbill::from_percent(75);
}

pub type AccountId = u64;

impl system::Trait for Test {
	type BaseCallFilter = ();
	type Origin = Origin;
	type Call = ();
	type Index = u64;
	type BlockNumber = u64;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type AccountId = AccountId;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Header = Header;
	type Event = ();
	type BlockHashCount = BlockHashCount;
	type MaximumBlockWeight = MaximumBlockWeight;
	type DbWeight = ();
	type BlockExecutionWeight = ();
	type ExtrinsicBaseWeight = ();
	type MaximumExtrinsicWeight = MaximumBlockWeight;
	type MaximumBlockLength = MaximumBlockLength;
	type AvailableBlockRatio = AvailableBlockRatio;
	type Version = ();
	type PalletInfo = ();
	type AccountData = ();
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
}

parameter_types! {
	pub const BulkAccountsLimit: usize = 10;
	pub const BulkBalanceLimit: u128 = 999;

}

impl pallet_hmtoken::Trait for Test {
	type Event = ();
	type Balance = u128;
	type BulkAccountsLimit = BulkAccountsLimit;
	type BulkBalanceLimit = BulkBalanceLimit;
	type WeightInfo = ();
}

parameter_types! {
	pub const MinimumPeriod: u64 = 6000 / 2;
}

pub type Moment = u64;

impl pallet_timestamp::Trait for Test {
	/// A timestamp: milliseconds since the unix epoch.
	type Moment = Moment;
	type OnTimestampSet = ();
	type MinimumPeriod = MinimumPeriod;
	type WeightInfo = ();
}

parameter_types! {
	pub const StandardDuration: Moment = 1000;
	pub const StringLimit: usize = 10;
}

impl Trait for Test {
	type Event = ();
	type StandardDuration = StandardDuration;
	type StringLimit = StringLimit;
}

pub type Escrow = Module<Test>;
pub type HmToken = pallet_hmtoken::Module<Test>;
pub type System = system::Module<Test>;

// Build genesis storage according to the mock runtime.
pub fn new_test_ext() -> sp_io::TestExternalities {
	let mut storage = system::GenesisConfig::default().build_storage::<Test>().unwrap();
	pallet_hmtoken::GenesisConfig::<Test> {
		total_supply: 1_000,
		name: b"Human Protocol Token".to_vec(),
		symbol: b"HMT".to_vec(),
		decimals: 15, // ignored in the UI for now
		initial_account: 1,
	}
	.assimilate_storage(&mut storage)
	.unwrap();
	let mut ext = sp_io::TestExternalities::from(storage);
	ext.execute_with(|| System::set_block_number(1));
	ext
}
