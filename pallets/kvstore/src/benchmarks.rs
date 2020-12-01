#![cfg(feature = "runtime-benchmarks")]

use super::*;
use sp_std::prelude::*;

use frame_system::{RawOrigin, EventRecord};
use frame_benchmarking::{benchmarks, whitelisted_caller};
use crate::Module as KVStore;

fn assert_last_event<T: Trait>(generic_event: <T as Trait>::Event) {
	let events = frame_system::Module::<T>::events();
	let system_event: <T as frame_system::Trait>::Event = generic_event.into();
	// compare to the last event record
	let EventRecord { event, .. } = &events[events.len() - 1];
	assert_eq!(event, &system_event);
}

benchmarks! {
	_ { }

	set {
		let k in 1..(T::StringLimit::get() as u32);
		let v in 1..(T::StringLimit::get() as u32);
		let caller: T::AccountId = whitelisted_caller();

		let junk_data = 111;
		let key = vec![junk_data; k as usize];
		let value = vec![junk_data; v as usize];

	} : set(RawOrigin::Signed(caller.clone()), key.clone(), value.clone())
	verify {
		assert_eq!(KVStore::<T>::get(&caller, &key), value);
		assert_last_event::<T>(RawEvent::Stored(caller, key, value).into())
	}

}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::mock::{Test, new_test_ext};
	use frame_support::assert_ok;
		
		#[test]
		fn test_KVStore() {
				new_test_ext().execute_with(|| {
					assert_ok!(test_benchmark_set::<Test>());
				});
		}

}