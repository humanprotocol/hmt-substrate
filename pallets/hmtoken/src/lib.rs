// This file is part of Substrate.

// Copyright (C) 2017-2020 Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! # Assets Module
//!
//! A simple, secure module for dealing with fungible assets.
//!
//! ## Overview
//!
//! The Assets module provides functionality for asset management of fungible asset classes
//! with a fixed supply, including:
//!
//! * Asset Issuance
//! * Asset Transfer
//! * Asset Destruction
//!
//! To use it in your runtime, you need to implement the assets [`Trait`](./trait.Trait.html).
//!
//! The supported dispatchable functions are documented in the [`Call`](./enum.Call.html) enum.
//!
//! ### Terminology
//!
//! * **Asset issuance:** The creation of a new asset, whose total supply will belong to the
//!   account that issues the asset.
//! * **Asset transfer:** The action of transferring assets from one account to another.
//! * **Asset destruction:** The process of an account removing its entire holding of an asset.
//! * **Fungible asset:** An asset whose units are interchangeable.
//! * **Non-fungible asset:** An asset for which each unit has unique characteristics.
//!
//! ### Goals
//!
//! The assets system in Substrate is designed to make the following possible:
//!
//! * Issue a unique asset to its creator's account.
//! * Move assets between accounts.
//! * Remove an account's balance of an asset when requested by that account's owner and update
//!   the asset's total supply.
//!
//! ## Interface
//!
//! ### Dispatchable Functions
//!
//! * `issue` - Issues the total supply of a new fungible asset to the account of the caller of the function.
//! * `transfer` - Transfers an `amount` of units of fungible asset `id` from the balance of
//! the function caller's account (`origin`) to a `target` account.
//! * `destroy` - Destroys the entire holding of a fungible asset `id` associated with the account
//! that called the function.
//!
//! Please refer to the [`Call`](./enum.Call.html) enum and its associated variants for documentation on each function.
//!
//! ### Public Functions
//! <!-- Original author of descriptions: @gavofyork -->
//!
//! * `balance` - Get the asset `id` balance of `who`.
//! * `total_supply` - Get the total supply of an asset `id`.
//!
//! Please refer to the [`Module`](./struct.Module.html) struct for details on publicly available functions.
//!
//! ## Usage
//!
//! The following example shows how to use the Assets module in your runtime by exposing public functions to:
//!
//! * Issue a new fungible asset for a token distribution event (airdrop).
//! * Query the fungible asset holding balance of an account.
//! * Query the total supply of a fungible asset that has been issued.
//!
//! ### Prerequisites
//!
//! Import the Assets module and types and derive your runtime's configuration traits from the Assets module trait.
//!
//! ### Simple Code Snippet
//!
//! ```rust,ignore
//! use pallet_assets as assets;
//! use frame_support::{decl_module, dispatch, ensure};
//! use frame_system::ensure_signed;
//!
//! pub trait Trait: assets::Trait { }
//!
//! decl_module! {
//! 	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
//! 		pub fn issue_token_airdrop(origin) -> dispatch::DispatchResult {
//! 			let sender = ensure_signed(origin).map_err(|e| e.as_str())?;
//!
//! 			const ACCOUNT_ALICE: u64 = 1;
//! 			const ACCOUNT_BOB: u64 = 2;
//! 			const COUNT_AIRDROP_RECIPIENTS: u64 = 2;
//! 			const TOKENS_FIXED_SUPPLY: u64 = 100;
//!
//! 			ensure!(!COUNT_AIRDROP_RECIPIENTS.is_zero(), "Divide by zero error.");
//!
//! 			let asset_id = Self::next_asset_id();
//!
//! 			<NextAssetId<T>>::mutate(|asset_id| *asset_id += 1);
//! 			<Balances<T>>::insert((asset_id, &ACCOUNT_ALICE), TOKENS_FIXED_SUPPLY / COUNT_AIRDROP_RECIPIENTS);
//! 			<Balances<T>>::insert((asset_id, &ACCOUNT_BOB), TOKENS_FIXED_SUPPLY / COUNT_AIRDROP_RECIPIENTS);
//! 			<TotalSupply<T>>::insert(asset_id, TOKENS_FIXED_SUPPLY);
//!
//! 			Self::deposit_event(RawEvent::Issued(asset_id, sender, TOKENS_FIXED_SUPPLY));
//! 			Ok(())
//! 		}
//! 	}
//! }
//! ```
//!
//! ## Assumptions
//!
//! Below are assumptions that must be held when using this module.  If any of
//! them are violated, the behavior of this module is undefined.
//!
//! * The total count of assets should be less than
//!   `Trait::AssetId::max_value()`.
//!
//! ## Related Modules
//!
//! * [`System`](../frame_system/index.html)
//! * [`Support`](../frame_support/index.html)

