//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 2.0.1

#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::weights::{Weight, constants::RocksDbWeight as DbWeight};

pub struct WeightInfo;
impl pallet_escrow::WeightInfo for WeightInfo {
	fn create_factory() -> Weight {
		(30_036_000 as Weight)
			.saturating_add(DbWeight::get().reads(1 as Weight))
			.saturating_add(DbWeight::get().writes(2 as Weight))
	}
	fn create() -> Weight {
		(87_556_000 as Weight)
			.saturating_add(DbWeight::get().reads(3 as Weight))
			.saturating_add(DbWeight::get().writes(7 as Weight))
	}
	fn add_trusted_handlers(h: u32, ) -> Weight {
		(25_763_000 as Weight)
			.saturating_add((4_656_000 as Weight).saturating_mul(h as Weight))
			.saturating_add(DbWeight::get().reads(2 as Weight))
			.saturating_add(DbWeight::get().writes(1 as Weight))
			.saturating_add(DbWeight::get().writes((1 as Weight).saturating_mul(h as Weight)))
	}
	fn abort(h: u32, f: u32, ) -> Weight {
		(145_556_000 as Weight)
			.saturating_add((893_000 as Weight).saturating_mul(h as Weight))
			.saturating_add((339_000 as Weight).saturating_mul(f as Weight))
			.saturating_add(DbWeight::get().reads(4 as Weight))
			.saturating_add(DbWeight::get().writes(8 as Weight))
			.saturating_add(DbWeight::get().writes((1 as Weight).saturating_mul(h as Weight)))
	}
	fn cancel() -> Weight {
		(122_983_000 as Weight)
			.saturating_add(DbWeight::get().reads(3 as Weight))
			.saturating_add(DbWeight::get().writes(2 as Weight))
	}
	fn complete() -> Weight {
		(36_158_000 as Weight)
			.saturating_add(DbWeight::get().reads(3 as Weight))
			.saturating_add(DbWeight::get().writes(1 as Weight))
	}
	fn note_intermediate_results() -> Weight {
		(50_986_000 as Weight)
			.saturating_add(DbWeight::get().reads(3 as Weight))
	}
	fn store_final_results() -> Weight {
		(35_708_000 as Weight)
			.saturating_add(DbWeight::get().reads(3 as Weight))
			.saturating_add(DbWeight::get().writes(1 as Weight))
	}
	fn bulk_payout(b: u32, ) -> Weight {
		(335_054_000 as Weight)
			.saturating_add((74_610_000 as Weight).saturating_mul(b as Weight))
			.saturating_add(DbWeight::get().reads(6 as Weight))
			.saturating_add(DbWeight::get().reads((1 as Weight).saturating_mul(b as Weight)))
			.saturating_add(DbWeight::get().writes(4 as Weight))
			.saturating_add(DbWeight::get().writes((1 as Weight).saturating_mul(b as Weight)))
	}
}
