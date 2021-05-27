#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};
use frame_support::{
    decl_error, decl_event, decl_module, decl_storage,
    dispatch::{DispatchError, DispatchResult},
    ensure,
    storage::{with_transaction, TransactionOutcome},
    traits::{Currency, ExistenceRequirement::AllowDeath, Get},
    weights::Weight,
};
use frame_system::ensure_signed;
use sp_runtime::{
    traits::{AccountIdConversion, Saturating, Zero},
    ModuleId, Percent,
};
use sp_std::prelude::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

mod benchmarks;

use pallet_timestamp as timestamp;

/// Id used for storing all information related to an escrow.
pub type EscrowId = u128;

/// Id used for storing all information related to an factory.
pub type FactoryId = u128;

const MODULE_ID: ModuleId = ModuleId(*b"escrowhp");

/// Maximum escrows per factory; to avoid an unbounded weight when querying.
const MAX_ESCROWS_PER_FACTORY: usize = 20;

/// Configuration and state for an escrow.
#[derive(Clone, Encode, Decode, Debug, PartialEq, Eq)]
pub struct EscrowInfo<Moment, AccountId> {
    /// Current status of the escrow. Is created as `Pending`.
    status: EscrowStatus,
    /// The expiry time of the escrow.
    end_time: Moment,
    /// Location of the manifest specifying metadata for this escrow.
    manifest_url: Vec<u8>,
    manifest_hash: Vec<u8>,
    reputation_oracle: AccountId,
    recording_oracle: AccountId,
    /// The amount payed to the reputation oracle at bulk payout.
    reputation_oracle_stake: Percent,
    /// The amount payed to the recording oracle at bulk payout.
    recording_oracle_stake: Percent,
    /// The account that will be refunded to on cancel/abort.
    canceller: AccountId,
    /// The account id used to hold escrow funds.
    account: AccountId,
	/// The factory with which the escrow is associated.
	factory: FactoryId
}

/// Points to where the results for an escrow are stored.
#[derive(Clone, Encode, Decode, Debug, PartialEq, Eq)]
pub struct ResultInfo {
    results_url: Vec<u8>,
    results_hash: Vec<u8>,
}

/// Defines the status of an escrow.
///
/// Valid state transitions:
///
///    | [create]
///    v
/// Pending --> Partial --> Paid --> Complete
///    |           |
///    +-----------+----> Cancelled
#[derive(Copy, Clone, Debug, Encode, Decode, PartialEq, Eq)]
pub enum EscrowStatus {
    /// An escrow is pending when created. Open for results and can be cancelled.
    Pending,
    /// The escrow is partially fulfilled, including partial payout.
    Partial,
    /// The escrow is completely paid.
    Paid,
    /// The escrow is marked as complete and cannot be altered anymore.
    Complete,
    /// The escrow is cancelled and refunded.
    Cancelled,
}

// Copied from ORML because the built-in `transactional` attribute doesn't work correctly in FRAME 2.0
fn with_transaction_result<R>(
    f: impl FnOnce() -> Result<R, DispatchError>,
) -> Result<R, DispatchError> {
    with_transaction(|| {
        let res = f();
        if res.is_ok() {
            TransactionOutcome::Commit(res)
        } else {
            TransactionOutcome::Rollback(res)
        }
    })
}

/// The weight info trait for `pallet_escrow`.
pub trait WeightInfo {
    // fn create_factory() -> Weight;
    fn create() -> Weight;
    fn add_trusted_handlers(h: u32) -> Weight;
    fn abort(h: u32) -> Weight;
    fn cancel() -> Weight;
    fn complete() -> Weight;
    fn note_intermediate_results() -> Weight;
    fn store_final_results() -> Weight;
    fn bulk_payout(b: u32) -> Weight;
}

