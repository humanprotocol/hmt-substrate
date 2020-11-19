#![cfg_attr(not(feature = "std"), no_std)]

use sp_std::prelude::*;
use codec::{Encode, Decode};
use frame_support::{decl_error, decl_event, decl_module, ensure, decl_storage, dispatch, traits::Get};
use frame_system::ensure_signed;
use sp_runtime::{Percent, traits::{Hash, Saturating}};

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

use pallet_hmtoken as hmtoken;
use pallet_timestamp as timestamp;

pub type EscrowId = u128;

#[derive(Clone, Encode, Decode, Debug, PartialEq, Eq)]
pub struct EscrowInfo<Moment, AccountId> {
	status: EscrowStatus,
	end_time: Moment,
	manifest_url: Vec<u8>,
	manifest_hash: Vec<u8>,
	reputation_oracle: AccountId,
	recording_oracle: AccountId,
	reputation_oracle_stake: Percent,
	recording_oracle_stake: Percent,
	canceller: AccountId,
	escrow_address: AccountId,
}

#[derive(Copy, Clone, Debug, Encode, Decode, PartialEq, Eq)]
pub enum EscrowStatus {
	Pending,
	Partial,
	Paid,
	Complete,
	Cancelled,
}

pub fn generate_account_id<T: Trait>(id: EscrowId, url: Vec<u8>, hash: Vec<u8>) -> T::AccountId {
	let mut data = vec![];
	data.extend(id.encode());
	data.extend(url);
	data.extend(hash);
	let data_hash = T::Hashing::hash(&data);
	T::AccountId::decode(&mut data_hash.as_ref()).unwrap_or_default()
}

pub trait Trait: frame_system::Trait + timestamp::Trait + hmtoken::Trait {
	type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;
	type StandardDuration: Get<Self::Moment>;
	type StringLimit: Get<usize>;
}

decl_storage! {
	trait Store for Module<T: Trait> as Escrow {
		Escrow get(fn escrow): map hasher(twox_64_concat) EscrowId => Option<EscrowInfo<T::Moment, T::AccountId>>;
		
		Counter get(fn counter): EscrowId;

		TrustedHandler get(fn is_trusted_handler):
			double_map hasher(twox_64_concat) EscrowId, hasher(twox_64_concat) T::AccountId => bool;
	}
}

