// Ensure we're `no_std` when compiling for Wasm.
#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::dispatch;
use frame_support::traits::{Get, Vec};
use frame_support::{decl_error, decl_event, decl_module, decl_storage, ensure, Parameter, weights::Weight};
use frame_system::ensure_signed;
use sp_runtime::traits::{
    AtLeast32BitUnsigned, Member, Saturating, StaticLookup, Zero,
};

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

mod benchmarks;

/// The module configuration trait.
pub trait Trait: frame_system::Trait {
    /// The overarching event type.
    type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;

    /// The units in which we record balances.
    type Balance: Member + Parameter + AtLeast32BitUnsigned + Default + Copy;

    type BulkAccountsLimit: Get<usize>;
    type BulkBalanceLimit: Get<Self::Balance>;
    type WeightInfo: WeightInfo;
}

pub trait WeightInfo {
	fn transfer() -> Weight;
	fn transfer_bulk(a: u32, ) -> Weight;
}

/// Implement WeightInfo for the unit type for easy mocking/testing
impl WeightInfo for () {
    fn transfer() -> Weight { 0 }
	fn transfer_bulk(_a: u32, ) -> Weight { 0 }
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
        pub fn transfer(origin,
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
        // 	tx_id: u128

        // ){

        // }
    //     #[weight = 0]
    //     fn transfer_bulk(origin,
    //         tos: Vec<T::AccountId>,
    //         values: Vec<T::Balance>,
    //         tx_id: u128
    //     ){
    //         let from = ensure_signed(origin)?;
    //         let (bulk_count, failures) = Self::do_transfer_bulk(from, tos, values)?;
    //         Self::deposit_event(RawEvent::BulkTransfer(tx_id, bulk_count, failures));
    //     }
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
        pub Balances get(fn balance): map hasher(blake2_128_concat) T::AccountId => T::Balance;

        // Approve get(fn approved_amount):
        // 	double_map hasher(twox_64_concat) T::AccountId, hasher(blake2_128_concat) T::AccountId => T::Balance;

        /// The total unit supply of the asset.
        pub TotalSupply get(fn total_supply) config(): T::Balance;
        pub Name get(fn name) config(): Vec<u8>;
        pub Symbol get(fn symbol) config(): Vec<u8>;
        pub Decimals get(fn decimals) config(): u8;
    } add_extra_genesis {
        config(initial_account): T::AccountId;
        build(|config: &GenesisConfig<T>| {
            Balances::<T>::insert(&config.initial_account, config.total_supply);
        })
    }
}

// The main implementation block for the module.
impl<T: Trait> Module<T> {
    pub fn do_transfer(
        from: T::AccountId,
        to: T::AccountId,
        value: T::Balance,
    ) -> dispatch::DispatchResult {
        ensure!(!value.is_zero(), Error::<T>::AmountZero);
        let from_balance = Self::balance(&from);
        ensure!(from_balance >= value, Error::<T>::BalanceLow);

        <Balances<T>>::insert(&from, from_balance.saturating_sub(value));
        <Balances<T>>::mutate(&to, |balance| *balance = balance.saturating_add(value));
        Self::deposit_event(RawEvent::Transferred(from, to, value));

        Ok(())
    }

    pub fn do_transfer_bulk(
        from: T::AccountId,
        tos: Vec<T::AccountId>,
        values: Vec<T::Balance>,
    ) -> dispatch::DispatchResult
    {
        ensure!(tos.len() <= T::BulkAccountsLimit::get(), Error::<T>::TooManyTos);
        ensure!(tos.len() == values.len(), Error::<T>::MismatchBulkTransfer);
        let mut sum: T::Balance = 0.into();
        for v in values.iter() {
            sum = sum.saturating_add(*v);
        }
        ensure!(sum <= T::BulkBalanceLimit::get(), Error::<T>::TransferTooBig);
        for (to, value) in tos.into_iter().zip(values.into_iter()) {
            Self::do_transfer(from.clone(), to, value)?;
        }
        Ok(())
    }
}