// default weights for tests
impl WeightInfo for () {
    // fn create_factory() -> Weight {
    //     0
    // }
    fn create() -> Weight {
        0
    }
    fn add_trusted_handlers(_h: u32) -> Weight {
        0
    }
    fn abort(_h: u32) -> Weight {
        0
    }
    fn cancel() -> Weight {
        0
    }
    fn complete() -> Weight {
        0
    }
    fn note_intermediate_results() -> Weight {
        0
    }
    fn store_final_results() -> Weight {
        0
    }
    fn bulk_payout(_b: u32) -> Weight {
        0
    }
}

pub trait Trait: frame_system::Trait + timestamp::Trait {
    type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;
    /// The duration for which an escrow stays open.
    type StandardDuration: Get<Self::Moment>;
    /// The maximum length for strings/byte arrays passed into functions.
    type StringLimit: Get<usize>;
    /// Currency implementation for doing transfers.
    type Currency: Currency<Self::AccountId>;
    /// The maximum balance that can be transferred via bulk transfer.
    type BulkBalanceLimit: Get<BalanceOf<Self>>;
    /// The maximum number of accounts that can be transferred to via bulk transfer.
    type BulkAccountsLimit: Get<usize>;
    /// The maximum amount of trusted handlers per escrow.
    ///
    /// *Note:* Not enforced, but used for weight estimation. Make sure to not add more trusted
    /// handlers than this.
    type HandlersLimit: Get<u32>;
    type WeightInfo: WeightInfo;
}

pub type BalanceOf<T> =
    <<T as Trait>::Currency as Currency<<T as frame_system::Trait>::AccountId>>::Balance;

decl_storage! {
    trait Store for Module<T: Trait> as Escrow {
        /// Used to determine the next escrow id for a new escrow.
        Counter get(fn counter): EscrowId;

        /// Used to determine the next factory id for a new factory.
        FactoryCounter get(fn factory_counter): FactoryId;

        /// Escrow storage. Stores configuration and state for an escorw.
        Escrows get(fn escrow): map hasher(twox_64_concat) EscrowId => Option<EscrowInfo<T::Moment, T::AccountId>>;

        /// List of all currently active jobs.
        // For supporting factory API query.
        EscrowFactory get(fn escrow_factory): map hasher(twox_64_concat) FactoryId => Vec<EscrowId>;

        /// Results storage for each escrow.
        FinalResults get(fn final_results): map hasher(twox_64_concat) EscrowId => Option<ResultInfo>;

        /// The privileged accounts associated with an escrow.
        // TODO: consider changing value type to `()` to save space
        TrustedHandlers get(fn is_trusted_handler):
            double_map hasher(twox_64_concat) EscrowId, hasher(twox_64_concat) T::AccountId => bool;

        /// The number of trusted handlers associated with an escrow.
        HandlersCount get(fn handlers_count): map hasher(twox_64_concat) EscrowId => u32;
    }
}

decl_event!(
    pub enum Event<T> where
        <T as frame_system::Trait>::AccountId,
    {
        /// The escrow is in Pending status. \[escrow_id, creator, manifest_url, manifest_hash, escrow_account\]
        Pending(EscrowId, AccountId, Vec<u8>, Vec<u8>, AccountId),
        /// Intermediate results can be found at the given url. \[escrow_id, url, hash\]
        IntermediateResults(EscrowId, Vec<u8>, Vec<u8>),
        /// Bulk payout was executed. \[escrow_id\]
        BulkPayout(EscrowId),
        /// Factory created successfully. \[escrow_id, creator\]
        FactoryCreated(FactoryId, AccountId),
    }
);

