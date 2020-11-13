#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{decl_error, decl_event, decl_module, decl_storage, dispatch, traits::Get};
use frame_system::ensure_signed;
use pallet_timestamp as timestamp;
#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

use pallet_hmtoken as hmtoken;

pub type EscrowId = u128;

#[derive(Encode, Decode)]
pub struct EscrowInfo<Moment, AccountId> {
	duration: Moment,
	manifest_url: Vec<u8>,
	manifest_hash: Vec<u8>,
	reputation_oracle: AccountId,
	recording_oracle: AccountId,
	reputation_oracle_stake: u128,
	recording_oracle_stake: u128,
	escrow_address: AccountId,
}

#[derive(Encode, Decode)]
enum EscrowStatus {
	Launched(LaunchedInfo),
	Pending(EscrowInfo),
	Partial(EscrowInfo),
	Paid(EscrowInfo),
	Complete(EscrowInfo),
	Cancelled(EscrowInfo),
}

#[derive(Encode, Decode)]
struct LaunchedInfo<Moment, AccountId> {
	duration: Moment,
	canceler: AccountId,
}

impl LaunchedInfo {
	fn to_pending(self, manifest_url) -> EscrowInfo {

	}
}


pub trait Trait: frame_system::Trait + timestamp::Trait {
	type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;
}

decl_storage! {
	trait Store for Module<T: Trait> as Escrow {
		Escrow get(fn escrow): map hasher(twox_64_concat) EscrowId => EscrowInfo<T::Moment, T::AccountId>;
		
		Counter: EscrowId;

		TrustedHandler get(fn is_trusted_handler):
			double_map hasher(twox_64_concat) EscrowId, hasher(twox_64_concat) T::AccountId => bool;
	}
}

decl_event!(
	pub enum Event<T>
	where
		AccountId = <T as frame_system::Trait>::AccountId, {
			/// The escrow was launched \[escrow_id, canceller\]
			Launched(EscrowId, AccountId),
			/// The escrow is in Pending status \[escrow_id, manifest_url, manifest_hash\]
			Pending(EscrowId, Vec<u8>, Vec<u8>),
		}
);

decl_error! {
	pub enum Error for Module<T: Trait> {

	}
}

decl_module! {
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
//let _now = <timestamp::Module<T>>::get();
		type Error = Error<T>;
		type HMToken = hmtoken::Module<T>;

		// Events must be initialized if they are used by the pallet.
		fn deposit_event() = default;

        [#weight = 0]
        fn create(origin, canceler: T::AccountId, duration: T::Moment, handlers: Vec<T::AccountId>) {
            let who = ensure_signed(origin)?;
            let new_escrow = EscrowInfo {
				status: EscrowStatus::Launched,
				duration: <timestamp::Module<T>>::get();
			};
        }
		
	}
}
 