decl_event!(
	pub enum Event<T> where
		<T as frame_system::Trait>::AccountId,
	{
		/// The escrow is in Pending status \[escrow_id, creator, canceller, manifest_url, manifest_hash\]
		Pending(EscrowId, AccountId, AccountId, Vec<u8>, Vec<u8>),
		IntermediateStorage(EscrowId, Vec<u8>, Vec<u8>),
		BulkPayoutAborted,
		/// Bulk payout was executed. Completion indicated by the boolean
		BulkPayout(bool),
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
		EscrowClosed,
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
			reputation_oracle_stake: Percent,
			recording_oracle_stake: Percent,
		) {
			let who = ensure_signed(origin)?;
			// This is fine as `100 + 100 < 256` so no chance of overflow.
			let total_stake = reputation_oracle_stake.deconstruct().saturating_add(recording_oracle_stake.deconstruct());
			ensure!(total_stake <= 100, Error::<T>::StakeOutOfBounds);
			let end_time = <timestamp::Module<T>>::get() + T::StandardDuration::get();
			// TODO check/limit the size of the url and hash
			let id = Counter::get();
			let escrow_address = generate_account_id::<T>(id, manifest_url.clone(), manifest_hash.clone());
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
			let mut trusted = vec![recording_oracle, reputation_oracle, canceller.clone(), who.clone()];
			trusted.extend(handlers);
			Self::add_trusted_handlers(id, trusted);
			Self::deposit_event(RawEvent::Pending(id, who, canceller, manifest_url, manifest_hash));
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
		fn store_results(origin, id: EscrowId, url: Vec<u8>, hash: Vec<u8>) {
			let who = ensure_signed(origin)?;
			let escrow = Self::escrow(id).ok_or(Error::<T>::MissingEscrow)?;
			ensure!(escrow.end_time > <timestamp::Module<T>>::get(), Error::<T>::EscrowExpired);
			ensure!(Self::is_trusted_handler(id, who), Error::<T>::NonTrustedAccount);
			ensure!(escrow.status == EscrowStatus::Pending || escrow.status == EscrowStatus::Partial, Error::<T>::EscrowClosed);
			Self::deposit_event(RawEvent::IntermediateStorage(id, url, hash));
		}

		#[weight = 0]
		fn bulk_payout(origin,
			id: EscrowId,
			recipients: Vec<T::AccountId>,
			amounts: Vec<T::Balance>,
			url: Vec<u8>,
			hash: Vec<u8>,
			tx_id: u128
		) {
			let who = ensure_signed(origin)?;
			let mut escrow = Self::escrow(id).ok_or(Error::<T>::MissingEscrow)?;
			ensure!(escrow.end_time > <timestamp::Module<T>>::get(), Error::<T>::EscrowExpired);
			ensure!(Self::is_trusted_handler(id, &who), Error::<T>::NonTrustedAccount);
			let balance = Self::get_balance(&escrow);
			ensure!(balance > 0.into(), Error::<T>::OutOfFunds);
			ensure!(escrow.status != EscrowStatus::Paid, Error::<T>::AlreadyPaid);

			let mut sum: T::Balance = 0.into();
            for a in amounts.iter() {
                sum = sum.saturating_add(*a);
			}
			if balance < sum {
				Self::deposit_event(RawEvent::BulkPayoutAborted);
				return Ok(());
			}
			if url.len() > 0 || hash.len() > 0 {
				// TODO: Store results as event like intermediate or in escrow storage?
			}
			let (reputation_fee, recording_fee, final_amounts) = Self::finalize_payouts(&escrow, &amounts);

			let (_, failures) = hmtoken::Module::<T>::do_transfer_bulk(who, recipients, final_amounts)?;
			let mut bulk_paid = false;
			if failures == 0 {
				let address = escrow.escrow_address.clone();
				// TODO: It seems easy to end up in inconsistent states. --> clarify requirements
				bulk_paid = hmtoken::Module::<T>::do_transfer(address.clone(), escrow.reputation_oracle.clone(), reputation_fee).is_ok() && hmtoken::Module::<T>::do_transfer(address.clone(), escrow.recording_oracle.clone(), recording_fee).is_ok();
			}
			let balance = Self::get_balance(&escrow);
			if bulk_paid {
				if escrow.status == EscrowStatus::Pending {
					escrow.status = EscrowStatus::Partial;
				}
				if balance == 0.into() && escrow.status == EscrowStatus::Partial {
					escrow.status = EscrowStatus::Paid;
				}
			}
			Self::deposit_event(RawEvent::BulkPayout(bulk_paid));
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

	fn finalize_payouts(escrow: &EscrowInfo<T::Moment, T::AccountId>, amounts: &Vec<T::Balance>) -> (T::Balance, T::Balance, Vec<T::Balance>) {
		let mut reputation_fee_total: T::Balance = 0.into();
		let reputation_stake = escrow.reputation_oracle_stake;
		let mut recording_fee_total: T::Balance = 0.into();
		let recording_stake = escrow.recording_oracle_stake;
		let final_amounts = amounts.iter().map(|amount| {
			// TODO: unclear whether this math is safe and has the intended semantics.
			let reputation_fee = reputation_stake.mul_floor(*amount);
			let recording_fee = recording_stake.mul_floor(*amount);
			let amount_without_fee = amount.saturating_sub(reputation_fee).saturating_sub(recording_fee);
			reputation_fee_total = reputation_fee_total.saturating_add(reputation_fee);
			recording_fee_total = recording_fee_total.saturating_add(recording_fee);
			amount_without_fee
		}).collect();
		(reputation_fee_total, recording_fee_total, final_amounts)
	}
}