decl_error! {
    pub enum Error for Module<T: Trait> {
        /// The oracle stake given is invalid by exceeding 100%.
        StakeOutOfBounds,
        /// A calculation overflowed.
        Overflow,
        /// The escrow specified cannot be found in storage.
        MissingEscrow,
        /// The account associated with the origin does not have the privilege for the operation.
        NonTrustedAccount,
        /// There are not enough funds to execute transfers.
        OutOfFunds,
        /// The escrow has reached the end of its life.
        EscrowExpired,
        /// The escrow does not have `Paid` status.
        EscrowNotPaid,
        /// The escrow is either `Paid` or `Complete` and cannot be altered.
        EscrowClosed,
        /// Spenders and values length do not match in bulk transfer
        MismatchBulkTransfer,
        /// Too many spenders in the bulk transfer function
        TooManyTos,
        /// Transfer is too big for bulk transfer
        TransferTooBig,
        /// The strings/byte arrays exceed the allowed size.
        StringSize,
        /// Tried to add too many trusted handlers to an escrow.
        TooManyHandlers,
        /// Maximum escrows per factory limit reached.
        FactoryOutOfBounds,
		/// Factory does not exist with this Id.
		FactoryDoesNotExist
    }
}

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        type Error = Error<T>;

        fn deposit_event() = default;
		
		/// Create a new factyory.
        // #[weight = <T as Trait>::WeightInfo::create_factory()]
        #[weight = 10_000]
        pub fn create_factory(origin) {
            let who = ensure_signed(origin)?;

            let id = FactoryCounter::get();
            FactoryCounter::set(id + 1);

            EscrowFactory::insert(id, Vec::<EscrowId>::new());
            Self::deposit_event(RawEvent::FactoryCreated(id, who));
        }

        /// Create a new escrow with the given manifest and oracles.
        ///
        /// Oracles and sender will be set as trusted handlers.
        /// Sender is set as canceller of the escrow.
        /// Emits the escrow id with the `Pending` event.
        #[weight = <T as Trait>::WeightInfo::create()]
        pub fn create(origin,
            manifest_url: Vec<u8>,
            manifest_hash: Vec<u8>,
			factory_id: u128,
            reputation_oracle: T::AccountId,
            recording_oracle: T::AccountId,
            // TODO: consider renaming to fee
            reputation_oracle_stake: Percent,
            recording_oracle_stake: Percent,
        ) {
            let who = ensure_signed(origin)?;
            ensure!(manifest_url.len() <= T::StringLimit::get(), Error::<T>::StringSize);
            ensure!(manifest_hash.len() <= T::StringLimit::get(), Error::<T>::StringSize);
			ensure!(<EscrowFactory>::contains_key(factory_id), Error::<T>::FactoryDoesNotExist);
            let factory_escrows = <EscrowFactory>::get(factory_id);
			ensure!(factory_escrows.len() <= MAX_ESCROWS_PER_FACTORY, Error::<T>::FactoryOutOfBounds);
			// This is fine as `100 + 100 < 256`, so no chance of overflow.
            let total_stake = reputation_oracle_stake.deconstruct()
                .saturating_add(recording_oracle_stake.deconstruct());
            ensure!(total_stake <= 100, Error::<T>::StakeOutOfBounds);
            let end_time = <timestamp::Module<T>>::get() + T::StandardDuration::get();

            let id = Counter::get();
            Counter::set(id + 1);

            // Both oracles as well as the creator are trusted.
            let trusted = vec![&recording_oracle, &reputation_oracle, &who];
            HandlersCount::insert(id, trusted.len() as u32);
            Self::do_add_trusted_handlers(id, trusted.into_iter());

            let account = Self::account_id_for(id);
            let new_escrow = EscrowInfo {
                status: EscrowStatus::Pending,
                end_time,
                manifest_url: manifest_url.clone(),
                manifest_hash: manifest_hash.clone(),
                reputation_oracle,
                recording_oracle,
                reputation_oracle_stake,
                recording_oracle_stake,
                canceller: who.clone(),
                account: account.clone(),
				factory: factory_id,
            };
            <Escrows<T>>::insert(id, new_escrow);
            <EscrowFactory>::mutate(factory_id, |list| {
				list.push(id)
			});

            Self::deposit_event(RawEvent::Pending(id, who, manifest_url, manifest_hash, account));
        }

        /// Add the given accounts as trusted for escrow with `id`.
        ///
        /// Allows these accounts to execute privileged operations.
        /// Requires trusted handler privileges.
        #[weight = <T as Trait>::WeightInfo::add_trusted_handlers(handlers.len() as u32)]
        fn add_trusted_handlers(origin, id: EscrowId, handlers: Vec<T::AccountId>) {
            // TODO: The security [fix PR](https://github.com/hCaptcha/hmt-escrow/pull/247/files)
            //       checks against the launcher here. What should we do?
            let _ = Self::ensure_trusted(origin, id)?;
            let count = Self::handlers_count(id);
            let new_count = (count).saturating_add(handlers.len() as u32);
            ensure!(new_count <= T::HandlersLimit::get(), Error::<T>::TooManyHandlers);
            Self::do_add_trusted_handlers(id, handlers.iter());
            HandlersCount::insert(id, new_count);
        }

        /// Abort the escrow at `id` and refund any balance to the canceller defined in the escrow.
        ///
        /// Clears escrow state.
        /// Requires trusted handler privileges.
        #[weight = <T as Trait>::WeightInfo::abort(T::HandlersLimit::get() as u32)]
        fn abort(origin, id: EscrowId) {
            let escrow = Self::escrow(id).ok_or(Error::<T>::MissingEscrow)?;
            let _ = Self::ensure_trusted(origin, id)?;
            ensure!(!matches!(escrow.status, EscrowStatus::Complete | EscrowStatus::Paid), Error::<T>::EscrowClosed);
            let balance = Self::get_balance(&escrow);
            if balance > Zero::zero() {
                T::Currency::transfer(&escrow.account, &escrow.canceller, balance, AllowDeath)?;
            }
            <Escrows<T>>::remove(id);
            FinalResults::remove(id);
            <TrustedHandlers<T>>::remove_prefix(id);
            HandlersCount::remove(id);

            let mut escrows = <EscrowFactory>::take(escrow.factory);
            let index = escrows.binary_search(&id).map_err(|_| Error::<T>::MissingEscrow)?;
            escrows.remove(index);
            <EscrowFactory>::insert(escrow.factory, escrows);
        }

        /// Cancel the escrow at `id` and refund any balance to the canceller defined in the escrow.
        ///
        /// Requires trusted handler privileges.
        #[weight = <T as Trait>::WeightInfo::cancel()]
        fn cancel(origin, id: EscrowId) {
            let mut escrow = Self::escrow(id).ok_or(Error::<T>::MissingEscrow)?;
            let _ = Self::ensure_trusted(origin, id)?;
            ensure!(matches!(escrow.status, EscrowStatus::Pending | EscrowStatus::Partial), Error::<T>::EscrowClosed);
            let balance = Self::get_balance(&escrow);
            ensure!(balance > Zero::zero(), Error::<T>::OutOfFunds);
            T::Currency::transfer(&escrow.account, &escrow.canceller, balance, AllowDeath)?;
            escrow.status = EscrowStatus::Cancelled;
            <Escrows<T>>::insert(id, escrow);
        }

        /// Set the escrow at `id` to be complete.
        ///
        /// Prohibits further editing or payouts of the escrow.
        /// Requires trusted handler privileges.
        // TODO: What is the intended use of `complete`?
        #[weight = <T as Trait>::WeightInfo::complete()]
        fn complete(origin, id: EscrowId) {
            let mut escrow = Self::escrow(id).ok_or(Error::<T>::MissingEscrow)?;
            let _ = Self::ensure_trusted(origin, id)?;
            ensure!(escrow.end_time > <timestamp::Module<T>>::get(), Error::<T>::EscrowExpired);
            ensure!(escrow.status == EscrowStatus::Paid, Error::<T>::EscrowNotPaid);
            escrow.status = EscrowStatus::Complete;
            <Escrows<T>>::insert(id, escrow);
            // TODO: consider cleaning up state here
        }

        /// Note intermediate results by emitting the `IntermediateResults` event.
        ///
        /// Requires trusted handler privileges.
        #[weight = <T as Trait>::WeightInfo::note_intermediate_results()]
        fn note_intermediate_results(origin, id: EscrowId, url: Vec<u8>, hash: Vec<u8>) {
            ensure!(url.len() <= T::StringLimit::get(), Error::<T>::StringSize);
            ensure!(hash.len() <= T::StringLimit::get(), Error::<T>::StringSize);
            let _ = Self::ensure_trusted(origin, id)?;
            let _ = Self::get_open_escrow(id)?;
            Self::deposit_event(RawEvent::IntermediateResults(id, url, hash));
        }

        /// Store the url and hash of the final results in storage.
        ///
        /// Requires trusted handler privileges.
        #[weight = <T as Trait>::WeightInfo::store_final_results()]
        fn store_final_results(origin, id: EscrowId, url: Vec<u8>, hash: Vec<u8>) {
            // TODO: determine necessary conditions for this
            ensure!(url.len() <= T::StringLimit::get(), Error::<T>::StringSize);
            ensure!(hash.len() <= T::StringLimit::get(), Error::<T>::StringSize);
            let _ = Self::ensure_trusted(origin, id)?;
            let _ = Self::get_open_escrow(id)?;
            FinalResults::insert(id, ResultInfo { results_url: url, results_hash: hash});
        }

        /// Pay out `recipients` with `amounts`. Calculates and transfer oracle fees.
        ///
        /// Sets the escrow to `Paid` if all balance is spent, otherwise to `Partial`.
        /// Requires trusted handler privileges.
        #[weight = <T as Trait>::WeightInfo::bulk_payout(recipients.len() as u32)]
        fn bulk_payout(origin,
            id: EscrowId,
            recipients: Vec<T::AccountId>,
            amounts: Vec<BalanceOf<T>>,
        ) -> DispatchResult {
            with_transaction_result(|| -> DispatchResult {
                let mut escrow = Self::get_open_escrow(id)?;
                let _ = Self::ensure_trusted(origin, id)?;
                let balance = Self::get_balance(&escrow);
                ensure!(balance > Zero::zero(), Error::<T>::OutOfFunds);

                // make sure we have enough funds to pay
                let mut sum: BalanceOf<T> = Zero::zero();
                for a in amounts.iter() {
                    sum = sum.saturating_add(*a);
                }
                if balance < sum {
                    return Err(Error::<T>::OutOfFunds.into());
                }
                // calculate fees
                let (reputation_fee, recording_fee, final_amounts) = Self::finalize_payouts(&escrow, &amounts);
                // transfer oracle fees
                T::Currency::transfer(&escrow.account, &escrow.reputation_oracle, reputation_fee, AllowDeath)?;
                T::Currency::transfer(&escrow.account, &escrow.recording_oracle, recording_fee, AllowDeath)?;
                Self::do_transfer_bulk(&escrow.account, &recipients, &final_amounts)?;

                // set the escrow state according to payout
                let balance = Self::get_balance(&escrow);
                if escrow.status == EscrowStatus::Pending {
                    escrow.status = EscrowStatus::Partial;
                }
                if balance == Zero::zero() && escrow.status == EscrowStatus::Partial {
                    escrow.status = EscrowStatus::Paid;
                }
                <Escrows<T>>::insert(id, escrow);
                Self::deposit_event(RawEvent::BulkPayout(id));
                Ok(())
            })
        }
    }
}

