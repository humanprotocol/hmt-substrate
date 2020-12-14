//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 2.0.0

#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::weights::{Weight, constants::RocksDbWeight as DbWeight};

pub struct WeightInfo;
impl pallet_escrow::WeightInfo for WeightInfo {
	fn create() -> Weight {
		(88_114_000 as Weight)
			.saturating_add(DbWeight::get().reads(2 as Weight))
			.saturating_add(DbWeight::get().writes(6 as Weight))
	}
	fn add_trusted_handlers(h: u32, ) -> Weight {
		(27_005_000 as Weight)
			.saturating_add((5_548_000 as Weight).saturating_mul(h as Weight))
			.saturating_add(DbWeight::get().reads(2 as Weight))
			.saturating_add(DbWeight::get().writes(1 as Weight))
			.saturating_add(DbWeight::get().writes((1 as Weight).saturating_mul(h as Weight)))
	}
	fn abort(h: u32, ) -> Weight {
		(177_919_000 as Weight)
			.saturating_add((276_000 as Weight).saturating_mul(h as Weight))
			.saturating_add(DbWeight::get().reads(3 as Weight))
			.saturating_add(DbWeight::get().writes(7 as Weight))
			.saturating_add(DbWeight::get().writes((1 as Weight).saturating_mul(h as Weight)))
	}
	fn cancel() -> Weight {
		(141_054_000 as Weight)
			.saturating_add(DbWeight::get().reads(3 as Weight))
			.saturating_add(DbWeight::get().writes(2 as Weight))
	}
	fn complete() -> Weight {
		(48_722_000 as Weight)
			.saturating_add(DbWeight::get().reads(3 as Weight))
			.saturating_add(DbWeight::get().writes(1 as Weight))
	}
	fn note_intermediate_results() -> Weight {
		(70_910_000 as Weight)
			.saturating_add(DbWeight::get().reads(3 as Weight))
	}
	fn store_final_results() -> Weight {
		(49_949_000 as Weight)
			.saturating_add(DbWeight::get().reads(3 as Weight))
			.saturating_add(DbWeight::get().writes(1 as Weight))
	}
	fn bulk_payout(b: u32, ) -> Weight {
		(294_401_000 as Weight)
			.saturating_add((67_825_000 as Weight).saturating_mul(b as Weight))
			.saturating_add(DbWeight::get().reads(6 as Weight))
			.saturating_add(DbWeight::get().reads((1 as Weight).saturating_mul(b as Weight)))
			.saturating_add(DbWeight::get().writes(4 as Weight))
			.saturating_add(DbWeight::get().writes((1 as Weight).saturating_mul(b as Weight)))
	}
}
