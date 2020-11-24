#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};
use frame_support::{
	decl_error, decl_event, decl_module, decl_storage,
	dispatch::{DispatchError, DispatchResult},
	ensure,
	storage::{with_transaction, TransactionOutcome},
	traits::{Get, Currency, ExistenceRequirement::AllowDeath},
};
use frame_system::ensure_signed;
use sp_runtime::{
	traits::{Hash, Saturating, AccountIdConversion},
	ModuleId, Percent,
};
use sp_std::prelude::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

mod benchmarks;

use pallet_timestamp as timestamp;

pub type EscrowId = u128;

const MODULE_ID: ModuleId = ModuleId(*b"escrowhp");

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
	account: AccountId,
}

#[derive(Clone, Encode, Decode, Debug, PartialEq, Eq)]
pub struct ResultInfo {
	results_url: Vec<u8>,
	results_hash: Vec<u8>,
}

#[derive(Copy, Clone, Debug, Encode, Decode, PartialEq, Eq)]
pub enum EscrowStatus {
	Pending,
	Partial,
	Paid,
	Complete,
	Cancelled,
}

// Copied from ORML because the built-in `transactional` attribute doesn't work correctly in FRAME 2.0
pub fn with_transaction_result<R>(f: impl FnOnce() -> Result<R, DispatchError>) -> Result<R, DispatchError> {
	with_transaction(|| {
		let res = f();
		if res.is_ok() {
			TransactionOutcome::Commit(res)
		} else {
			TransactionOutcome::Rollback(res)
		}
	})
}

pub trait Trait: frame_system::Trait + timestamp::Trait {
	type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;
	type StandardDuration: Get<Self::Moment>;
	type StringLimit: Get<usize>;
	type Currency: Currency<Self::AccountId>;
	type BulkBalanceLimit: Get<BalanceOf<Self>>;
	type BulkAccountsLimit: Get<usize>;
	type HandlersLimit: Get<usize>;
}

pub type BalanceOf<T> = <<T as Trait>::Currency as Currency<<T as frame_system::Trait>::AccountId>>::Balance;

decl_storage! {
	trait Store for Module<T: Trait> as Escrow {
		Escrows get(fn escrow): map hasher(twox_64_concat) EscrowId => Option<EscrowInfo<T::Moment, T::AccountId>>;

		Counter get(fn counter): EscrowId;

		FinalResults get(fn final_results): map hasher(twox_64_concat) EscrowId => Option<ResultInfo>;

		TrustedHandlers get(fn is_trusted_handler):
			double_map hasher(twox_64_concat) EscrowId, hasher(twox_64_concat) T::AccountId => bool;
	}
}

