//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 2.0.0

#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::weights::{Weight, constants::RocksDbWeight as DbWeight};

pub struct WeightInfo;
impl pallet_escrow::WeightInfo for WeightInfo {
	fn create() -> Weight {
		(112_165_000 as Weight)
			.saturating_add(DbWeight::get().reads(2 as Weight))
			.saturating_add(DbWeight::get().writes(5 as Weight))
	}
	fn add_trusted_handlers(h: u32, ) -> Weight {
		(16_999_000 as Weight)
			.saturating_add((4_323_000 as Weight).saturating_mul(h as Weight))
			.saturating_add(DbWeight::get().reads(1 as Weight))
			.saturating_add(DbWeight::get().writes((1 as Weight).saturating_mul(h as Weight)))
	}
	fn abort(h: u32, ) -> Weight {
		(240_983_000 as Weight)
			.saturating_add(DbWeight::get().reads(3 as Weight))
			.saturating_add(DbWeight::get().writes(5 as Weight))
			.saturating_add(DbWeight::get().writes((1 as Weight).saturating_mul(h as Weight)))
	}
	fn cancel() -> Weight {
		(153_356_000 as Weight)
			.saturating_add(DbWeight::get().reads(3 as Weight))
			.saturating_add(DbWeight::get().writes(2 as Weight))
	}
	fn complete() -> Weight {
		(48_519_000 as Weight)
			.saturating_add(DbWeight::get().reads(3 as Weight))
			.saturating_add(DbWeight::get().writes(1 as Weight))
	}
	fn note_intermediate_results() -> Weight {
		(67_591_000 as Weight)
			.saturating_add(DbWeight::get().reads(3 as Weight))
	}
	fn store_final_results() -> Weight {
		(49_339_000 as Weight)
			.saturating_add(DbWeight::get().reads(3 as Weight))
			.saturating_add(DbWeight::get().writes(1 as Weight))
	}
	fn bulk_payout(b: u32, ) -> Weight {
		(448_741_000 as Weight)
			.saturating_add((79_480_000 as Weight).saturating_mul(b as Weight))
			.saturating_add(DbWeight::get().reads(6 as Weight))
			.saturating_add(DbWeight::get().reads((1 as Weight).saturating_mul(b as Weight)))
			.saturating_add(DbWeight::get().writes(4 as Weight))
			.saturating_add(DbWeight::get().writes((1 as Weight).saturating_mul(b as Weight)))
	}
}
