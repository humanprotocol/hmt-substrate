#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Encode, Decode};
use frame_support::{decl_error, decl_event, decl_module, ensure, decl_storage, dispatch, traits::Get};
use frame_system::ensure_signed;
use sp_runtime::traits::Hash;


// #[cfg(test)]
// mod mock;

// #[cfg(test)]
// mod tests;

use pallet_hmtoken as hmtoken;
use pallet_timestamp as timestamp;

pub type EscrowId = u128;

#[derive(Encode, Decode)]
pub struct EscrowInfo<Moment, AccountId> {
	end_time: Moment,
	manifest_url: Vec<u8>,
	manifest_hash: Vec<u8>,
	reputation_oracle: AccountId,
	recording_oracle: AccountId,
	reputation_oracle_stake: u128,
	recording_oracle_stake: u128,
	escrow_address: AccountId,
}

#[derive(Encode, Decode)]
pub enum EscrowStatus<Moment, AccountId> {
	None,
	Pending(EscrowInfo<Moment, AccountId>),
	Partial(EscrowInfo<Moment, AccountId>),
	Paid(EscrowInfo<Moment, AccountId>),
	Complete(EscrowInfo<Moment, AccountId>),
	Cancelled(EscrowInfo<Moment, AccountId>),
}

impl<Moment, AccountId> Default for EscrowStatus<Moment, AccountId> {
	fn default() -> EscrowStatus<Moment, AccountId> {
		EscrowStatus::None
	}
}

pub trait Trait: frame_system::Trait + timestamp::Trait {
	type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;
	type StandardDuration: Get<Self::Moment>;
}

decl_storage! {
	trait Store for Module<T: Trait> as Escrow {
		Escrow get(fn escrow): map hasher(twox_64_concat) EscrowId => EscrowStatus<T::Moment, T::AccountId>;
		
		Counter: EscrowId;

		TrustedHandler get(fn is_trusted_handler):
			double_map hasher(twox_64_concat) EscrowId, hasher(twox_64_concat) T::AccountId => bool;
	}
}

decl_event!(
	pub enum Event<T>
	where
		AccountId = <T as frame_system::Trait>::AccountId
		{
			/// The escrow was launched \[escrow_id, canceller\]
			Launched(EscrowId, AccountId),
			/// The escrow is in Pending status \[escrow_id, manifest_url, manifest_hash\]
			Pending(EscrowId, Vec<u8>, Vec<u8>),
		}
);

decl_error! {
	pub enum Error for Module<T: Trait> {
		StakeOutOfBounds,
		Overflow
	}
}
 
decl_module! {
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		type Error = Error<T>;

		fn deposit_event() = default;

		#[weight = 0]
		pub fn create(origin,
			canceler: T::AccountId, 
			handlers: Vec<T::AccountId>, 
			manifest_url: Vec<u8>, 
			manifest_hash: Vec<u8>, 
			reputation_oracle: T::AccountId, 
			recording_oracle: T::AccountId, 
			reputation_oracle_stake: u128, 
			recording_oracle_stake: u128) {

			let who = ensure_signed(origin)?;
			let total_stake = reputation_oracle_stake.checked_add(recording_oracle_stake).ok_or(Error::<T>::Overflow)?;
			ensure!(total_stake >= 0 && total_stake <= 100, Error::<T>::StakeOutOfBounds);
			let end_time = <timestamp::Module<T>>::get() + T::StandardDuration::get();
			let id = Counter::get();
			let mut data = vec![];
			data.extend(id.encode());
			data.extend(manifest_url.clone());
			data.extend(manifest_hash.clone());
			let data_hash = T::Hashing::hash(&data);
			let escrow_address = T::AccountId::decode(&mut data_hash.as_ref()).unwrap_or_default();
			let new_escrow = EscrowInfo {
				end_time,
				manifest_url: manifest_url.clone(),
				manifest_hash: manifest_hash.clone(),
				reputation_oracle: reputation_oracle.clone(),
				recording_oracle: recording_oracle.clone(),
				reputation_oracle_stake,
				recording_oracle_stake,
				escrow_address,
			};
			Counter::set(id + 1);
			<Escrow<T>>::insert(id, EscrowStatus::Pending(new_escrow));
			let mut trusted = vec![recording_oracle, reputation_oracle, canceler, who];
			trusted.extend(handlers);
			Self::addTrustedHandlers(id, trusted);
			Self::deposit_event(RawEvent::Pending(id, manifest_url, manifest_hash));
		}
	}
}

impl<T: Trait> Module<T> {
	fn addTrustedHandlers(id: EscrowId, trusted: Vec<T::AccountId>) {
		for trust in trusted {
			<TrustedHandler<T>>::insert(id, trust, true);
		}
	}
}