decl_event!(
	pub enum Event<T> where
		<T as frame_system::Trait>::AccountId,
	{
		/// The escrow is in Pending status \[escrow_id, creator, canceller, manifest_url, manifest_hash, escrow_account\]
		Pending(EscrowId, AccountId, AccountId, Vec<u8>, Vec<u8>, AccountId),
		IntermediateStorage(EscrowId, Vec<u8>, Vec<u8>),
		/// Bulk payout was executed. Completion indicated by the boolean
		BulkPayout(EscrowId, u128),
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
		/// Spenders and values length do not match in bulk transfer
		MismatchBulkTransfer,
		/// Too many spenders in the bulk transfer function
		TooManyTos,
		/// Transfer is too big for bulk transfer
		TransferTooBig,
		StringSize,
		TooManyHandlers,
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
			ensure!(manifest_url.len() <= T::StringLimit::get(), Error::<T>::StringSize);
			ensure!(manifest_hash.len() <= T::StringLimit::get(), Error::<T>::StringSize);
			ensure!(handlers.len() <= T::HandlersLimit::get(), Error::<T>::TooManyHandlers);
			// This is fine as `100 + 100 < 256` so no chance of overflow.
			let total_stake = reputation_oracle_stake.deconstruct().saturating_add(recording_oracle_stake.deconstruct());
			ensure!(total_stake <= 100, Error::<T>::StakeOutOfBounds);
			let end_time = <timestamp::Module<T>>::get() + T::StandardDuration::get();
			let id = Counter::get();
			let account = Self::account_id_for(id);
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
				account: account.clone(),
			};
			Counter::set(id + 1);
			<Escrows<T>>::insert(id, new_escrow);
			let mut trusted = vec![recording_oracle, reputation_oracle, canceller.clone(), who.clone()];
			trusted.extend(handlers);
			Self::add_trusted_handlers(id, trusted);
			Self::deposit_event(RawEvent::Pending(id, who, canceller, manifest_url, manifest_hash, account));
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
			// TODO: Should this really go to the canceller?
			T::Currency::transfer(&escrow.account, &escrow.canceller, balance, AllowDeath)?;
			<Escrows<T>>::remove(id);
			<TrustedHandlers<T>>::remove_prefix(id);
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
			// TODO: Should this really go to the canceller?
			T::Currency::transfer(&escrow.account, &escrow.canceller, balance, AllowDeath)?;
			escrow.status = EscrowStatus::Cancelled;
			<Escrows<T>>::insert(id, escrow);
		}

		#[weight = 0]
		fn complete(origin, id: EscrowId) {
			let who = ensure_signed(origin)?;
			ensure!(Self::is_trusted_handler(id, who), Error::<T>::NonTrustedAccount);
			let mut escrow = Self::escrow(id).ok_or(Error::<T>::MissingEscrow)?;
			ensure!(escrow.end_time > <timestamp::Module<T>>::get(), Error::<T>::EscrowExpired);
			ensure!(escrow.status == EscrowStatus::Paid, Error::<T>::EscrowNotPaid);
			escrow.status = EscrowStatus::Complete;
			<Escrows<T>>::insert(id, escrow);
		}

		#[weight = 0]
		fn store_results(origin, id: EscrowId, url: Vec<u8>, hash: Vec<u8>) {
			let who = ensure_signed(origin)?;
			ensure!(url.len() <= T::StringLimit::get(), Error::<T>::StringSize);
			ensure!(hash.len() <= T::StringLimit::get(), Error::<T>::StringSize);
			ensure!(Self::is_trusted_handler(id, who), Error::<T>::NonTrustedAccount);
			let escrow = Self::escrow(id).ok_or(Error::<T>::MissingEscrow)?;
			ensure!(escrow.end_time > <timestamp::Module<T>>::get(), Error::<T>::EscrowExpired);
			ensure!(escrow.status == EscrowStatus::Pending || escrow.status == EscrowStatus::Partial, Error::<T>::EscrowClosed);
			Self::deposit_event(RawEvent::IntermediateStorage(id, url, hash));
		}

		#[weight = 0]
		fn bulk_payout(origin,
			id: EscrowId,
			recipients: Vec<T::AccountId>,
			amounts: Vec<BalanceOf<T>>,
			results_url: Option<Vec<u8>>,
			results_hash: Option<Vec<u8>>,
			tx_id: u128
		) -> DispatchResult {
			with_transaction_result(|| -> DispatchResult {
				let who = ensure_signed(origin)?;
				ensure!(results_url.as_ref().map(|u| u.len()).unwrap_or_default() <= T::StringLimit::get(), Error::<T>::StringSize);
				ensure!(results_hash.as_ref().map(|h| h.len()).unwrap_or_default() <= T::StringLimit::get(), Error::<T>::StringSize);
				ensure!(Self::is_trusted_handler(id, &who), Error::<T>::NonTrustedAccount);
				let mut escrow = Self::escrow(id).ok_or(Error::<T>::MissingEscrow)?;
				ensure!(escrow.end_time > <timestamp::Module<T>>::get(), Error::<T>::EscrowExpired);
				let balance = Self::get_balance(&escrow);
				ensure!(balance > 0.into(), Error::<T>::OutOfFunds);
				ensure!(escrow.status != EscrowStatus::Paid, Error::<T>::AlreadyPaid);

				let mut sum: BalanceOf<T> = 0.into();
				for a in amounts.iter() {
					sum = sum.saturating_add(*a);
				}
				if balance < sum {
					return Err(Error::<T>::OutOfFunds.into());
				}
				if results_url.is_some() || results_hash.is_some() {
					let new_results = ResultInfo {
						results_url: results_url.unwrap_or_default(),
						results_hash: results_hash.unwrap_or_default(),
					};
					// TODO: Is it fine to override this on every bulk payout?
					<FinalResults>::insert(id, new_results);
				}
				let (reputation_fee, recording_fee, final_amounts) = Self::finalize_payouts(&escrow, &amounts);

				let address = escrow.account.clone();
				
				T::Currency::transfer(&address.clone(), &escrow.reputation_oracle.clone(), reputation_fee, AllowDeath)?;
				T::Currency::transfer(&address.clone(), &escrow.recording_oracle.clone(), recording_fee, AllowDeath)?;
				Self::do_transfer_bulk(address, recipients, final_amounts)?;

				let balance = Self::get_balance(&escrow);
				if escrow.status == EscrowStatus::Pending {
					escrow.status = EscrowStatus::Partial;
				}
				if balance == 0.into() && escrow.status == EscrowStatus::Partial {
					escrow.status = EscrowStatus::Paid;
				}
				<Escrows<T>>::insert(id, escrow);
				Self::deposit_event(RawEvent::BulkPayout(id, tx_id));
				Ok(())
			})
		}
	}
}

