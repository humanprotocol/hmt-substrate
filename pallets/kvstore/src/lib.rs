#![cfg_attr(not(feature = "std"), no_std)]

use sp_std::prelude::*;
use frame_support::{decl_module, decl_storage, decl_event, decl_error, dispatch, ensure, weights::Weight, traits::Get};
use frame_system::ensure_signed;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

mod benchmarks;

pub trait Trait: frame_system::Trait {
	type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;
	type StringLimit: Get<usize>;
	type WeightInfo: WeightInfo;
}

pub trait WeightInfo {
	fn set(k: u32, v: u32) -> Weight;
}

decl_storage! {
	trait Store for Module<T: Trait> as KVStore {
		/// The underlying storage for the key-value store.
		///
		/// Hasher note: twox_64 should be safe because account ids cannot be freely controlled by
		/// potential attackers. (Using pallets will have to keep to that constraint, though.)
		Storage get(fn get):
			double_map hasher(twox_64_concat) T::AccountId, hasher(blake2_128_concat) Vec<u8> => Vec<u8>;
	}
}

decl_event!(
	pub enum Event<T> where AccountId = <T as frame_system::Trait>::AccountId {
		/// Stored a value at (account id, key). [account id, key, value]
		Stored(AccountId, Vec<u8>, Vec<u8>),
	}
);

decl_error! {
	pub enum Error for Module<T: Trait> {
		/// The given key exceeds `StringLimit`
		KeyTooLong,
		/// The given value exceeds `StringLimit`
		ValueTooLong,
	}
}

decl_module! {
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		type Error = Error<T>;

		fn deposit_event() = default;

		/// Set the `value` under the sender's account id and `key`.
		#[weight = T::WeightInfo::set(key.len() as u32, value.len() as u32)]
		pub fn set(origin, key: Vec<u8>, value: Vec<u8>) -> dispatch::DispatchResult {
			let acc = ensure_signed(origin)?;

			Self::set_for_account(&acc, &key, &value)?;

			Self::deposit_event(RawEvent::Stored(acc, key, value));
			
			Ok(())
		}
	}
}

impl <T: Trait> Module<T> {
	/// Set the given `value` in the double map under `acc` and `key`.
	pub fn set_for_account(acc: &T::AccountId, key: &[u8], value: &[u8]) -> dispatch::DispatchResult {
		ensure!(key.len() <= T::StringLimit::get(), Error::<T>::KeyTooLong);
		ensure!(value.len() <= T::StringLimit::get(), Error::<T>::ValueTooLong);

		Storage::<T>::insert(acc, key, value);

		Ok(())
	}
}