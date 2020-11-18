#![cfg_attr(not(feature = "std"), no_std)]

use sp_std::prelude::*;
use codec::{Encode, Decode};
use frame_support::{decl_error, decl_event, decl_module, ensure, decl_storage, dispatch::{DispatchError}, traits::Get};
use frame_system::ensure_signed;
use sp_runtime::traits::Hash;


// #[cfg(test)]
// mod mock;

// #[cfg(test)]
// mod tests;

use pallet_hmtoken as hmtoken;
use pallet_timestamp as timestamp;

pub type EscrowId = u128;

// TODO check the size of the urls and hash
#[derive(Encode, Decode)]
pub struct EscrowInfo<Moment, AccountId> {
	status: EscrowStatus,
	end_time: Moment,
	manifest_url: Vec<u8>,
	manifest_hash: Vec<u8>,
	reputation_oracle: AccountId,
	recording_oracle: AccountId,
	reputation_oracle_stake: u128,
	recording_oracle_stake: u128,
	canceller: AccountId,
	escrow_address: AccountId,
}

#[derive(Encode, Decode, PartialEq, Eq)]
pub enum EscrowStatus {
	Pending,
	Partial,
	Paid,
	Complete,
	Cancelled,
}

pub trait Trait: frame_system::Trait + timestamp::Trait + hmtoken::Trait {
	type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;
	type StandardDuration: Get<Self::Moment>;
}

decl_storage! {
	trait Store for Module<T: Trait> as Escrow {
		Escrow get(fn escrow): map hasher(twox_64_concat) EscrowId => Option<EscrowInfo<T::Moment, T::AccountId>>;
		
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
			IntermediateStorage(EscrowId, Vec<u8>, Vec<u8>)
		}
);

decl_error! {
	pub enum Error for Module<T: Trait> {
		StakeOutOfBounds,
		Overflow,
		MissingEscrow,
		NonTrustedAccount,
		AlreadyComplete,
		AlreadyPaid,
		OutOfFunds,
		EscrowExpired,
		EscrowNotPaid,
	}
}
 
decl_module! {
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		type Error = Error<T>;

		fn deposit_event() = default;

		#[weight = 0]
		pub fn create(origin,
			canceller: T::AccountId, 
			handlers: Vec<T::AccountId>, 
			manifest_url: Vec<u8>, 
			manifest_hash: Vec<u8>, 
			reputation_oracle: T::AccountId, 
			recording_oracle: T::AccountId, 
			reputation_oracle_stake: u128, 
			recording_oracle_stake: u128) {

			let who = ensure_signed(origin)?;
			let total_stake = reputation_oracle_stake.checked_add(recording_oracle_stake).ok_or(Error::<T>::Overflow)?;
			ensure!(total_stake <= 100, Error::<T>::StakeOutOfBounds);
			let end_time = <timestamp::Module<T>>::get() + T::StandardDuration::get();
			let id = Counter::get();
			let mut data = vec![];
			data.extend(id.encode());
			data.extend(manifest_url.clone());
			data.extend(manifest_hash.clone());
			let data_hash = T::Hashing::hash(&data);
			let escrow_address = T::AccountId::decode(&mut data_hash.as_ref()).unwrap_or_default();
			let new_escrow = EscrowInfo {
				status: EscrowStatus::Pending,
				end_time,
				manifest_url: manifest_url.clone(),
				manifest_hash: manifest_hash.clone(),
				reputation_oracle: reputation_oracle.clone(),
				recording_oracle: recording_oracle.clone(),
				reputation_oracle_stake,
				recording_oracle_stake,
				canceller: canceller.clone(),
				escrow_address,
			};
			Counter::set(id + 1);
			<Escrow<T>>::insert(id, new_escrow);
			let mut trusted = vec![recording_oracle, reputation_oracle, canceller, who];
			trusted.extend(handlers);
			Self::add_trusted_handlers(id, trusted);
			Self::deposit_event(RawEvent::Pending(id, manifest_url, manifest_hash));
		}

		#[weight = 0]
		fn abort(origin, id: EscrowId) {
			let who = ensure_signed(origin)?;
			ensure!(Self::is_trusted_handler(id, who), Error::<T>::NonTrustedAccount);
			let escrow = Self::escrow(id).ok_or(Error::<T>::MissingEscrow)?;
			ensure!(escrow.status != EscrowStatus::Complete, Error::<T>::AlreadyComplete);
			ensure!(escrow.status != EscrowStatus::Paid, Error::<T>::AlreadyPaid);
			let balance = Self::get_balance(&escrow);
			ensure!(balance > 0.into(), Error::<T>::OutOfFunds);
			hmtoken::Module::<T>::do_transfer(escrow.escrow_address.clone(), escrow.canceller.clone(), balance)?;
			<Escrow<T>>::remove(id);
		}
		
		#[weight = 0]
		fn cancel(origin, id: EscrowId) {
			let who = ensure_signed(origin)?;
			ensure!(Self::is_trusted_handler(id, who), Error::<T>::NonTrustedAccount);
			let mut escrow = Self::escrow(id).ok_or(Error::<T>::MissingEscrow)?;
			ensure!(escrow.status != EscrowStatus::Complete, Error::<T>::AlreadyComplete);
			ensure!(escrow.status != EscrowStatus::Paid, Error::<T>::AlreadyPaid);
			let balance = Self::get_balance(&escrow);
			ensure!(balance > 0.into(), Error::<T>::OutOfFunds);

			hmtoken::Module::<T>::do_transfer(escrow.escrow_address.clone(), escrow.canceller.clone(), balance)?;
			escrow.status = EscrowStatus::Cancelled;
			<Escrow<T>>::insert(id, escrow);
		}

		#[weight = 0]
		fn complete(origin, id: EscrowId) {
			let who = ensure_signed(origin)?;
			let mut escrow = Self::escrow(id).ok_or(Error::<T>::MissingEscrow)?;
			ensure!(escrow.end_time > <timestamp::Module<T>>::get(), Error::<T>::EscrowExpired);
			ensure!(Self::is_trusted_handler(id, who), Error::<T>::NonTrustedAccount);
			ensure!(escrow.status == EscrowStatus::Paid, Error::<T>::EscrowNotPaid);
			escrow.status = EscrowStatus::Complete;
			<Escrow<T>>::insert(id, escrow);
		}

		#[weight = 0]
		fn store_results(origin, url: Vec<u8>, hash: Vec<u8>){
			let who = ensure_signed(origin)?;
			ensure!(escrow.end_time > <timestamp::Module<T>>::get(), Error::<T>::EscrowExpired);
			ensure!(Self::is_trusted_handler(id, who), Error::<T>::NonTrustedAccount);
			ensure!(escrow.status == EscrowStatus::Pending || escrow.status == EscrowStatus::partial, Error::<T>::EscrowClosed);
			Self::deposit_event(RawEvent::IntermediateStorage(id, url, hash));
		}
	}
}

impl<T: Trait> Module<T> {
	fn add_trusted_handlers(id: EscrowId, trusted: Vec<T::AccountId>) {
		for trust in trusted {
			<TrustedHandler<T>>::insert(id, trust, true);
		}
	}

	pub fn get_balance(target_escrow: &EscrowInfo<T::Moment, T::AccountId>) -> T::Balance {
		hmtoken::Module::<T>::balance(target_escrow.escrow_address.clone())
	}
}