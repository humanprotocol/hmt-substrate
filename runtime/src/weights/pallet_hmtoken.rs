//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 2.0.0

#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::weights::{Weight, constants::RocksDbWeight as DbWeight};

pub struct WeightInfo;
impl pallet_hmtoken::WeightInfo for WeightInfo {
	fn transfer() -> Weight {
		(51_142_000 as Weight)
			.saturating_add(DbWeight::get().reads(2 as Weight))
			.saturating_add(DbWeight::get().writes(2 as Weight))
	}
	fn transfer_bulk(a: u32, ) -> Weight {
		(53_073_000 as Weight)
			.saturating_add((26_177_000 as Weight).saturating_mul(a as Weight))
			.saturating_add(DbWeight::get().reads(1 as Weight))
			.saturating_add(DbWeight::get().reads((1 as Weight).saturating_mul(a as Weight)))
			.saturating_add(DbWeight::get().writes(1 as Weight))
			.saturating_add(DbWeight::get().writes((1 as Weight).saturating_mul(a as Weight)))
	}
}