impl<T: Trait> Module<T> {
    /// Determine the account id corresponding to an escrow id.
    pub(crate) fn account_id_for(id: EscrowId) -> T::AccountId {
        MODULE_ID.into_sub_account(id)
    }

    /// Add the given accounts as trusted handlers (privileged accounts).
    pub(crate) fn do_add_trusted_handlers<'a, I>(id: EscrowId, trusted: I)
    where
        I: Iterator<Item = &'a T::AccountId>,
    {
        for trust in trusted {
            <TrustedHandlers<T>>::insert(id, trust, true);
        }
    }

    /// Ensure the origin represents a trusted user account.
    pub fn ensure_trusted(origin: T::Origin, id: EscrowId) -> Result<T::AccountId, DispatchError> {
        let who = ensure_signed(origin)?;
        ensure!(
            Self::is_trusted_handler(id, &who),
            Error::<T>::NonTrustedAccount
        );
        Ok(who)
    }

    /// Get the balance associated with an escrow.
    pub fn get_balance(escrow: &EscrowInfo<T::Moment, T::AccountId>) -> BalanceOf<T> {
        T::Currency::free_balance(&escrow.account)
    }

    /// Get the escrow for `id` and check that it is not expired and
    /// has `Pending` or `Partial` status.
    pub fn get_open_escrow(
        id: EscrowId,
    ) -> Result<EscrowInfo<T::Moment, T::AccountId>, DispatchError> {
        let escrow = Self::escrow(id).ok_or(Error::<T>::MissingEscrow)?;
        ensure!(
            escrow.end_time > <timestamp::Module<T>>::get(),
            Error::<T>::EscrowExpired
        );
        ensure!(
            matches!(escrow.status, EscrowStatus::Pending | EscrowStatus::Partial),
            Error::<T>::EscrowClosed
        );
        Ok(escrow)
    }

