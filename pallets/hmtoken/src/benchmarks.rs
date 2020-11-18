#![cfg(feature = "runtime-benchmarks")]

use super::*;
use sp_std::prelude::*;

use codec::Encode;
use frame_system::{RawOrigin, EventRecord};
use frame_benchmarking::{benchmarks, account, whitelisted_caller};
use crate::Module as HMToken;

const SEED: u32 = 0;

fn assert_last_event<T: Trait>(generic_event: <T as Trait>::Event) {
	let events = frame_system::Module::<T>::events();
	let system_event: <T as frame_system::Trait>::Event = generic_event.into();
	// compare to the last event record
	let EventRecord { event, .. } = &events[events.len() - 1];
	assert_eq!(event, &system_event);
}

benchmarks! {
	_ { }

	transfer {
		let caller: T::AccountId = whitelisted_caller();
		let recipient: T::AccountId = account("recipient", 0, SEED);
		let recipient_lookup: <T::Lookup as StaticLookup>::Source = T::Lookup::unlookup(recipient.clone());
		
		// set up account balance
		let initial: T::Balance = 1_000.into();
		Balances::<T>::insert(caller.clone(), initial);
		let value: T::Balance = 100.into();

	} : _(RawOrigin::Signed(caller.clone()), recipient_lookup, value)
	verify {
		assert_eq!(HMToken::<T>::balance(&caller), initial - value);
		assert_eq!(HMToken::<T>::balance(&recipient), value);
		assert_last_event::<T>(RawEvent::Transferred(caller, recipient, value).into())
	}

	transfer_bulk {
		let a in 1..(T::BulkAccountsLimit::get() as u32);
		let caller: T::AccountId = whitelisted_caller();
		let recipients: Vec<T::AccountId> = (0..a).map(|i| account("recipient", i, SEED)).collect();
		
		// set up account balance
		let initial: T::Balance = T::BulkBalanceLimit::get();
		Balances::<T>::insert(caller.clone(), initial);
		let value: T::Balance = 10.into();
		let values = vec![value; a as usize];

	} : _(RawOrigin::Signed(caller.clone()), recipients.clone(), values, 0)
	verify {
		let new_balance = initial - T::Balance::from(a) * value;
		assert_eq!(HMToken::<T>::balance(&caller), new_balance);
		let recipient: T::AccountId = account("recipient", 0, SEED);
		assert_eq!(HMToken::<T>::balance(&recipient), value);
		// Bulk transfer should succeed for all accounts
		assert_last_event::<T>(RawEvent::BulkTransfer(0, a, 0).into())
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::mock::{Test, new_test_ext};
	use frame_support::assert_ok;
		
		#[test]
		fn test_HMToken() {
				new_test_ext().execute_with(|| {
					assert_ok!(test_benchmark_transfer::<Test>());
					assert_ok!(test_benchmark_transfer_bulk::<Test>());
				});
		}

}