// Ensure we're `no_std` when compiling for Wasm.
#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{Parameter, decl_module, decl_event, decl_storage, decl_error, ensure};
use frame_support::traits::Get;
use frame_support::dispatch;
use sp_runtime::traits::{Member, AtLeast32Bit, AtLeast32BitUnsigned, Zero, StaticLookup, Saturating};
use frame_system::ensure_signed;
use sp_runtime::traits::One;

// #[cfg(test)]
// mod mock;

// #[cfg(test)]
// mod tests;

/// The module configuration trait.
pub trait Trait: frame_system::Trait {
	/// The overarching event type.
	type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;

	/// The units in which we record balances.
	type Balance: Member + Parameter + AtLeast32BitUnsigned + Default + Copy;

	type BulkAccountsLimit: Get<usize>;
	type BulkBalanceLimit: Get<Self::Balance>;
}

decl_module! {
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		type Error = Error<T>;

		fn deposit_event() = default;
		
		/// Move some assets from one holder to another.
		///
		/// # <weight>
		/// - `O(1)`
		/// - 1 static lookup
		/// - 2 storage mutations (codec `O(1)`).
		/// - 1 event.
		/// # </weight>
		#[weight = 0]
		fn transfer(origin,
			to: <T::Lookup as StaticLookup>::Source,
			#[compact] value: T::Balance
		) {
			let from = ensure_signed(origin)?;
			let to = T::Lookup::lookup(to)?;

			Self::do_transfer(from, to, value)?;
		}
		
		//TODO talk to Client about this
		// #[weight = 0]
		// fn transfer_from(origin,
		// 	from: <T::Lookup as StaticLookup>::Source,
		// 	to: <T::Lookup as StaticLookup>::Source,
		// 	#[compact] value: T::Balance,
		// ) {
		// 	let spender = ensure_signed(origin)?;
		// 	let authorizer = T::Lookup::lookup(from)?;
		// 	let to = T::Lookup::lookup(to)?;
			
		// 	if Self::approved_amount(authorizer, spender) >= value {
		// 		Self::do_transfer(authorizer, to, value)?;
		// 	} else {
		// 		Error::<T>::NotApproved
		// 	}
		// }

		// #[weight = 0]
		// fn approve(origin,
		// 	spender: <T::Lookup as StaticLookup>::Source,
		// 	#[compact] value: T::Balance
		// ) {
		// 	let authorizer = ensure_signed(origin)?;
		// 	let spender = T::Lookup::lookup(spender)?;
		// 	Storage::<T>::insert(&authorizer, &spender, value);
		// 	Self::deposit_event(RawEvent::Approval(authorizer, spender, value));
		// }

		// #[weight = 0]
		// fn increase_approval(origin,
		// 	spender: <T::Lookup as StaticLookup>::Source,
		// 	#[compact] value: T::Balance
			
		// ) {
			
		// }

		// #[weight = 0]
		// fn decrease_approval(origin,
		// 	spender: <T::Lookup as StaticLookup>::Source,
		// 	#[compact] value: T::Balance
			
		// ){
			
		// }

		// #[weight = 0]
		// fn approve_bulk(origin,
		// 	spenders: [<T::Lookup as StaticLookup>::Source],
		// 	#[compact] values: [T::Balance],
		// 	txId: u128
			
		// ){
			
		// }
		
		#[weight = 0]
		fn transfer_bulk(origin,
			tos: Vec<T::AccountId>,
			values: Vec<T::Balance>,
			txId: u128
			
		){
			let from = ensure_signed(origin)?;
			ensure!(tos.len() <= T::BulkAccountsLimit::get(), Error::<T>::TooManyTos);
			ensure!(tos.len() == values.len(), Error::<T>::MismatchBulkTransfer);
			let mut sum: T::Balance = 0.into();
			for v in values.iter() {
				sum.saturating_add(*v);
			}
			ensure!(sum <= T::BulkBalanceLimit::get(), Error::<T>::TransferTooBig);
			let mut failures = 0;
			let mut bulk_count = 0;
			//TODO seem difficult for debugging which txs failed if they do
			for (to, value) in tos.into_iter().zip(values.into_iter()) {
				let result = Self::do_transfer(from.clone(), to, value);
				match result {
					Ok(()) => bulk_count += 1,
					Err(_) => failures += 1,
				}
			}
			Self::deposit_event(RawEvent::BulkTransfer(txId, bulk_count, failures));
		}
	}
}

