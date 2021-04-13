#![cfg(feature = "runtime-benchmarks")]

use super::*;
use sp_std::prelude::*;

use crate::Module as Escrow;
use frame_benchmarking::{account, benchmarks, whitelisted_caller};
use frame_system::{EventRecord, RawOrigin};

pub type BalanceOf<T> = <<T as Trait>::Currency as Currency<<T as frame_system::Trait>::AccountId>>::Balance;

const SEED: u32 = 0;

fn assert_last_event<T: Trait>(generic_event: <T as Trait>::Event) {
	let events = frame_system::Module::<T>::events();
	let system_event: <T as frame_system::Trait>::Event = generic_event.into();
	// compare to the last event record
	let EventRecord { event, .. } = &events[events.len() - 1];
	assert_eq!(event, &system_event);
}

fn set_status<T: Trait>(id: EscrowId, status: EscrowStatus) -> DispatchResult {
	Escrows::<T>::try_mutate(id, |e| -> DispatchResult {
		if let Some(escrow) = e {
			escrow.status = status;
			Ok(())
		} else {
			Err(DispatchError::Other("escrow missing"))
		}
	})
}

benchmarks! {
	_ { }

	create {
		let caller: T::AccountId = whitelisted_caller();
		let junk = 42;
		let manifest_url = vec![junk; T::StringLimit::get()];
		let manifest_hash = vec![junk; T::StringLimit::get()];
		let reputation_oracle: T::AccountId = account("oracle", 0, SEED);
		let recording_oracle: T::AccountId = account("oracle", 1, SEED);
		let reputation_oracle_stake = Percent::from_percent(10);
		let recording_oracle_stake = Percent::from_percent(10);

	} : _(RawOrigin::Signed(caller.clone()), manifest_url.clone(), manifest_hash.clone(), reputation_oracle.clone(), recording_oracle.clone(), reputation_oracle_stake, recording_oracle_stake)
	verify {
		let id = 0;
		let escrow = Escrows::<T>::get(id).unwrap();
		assert_eq!(escrow.status, EscrowStatus::Pending);
		let all_handlers = vec![caller.clone(), reputation_oracle, recording_oracle];
		for handler in all_handlers {
			assert!(Escrow::<T>::is_trusted_handler(id, handler));
		}
		assert_last_event::<T>(RawEvent::Pending(id, caller, manifest_url, manifest_hash, Escrow::<T>::account_id_for(id)).into())
	}

	add_trusted_handlers {
		// By default `create` sets 3 trusted handlers (sender, rep_oracle, rec_oracle)
		let h in 1..((T::HandlersLimit::get() - 3) as u32);

		let caller: T::AccountId = whitelisted_caller();
		let handlers: Vec<T::AccountId> = (0..h).map(|h| account("handler", h, SEED)).collect();
		let junk = 42;
		let manifest_url = vec![junk; T::StringLimit::get()];
		let manifest_hash = vec![junk; T::StringLimit::get()];
		let reputation_oracle: T::AccountId = account("oracle", 0, SEED);
		let recording_oracle: T::AccountId = account("oracle", 1, SEED);
		let reputation_oracle_stake = Percent::from_percent(10);
		let recording_oracle_stake = Percent::from_percent(10);

		assert_eq!(Escrow::<T>::create(RawOrigin::Signed(caller.clone()).into(), manifest_url.clone(), manifest_hash.clone(), reputation_oracle.clone(), recording_oracle.clone(), reputation_oracle_stake, recording_oracle_stake), Ok(()));
		let id = 0;
	} : _(RawOrigin::Signed(caller.clone()), id, handlers.clone())
	verify {
		for handler in handlers {
			assert!(Escrow::<T>::is_trusted_handler(id, handler));
		}
	}

	abort {
		// By default `create` sets 3 trusted handlers (sender, rep_oracle, rec_oracle)
		let h in 1..((T::HandlersLimit::get() - 3) as u32);

		let caller: T::AccountId = whitelisted_caller();
		let handlers: Vec<T::AccountId> = (0..h).map(|h| account("handler", h, SEED)).collect();
		let junk = 42;
		let manifest_url = vec![junk; T::StringLimit::get()];
		let manifest_hash = vec![junk; T::StringLimit::get()];
		let reputation_oracle: T::AccountId = account("oracle", 0, SEED);
		let recording_oracle: T::AccountId = account("oracle", 1, SEED);
		let reputation_oracle_stake = Percent::from_percent(10);
		let recording_oracle_stake = Percent::from_percent(10);

		assert_eq!(Escrow::<T>::create(RawOrigin::Signed(caller.clone()).into(), manifest_url.clone(), manifest_hash.clone(), reputation_oracle.clone(), recording_oracle.clone(), reputation_oracle_stake, recording_oracle_stake), Ok(()));
		let id = 0;
		assert_eq!(Escrow::<T>::add_trusted_handlers(RawOrigin::Signed(caller.clone()).into(), id, handlers.clone()), Ok(()));
		let escrow = Escrows::<T>::get(id).unwrap();
		let amount = 1000u32;
		T::Currency::make_free_balance_be(&escrow.account, amount.into());
		assert_eq!(T::Currency::free_balance(&escrow.account), amount.into());
	} : _(RawOrigin::Signed(caller.clone()), id)
	verify {
		assert_eq!(Escrows::<T>::get(id), None);
		let all_handlers = [handlers, vec![caller.clone(), reputation_oracle, recording_oracle]].concat();
		for handler in all_handlers {
			assert!(!Escrow::<T>::is_trusted_handler(id, handler));
		}
		assert_eq!(T::Currency::free_balance(&escrow.account), Zero::zero());
		assert_eq!(T::Currency::free_balance(&caller), amount.into());
	}

	cancel {
		let caller: T::AccountId = whitelisted_caller();
		let junk = 42;
		let manifest_url = vec![junk; T::StringLimit::get()];
		let manifest_hash = vec![junk; T::StringLimit::get()];
		let reputation_oracle: T::AccountId = account("oracle", 0, SEED);
		let recording_oracle: T::AccountId = account("oracle", 1, SEED);
		let reputation_oracle_stake = Percent::from_percent(10);
		let recording_oracle_stake = Percent::from_percent(10);

		assert_eq!(Escrow::<T>::create(RawOrigin::Signed(caller.clone()).into(), manifest_url.clone(), manifest_hash.clone(), reputation_oracle.clone(), recording_oracle.clone(), reputation_oracle_stake, recording_oracle_stake), Ok(()));
		let id = 0;
		let escrow = Escrows::<T>::get(id).unwrap();
		let amount = 1000u32;
		T::Currency::make_free_balance_be(&escrow.account, amount.into());
	} : _(RawOrigin::Signed(caller.clone()), id)
	verify {
		assert_eq!(Escrows::<T>::get(id).unwrap().status, EscrowStatus::Cancelled);
		assert_eq!(T::Currency::free_balance(&escrow.account), Zero::zero());
		assert_eq!(T::Currency::free_balance(&caller), amount.into());
	}

	complete {
		let caller: T::AccountId = whitelisted_caller();
		let junk = 42;
		let manifest_url = vec![junk; T::StringLimit::get()];
		let manifest_hash = vec![junk; T::StringLimit::get()];
		let reputation_oracle: T::AccountId = account("oracle", 0, SEED);
		let recording_oracle: T::AccountId = account("oracle", 1, SEED);
		let reputation_oracle_stake = Percent::from_percent(10);
		let recording_oracle_stake = Percent::from_percent(10);

		assert_eq!(Escrow::<T>::create(RawOrigin::Signed(caller.clone()).into(), manifest_url.clone(), manifest_hash.clone(), reputation_oracle.clone(), recording_oracle.clone(), reputation_oracle_stake, recording_oracle_stake), Ok(()));
		let id = 0;
		set_status::<T>(id, EscrowStatus::Paid)?;
	} : _(RawOrigin::Signed(caller.clone()), id)
	verify {
		assert_eq!(Escrows::<T>::get(id).unwrap().status, EscrowStatus::Complete);
	}

	note_intermediate_results {
		let caller: T::AccountId = whitelisted_caller();
		let junk = 42;
		let manifest_url = vec![junk; T::StringLimit::get()];
		let manifest_hash = vec![junk; T::StringLimit::get()];
		let reputation_oracle: T::AccountId = account("oracle", 0, SEED);
		let recording_oracle: T::AccountId = account("oracle", 1, SEED);
		let reputation_oracle_stake = Percent::from_percent(10);
		let recording_oracle_stake = Percent::from_percent(10);

		assert_eq!(Escrow::<T>::create(RawOrigin::Signed(caller.clone()).into(), manifest_url.clone(), manifest_hash.clone(), reputation_oracle.clone(), recording_oracle.clone(), reputation_oracle_stake, recording_oracle_stake), Ok(()));
		let id = 0;
		let url = vec![junk; T::StringLimit::get()];
		let hash = vec![junk; T::StringLimit::get()];
	} : _(RawOrigin::Signed(caller.clone()), id, url.clone(), hash.clone())
	verify {
		assert_last_event::<T>(RawEvent::IntermediateResults(id, url, hash).into())
	}

	store_final_results {
		let caller: T::AccountId = whitelisted_caller();
		let junk = 42;
		let manifest_url = vec![junk; T::StringLimit::get()];
		let manifest_hash = vec![junk; T::StringLimit::get()];
		let reputation_oracle: T::AccountId = account("oracle", 0, SEED);
		let recording_oracle: T::AccountId = account("oracle", 1, SEED);
		let reputation_oracle_stake = Percent::from_percent(10);
		let recording_oracle_stake = Percent::from_percent(10);

		assert_eq!(Escrow::<T>::create(RawOrigin::Signed(caller.clone()).into(), manifest_url.clone(), manifest_hash.clone(), reputation_oracle.clone(), recording_oracle.clone(), reputation_oracle_stake, recording_oracle_stake), Ok(()));
		let id = 0;
		let url = vec![junk; T::StringLimit::get()];
		let hash = vec![junk; T::StringLimit::get()];
	} : _(RawOrigin::Signed(caller.clone()), id, url.clone(), hash.clone())
	verify {
		assert_eq!(FinalResults::get(id), Some(ResultInfo { results_url: url, results_hash: hash}));
	}

	bulk_payout {
		let b in 1..(T::BulkAccountsLimit::get() as u32);

		let caller: T::AccountId = whitelisted_caller();
		let junk = 42;
		let manifest_url = vec![junk; T::StringLimit::get()];
		let manifest_hash = vec![junk; T::StringLimit::get()];
		let reputation_oracle: T::AccountId = account("oracle", 0, SEED);
		let recording_oracle: T::AccountId = account("oracle", 1, SEED);
		let reputation_oracle_stake = Percent::from_percent(10);
		let recording_oracle_stake = Percent::from_percent(10);

		assert_eq!(Escrow::<T>::create(RawOrigin::Signed(caller.clone()).into(), manifest_url.clone(), manifest_hash.clone(), reputation_oracle.clone(), recording_oracle.clone(), reputation_oracle_stake, recording_oracle_stake), Ok(()));
		let id = 0;
		let escrow = Escrows::<T>::get(id).unwrap();
		// Need a high enough value so we don't run into ExistentialDeposit issues for the oracles.
		let amount: BalanceOf<T> = 100_000u32.into();
		let total_amount = amount * b.into();
		T::Currency::make_free_balance_be(&escrow.account, total_amount.into());
		let recipients: Vec<T::AccountId> = (0..b).map(|b| account("recipient", b, SEED)).collect();
		let amounts = vec![amount; b as usize];
	} : _(RawOrigin::Signed(caller.clone()), id, recipients.clone(), amounts.clone())
	verify {
		assert_eq!(T::Currency::free_balance(&reputation_oracle), reputation_oracle_stake.mul_floor(total_amount));
		assert_eq!(T::Currency::free_balance(&recording_oracle), recording_oracle_stake.mul_floor(total_amount));
		let received =  amount - reputation_oracle_stake.mul_floor(amount) - recording_oracle_stake.mul_floor(amount);
		for r in recipients {
			assert_eq!(T::Currency::free_balance(&r), received);
		}
		assert_eq!(Escrows::<T>::get(id).unwrap().status, EscrowStatus::Paid);
		assert_last_event::<T>(RawEvent::BulkPayout(id).into());
	}

}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::mock::{new_test_ext, Test};
	use frame_support::assert_ok;

	#[test]
	fn escrow_create() {
		new_test_ext().execute_with(|| {
			assert_ok!(test_benchmark_create::<Test>());
		});
	}

	#[test]
	fn escrow_add_trusted_handlers() {
		new_test_ext().execute_with(|| {
			assert_ok!(test_benchmark_add_trusted_handlers::<Test>());
		});
	}

	#[test]
	fn escrow_abort() {
		new_test_ext().execute_with(|| {
			assert_ok!(test_benchmark_abort::<Test>());
		});
	}

	#[test]
	fn escrow_cancel() {
		new_test_ext().execute_with(|| {
			assert_ok!(test_benchmark_cancel::<Test>());
		});
	}

	#[test]
	fn escrow_complete() {
		new_test_ext().execute_with(|| {
			assert_ok!(test_benchmark_complete::<Test>());
		});
	}

	#[test]
	fn escrow_note_intermediate_results() {
		new_test_ext().execute_with(|| {
			assert_ok!(test_benchmark_note_intermediate_results::<Test>());
		});
	}

	#[test]
	fn escrow_store_final_results() {
		new_test_ext().execute_with(|| {
			assert_ok!(test_benchmark_store_final_results::<Test>());
		});
	}

	#[test]
	fn escrow_bulk_payout() {
		new_test_ext().execute_with(|| {
			assert_ok!(test_benchmark_bulk_payout::<Test>());
		});
	}
}