    /// Determine the oracle fees for the given `escrow` and `amounts`.
    pub(crate) fn finalize_payouts(
        escrow: &EscrowInfo<T::Moment, T::AccountId>,
        amounts: &[BalanceOf<T>],
    ) -> (BalanceOf<T>, BalanceOf<T>, Vec<BalanceOf<T>>) {
        let mut reputation_fee_total: BalanceOf<T> = Zero::zero();
        let reputation_stake = escrow.reputation_oracle_stake;
        let mut recording_fee_total: BalanceOf<T> = Zero::zero();
        let recording_stake = escrow.recording_oracle_stake;
        let final_amounts = amounts
            .iter()
            .map(|amount| {
                let reputation_fee = reputation_stake.mul_floor(*amount);
                let recording_fee = recording_stake.mul_floor(*amount);
                let amount_without_fee = amount
                    .saturating_sub(reputation_fee)
                    .saturating_sub(recording_fee);
                reputation_fee_total = reputation_fee_total.saturating_add(reputation_fee);
                recording_fee_total = recording_fee_total.saturating_add(recording_fee);
                amount_without_fee
            })
            .collect();
        (reputation_fee_total, recording_fee_total, final_amounts)
    }

    /// Do a bulk transfer from the given account to the recepients.
    ///
    /// Will abort the bulk transfer at the first failing transfer.
    ///
    /// **Warning**: Will not revert the successful transfers on failure.
    /// Use with transactional storage if that is desired.
    pub(crate) fn do_transfer_bulk(
        from: &T::AccountId,
        tos: &[T::AccountId],
        values: &[BalanceOf<T>],
    ) -> DispatchResult {
        ensure!(
            tos.len() <= T::BulkAccountsLimit::get(),
            Error::<T>::TooManyTos
        );
        ensure!(tos.len() == values.len(), Error::<T>::MismatchBulkTransfer);
        let mut sum: BalanceOf<T> = Zero::zero();
        for v in values.iter() {
            sum = sum.saturating_add(*v);
        }
        ensure!(
            sum <= T::BulkBalanceLimit::get(),
            Error::<T>::TransferTooBig
        );
        for (to, value) in tos.into_iter().zip(values.into_iter()) {
            T::Currency::transfer(&from, to, *value, AllowDeath)?;
        }
        Ok(())
    }
}
