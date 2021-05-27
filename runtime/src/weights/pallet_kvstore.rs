//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 2.0.1

#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::weights::{Weight, constants::RocksDbWeight as DbWeight};

pub struct WeightInfo;
impl pallet_kvstore::WeightInfo for WeightInfo {
	fn set(k: u32, v: u32, ) -> Weight {
		(31_869_000 as Weight)
			.saturating_add((2_000 as Weight).saturating_mul(k as Weight))
			.saturating_add((2_000 as Weight).saturating_mul(v as Weight))
			.saturating_add(DbWeight::get().writes(1 as Weight))
	}
}
