//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 2.0.0

#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::weights::{Weight, constants::RocksDbWeight as DbWeight};

pub struct WeightInfo;
impl pallet_escrow::WeightInfo for WeightInfo {
	fn create(h: u32, s: u32, ) -> Weight {
		(96_290_000 as Weight)
			.saturating_add((3_361_000 as Weight).saturating_mul(h as Weight))
			.saturating_add((2_000 as Weight).saturating_mul(s as Weight))
			.saturating_add(DbWeight::get().reads(2 as Weight))
			.saturating_add(DbWeight::get().writes(6 as Weight))
			.saturating_add(DbWeight::get().writes((1 as Weight).saturating_mul(h as Weight)))
	}
	fn abort(h: u32, ) -> Weight {
		(152_871_000 as Weight)
			.saturating_add(DbWeight::get().reads(4 as Weight))
			.saturating_add(DbWeight::get().writes(7 as Weight))
			.saturating_add(DbWeight::get().writes((1 as Weight).saturating_mul(h as Weight)))
	}
	fn cancel() -> Weight {
		(155_549_000 as Weight)
			.saturating_add(DbWeight::get().reads(4 as Weight))
			.saturating_add(DbWeight::get().writes(3 as Weight))
	}
	fn complete() -> Weight {
		(49_148_000 as Weight)
			.saturating_add(DbWeight::get().reads(3 as Weight))
			.saturating_add(DbWeight::get().writes(1 as Weight))
	}
	fn store_results(s: u32, ) -> Weight {
		(53_721_000 as Weight)
			.saturating_add((17_000 as Weight).saturating_mul(s as Weight))
			.saturating_add(DbWeight::get().reads(3 as Weight))
	}
	fn bulk_payout(s: u32, b: u32, ) -> Weight {
		(0 as Weight)
			.saturating_add((854_000 as Weight).saturating_mul(s as Weight))
			.saturating_add((70_121_000 as Weight).saturating_mul(b as Weight))
			.saturating_add(DbWeight::get().reads(6 as Weight))
			.saturating_add(DbWeight::get().reads((1 as Weight).saturating_mul(b as Weight)))
			.saturating_add(DbWeight::get().writes(5 as Weight))
			.saturating_add(DbWeight::get().writes((1 as Weight).saturating_mul(b as Weight)))
	}
}