impl<T: Trait> Module<T> {
	pub(crate) fn account_id_for(id: EscrowId) -> T::AccountId {
		MODULE_ID.into_sub_account(id)
	}
	
	pub(crate) fn add_trusted_handlers(id: EscrowId, trusted: Vec<T::AccountId>) {
		for trust in trusted {
			<TrustedHandlers<T>>::insert(id, trust, true);
		}
	}

	pub(crate) fn get_balance(escrow: &EscrowInfo<T::Moment, T::AccountId>) -> BalanceOf<T> {
		T::Currency::free_balance(&escrow.account)
	}

	pub(crate) fn finalize_payouts(
		escrow: &EscrowInfo<T::Moment, T::AccountId>,
		amounts: &Vec<BalanceOf<T>>,
	) -> (BalanceOf<T>, BalanceOf<T>, Vec<BalanceOf<T>>) {
		let mut reputation_fee_total: BalanceOf<T> = 0.into();
		let reputation_stake = escrow.reputation_oracle_stake;
		let mut recording_fee_total: BalanceOf<T> = 0.into();
		let recording_stake = escrow.recording_oracle_stake;
		let final_amounts = amounts
			.iter()
			.map(|amount| {
				// TODO: unclear whether this math is safe and has the intended semantics.
				let reputation_fee = reputation_stake.mul_floor(*amount);
				let recording_fee = recording_stake.mul_floor(*amount);
				let amount_without_fee = amount.saturating_sub(reputation_fee).saturating_sub(recording_fee);
				reputation_fee_total = reputation_fee_total.saturating_add(reputation_fee);
				recording_fee_total = recording_fee_total.saturating_add(recording_fee);
				amount_without_fee
			})
			.collect();
		(reputation_fee_total, recording_fee_total, final_amounts)
	}

	pub(crate) fn do_transfer_bulk(
		from: T::AccountId,
		tos: Vec<T::AccountId>,
		values: Vec<BalanceOf<T>>,
	) -> DispatchResult
	{
		ensure!(tos.len() <= T::BulkAccountsLimit::get(), Error::<T>::TooManyTos);
		ensure!(tos.len() == values.len(), Error::<T>::MismatchBulkTransfer);
		let mut sum: BalanceOf<T> = 0.into();
		for v in values.iter() {
			sum = sum.saturating_add(*v);
		}
		ensure!(sum <= T::BulkBalanceLimit::get(), Error::<T>::TransferTooBig);
		for (to, value) in tos.into_iter().zip(values.into_iter()) {
			T::Currency::transfer(&from, &to, value, AllowDeath)?;
		}
		Ok(())
	}
}
