use crate::{Error, mock::*, RawEvent};
use frame_support::{assert_ok, assert_noop};

fn last_event() -> TestEvent {
	frame_system::Module::<Test>::events().pop().expect("Event expected").event
}

#[test]
fn get_and_set_work() {
	new_test_ext().execute_with(|| {
		// Dispatch a signed extrinsic.
		assert_ok!(KVStore::set(Origin::signed(1), vec![1,2,3,4], vec![5,6,7,8]));
		assert_eq!(last_event(), TestEvent::KVStorePallet(RawEvent::Stored(1, vec![1,2,3,4], vec![5,6,7,8])));

		// Read pallet storage and assert an expected result.
		assert_eq!(KVStore::get(1, vec![1,2,3,4]), vec![5,6,7,8]);

		// Use a module function to set the value.
		assert_ok!(KVStore::set_for_account(&42, &vec![1,2,3], &vec![6,7,8]));
		// Read pallet storage and assert an expected result.
		assert_eq!(KVStore::get(42, vec![1,2,3]), vec![6,7,8]);
	});
}

#[test]
fn string_limit_enforced() {
	new_test_ext().execute_with(|| {
		// Ensure the expected error is thrown when the key or value is too long.
		assert_noop!(
			KVStore::set(Origin::signed(1), vec![21; 100], vec![1,2,3]),
			Error::<Test>::KeyTooLong
		);
		assert_noop!(
			KVStore::set(Origin::signed(1), vec![1,2,3], vec![21; 100]),
			Error::<Test>::ValueTooLong
		);
	});
}