decl_event! {
	pub enum Event<T> where
		<T as frame_system::Trait>::AccountId,
		<T as Trait>::Balance,
	{
		/// Some assets were issued. \[asset_id, owner, total_supply\]
		Issued(AccountId, Balance),
		/// Some assets were transferred. \[asset_id, from, to, amount\]
		Transferred(AccountId, AccountId, Balance),
		/// Some assets were destroyed. \[asset_id, owner, balance\]
		Destroyed(AccountId, Balance),
		// Approval(AccountId, AccountId, Balance),
		/// A bulk transfer was executed \[tx_id, successes, failures\]
		BulkTransfer(u128, u32, u32),
	}
}

decl_error! {
	pub enum Error for Module<T: Trait> {
		/// Transfer amount should be non-zero
		AmountZero,
		/// Account balance must be greater than or equal to the transfer amount
		BalanceLow,
		/// Balance should be non-zero
		BalanceZero,
		// NoApproval,
		/// Spenders and values length do not match in bulk transfer
		MismatchBulkTransfer,
		/// Too many spenders in the bulk transfer function
		TooManyTos,
		/// Transfer is too big for bulk transfer
		TransferTooBig
	}
}

decl_storage! {
	trait Store for Module<T: Trait> as HMToken {
		/// The number of units of assets held by any given account.
		Balances get(fn balance): map hasher(blake2_128_concat) T::AccountId => T::Balance;

		// Approve get(fn approved_amount):
		// 	double_map hasher(twox_64_concat) T::AccountId, hasher(blake2_128_concat) T::AccountId => T::Balance; 

		/// The total unit supply of the asset.
		TotalSupply get(fn total_supply): T::Balance;
		
	}
}

// The main implementation block for the module.
impl<T: Trait> Module<T> {
	
	pub fn do_transfer(from: T::AccountId, to: T::AccountId, value: T::Balance) -> dispatch::DispatchResult {
		ensure!(!value.is_zero(), Error::<T>::AmountZero);
		let from_balance = Self::balance(&from);
		ensure!(from_balance >= value, Error::<T>::BalanceLow);

		<Balances<T>>::insert(&from, from_balance.saturating_sub(value));
		<Balances<T>>::mutate(&to, |balance| *balance = balance.saturating_add(value));
		Self::deposit_event(RawEvent::Transferred(from, to, value));

		Ok(())
	}

	// pub fn approved_amount() -> T::Balance {
		
	// }

}



	
