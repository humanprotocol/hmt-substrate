#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// https://substrate.dev/docs/en/knowledgebase/runtime/frame

use frame_support::{decl_module, decl_storage, decl_event, decl_error, dispatch, ensure, traits::Get};
use frame_system::ensure_signed;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

/// Configure the pallet by specifying the parameters and types on which it depends.
pub trait Trait: frame_system::Trait {
	/// Because this pallet emits events, it depends on the runtime's definition of an event.
	type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;
	type StringLimit: Get<usize>;
}

// The pallet's runtime storage items.
// https://substrate.dev/docs/en/knowledgebase/runtime/storage
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

// Pallets use events to inform users when important changes are made.
// https://substrate.dev/docs/en/knowledgebase/runtime/events
decl_event!(
	pub enum Event<T> where AccountId = <T as frame_system::Trait>::AccountId {
		/// Stored a value at (account id, key). [account id, key, value]
		Stored(AccountId, Vec<u8>, Vec<u8>),
	}
);

// Errors inform users that something went wrong.
decl_error! {
	pub enum Error for Module<T: Trait> {
		/// The given key exceeds `StringLimit`
		KeyTooLong,
		/// The given value exceeds `StringLimit`
		ValueTooLong,
	}
}

// Dispatchable functions allows users to interact with the pallet and invoke state changes.
// These functions materialize as "extrinsics", which are often compared to transactions.
// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
decl_module! {
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		type Error = Error<T>;

		fn deposit_event() = default;

		/// An example dispatchable that takes a singles value as a parameter, writes the value to
		/// storage and emits an event. This function must be dispatched by a signed extrinsic.
		#[weight = 10_000 + T::DbWeight::get().writes(1)]
		pub fn set(origin, key: Vec<u8>, value: Vec<u8>) -> dispatch::DispatchResult {
			let acc = ensure_signed(origin)?;

			Self::set_for_account(&acc, &key, &value)?;

			Self::deposit_event(RawEvent::Stored(acc, key, value));
			
			Ok(())
		}
	}
}

impl <T: Trait> Module<T> {
	pub fn set_for_account(acc: &T::AccountId, key: &[u8], value: &[u8]) -> dispatch::DispatchResult {
		ensure!(key.len() <= T::StringLimit::get(), Error::<T>::KeyTooLong);
		ensure!(value.len() <= T::StringLimit::get(), Error::<T>::ValueTooLong);

		Storage::<T>::insert(acc, key, value);

		Ok(())
	}
}