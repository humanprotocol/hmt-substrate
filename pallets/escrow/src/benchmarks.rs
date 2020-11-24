#![cfg(feature = "runtime-benchmarks")]

use super::*;
use sp_std::prelude::*;

use crate::Module as Escrow;
use codec::Encode;
use frame_benchmarking::{account, benchmarks, whitelisted_caller};
use frame_system::{EventRecord, RawOrigin};

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

    create {
        let h in 1..(T::HandlersLimit::get() as u32);
        let s in 1..(T::StringLimit::get() as u32);
        let caller: T::AccountId = whitelisted_caller();
        let canceller: T::AccountId = account("canceller", 0, SEED);
        let handlers: Vec<T::AccountId> = (0..h).map(|h| account("handler", h, SEED)).collect();
        let junk = 42;
        let manifest_url = vec![junk; s as usize];
        let manifest_hash = vec![junk; s as usize];
        let reputation_oracle: T::AccountId = account("oracle", 0, SEED);
        let recording_oracle: T::AccountId = account("oracle", 1, SEED);
        let reputation_oracle_stake = Percent::from_percent(10);
        let recording_oracle_stake = Percent::from_percent(10);

    } : _(RawOrigin::Signed(caller.clone()), canceller.clone(), handlers.clone(), manifest_url.clone(), manifest_hash.clone(), reputation_oracle.clone(), recording_oracle.clone(), reputation_oracle_stake, recording_oracle_stake)
    verify {
        let id = 0;
        assert_eq!(Escrows::<T>::get(id).unwrap().canceller, canceller.clone());
        assert_eq!(Escrows::<T>::get(id).unwrap().status, EscrowStatus::Pending);
        let all_handlers = [handlers, vec![caller.clone(), canceller.clone(), reputation_oracle, recording_oracle]].concat();
        for handler in all_handlers {
            assert!(Escrow::<T>::is_trusted_handler(id, handler));
        }
        assert_last_event::<T>(RawEvent::Pending(id, caller, canceller, manifest_url, manifest_hash).into())
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
}
