use crate::{
	mock::*, Error, EscrowId, EscrowInfo, EscrowStatus, Escrows, RawEvent, ResultInfo, Trait, TrustedHandlers,
};
use frame_support::{
	assert_noop, assert_ok,
	dispatch::{DispatchError, DispatchResult},
	storage::{StorageDoubleMap, StorageMap},
	traits::Currency,
};
use frame_system::EventRecord;
use sp_runtime::Percent;

#[derive(Debug, Default)]
struct EscrowBuilder {
	id: Option<EscrowId>,
	status: Option<EscrowStatus>,
	canceller: Option<AccountId>,
	manifest_url: Option<Vec<u8>>,
	manifest_hash: Option<Vec<u8>>,
	reputation_oracle: Option<AccountId>,
	recording_oracle: Option<AccountId>,
	reputation_oracle_stake: Option<Percent>,
	recording_oracle_stake: Option<Percent>,
	account: Option<AccountId>,
}

impl EscrowBuilder {
	pub fn new() -> Self {
		EscrowBuilder { ..Default::default() }
	}

	pub fn id(mut self, id: EscrowId) -> Self {
		self.id = Some(id);
		self
	}

	pub fn canceller(mut self, a: AccountId) -> Self {
		self.canceller = Some(a);
		self
	}

	pub fn reputation_oracle(mut self, a: AccountId) -> Self {
		self.reputation_oracle = Some(a);
		self
	}

	pub fn recording_oracle(mut self, a: AccountId) -> Self {
		self.recording_oracle = Some(a);
		self
	}

	pub fn reputation_stake(mut self, p: Percent) -> Self {
		self.reputation_oracle_stake = Some(p);
		self
	}

	pub fn recording_stake(mut self, p: Percent) -> Self {
		self.recording_oracle_stake = Some(p);
		self
	}

	pub fn manifest_url(mut self, u: Vec<u8>) -> Self {
		self.manifest_url = Some(u);
		self
	}

	pub fn manifest_hash(mut self, h: Vec<u8>) -> Self {
		self.manifest_hash = Some(h);
		self
	}

	pub fn build(self) -> EscrowInfo<Moment, AccountId> {
		let status = self.status.unwrap_or(EscrowStatus::Pending);
		let canceller = self.canceller.unwrap_or(1);
		let manifest_url = self.manifest_url.unwrap_or(b"some.url".to_vec());
		let manifest_hash = self.manifest_hash.unwrap_or(b"0xdev".to_vec());
		let reputation_oracle = self.reputation_oracle.unwrap_or(3);
		let recording_oracle = self.recording_oracle.unwrap_or(4);
		let reputation_oracle_stake = self.reputation_oracle_stake.unwrap_or(Percent::from_percent(10));
		let recording_oracle_stake = self.recording_oracle_stake.unwrap_or(Percent::from_percent(10));
		let id = self.id.unwrap_or(0);
		let account = Escrow::account_id_for(id);
		let end_time = 1000;
		EscrowInfo {
			status,
			end_time,
			canceller,
			manifest_url,
			manifest_hash,
			reputation_oracle,
			recording_oracle,
			reputation_oracle_stake,
			recording_oracle_stake,
			account,
		}
	}
}

fn create_escrow(sender: AccountId, e: &EscrowInfo<Moment, AccountId>) -> DispatchResult {
	let i = e.clone();
	Escrow::create(
		Origin::signed(sender),
		i.manifest_url,
		i.manifest_hash,
		i.reputation_oracle,
		i.recording_oracle,
		i.reputation_oracle_stake,
		i.recording_oracle_stake,
	)
}

fn store_escrow(sender: AccountId, e: &EscrowInfo<Moment, AccountId>) {
	assert_ok!(create_escrow(sender, e));
}

fn store_default_escrow(id: EscrowId, sender: AccountId) -> EscrowInfo<Moment, AccountId> {
	let i = EscrowBuilder::new().id(id).canceller(sender).build();
	store_escrow(sender, &i);
	i
}

fn set_status(id: EscrowId, status: EscrowStatus) -> DispatchResult {
	Escrows::<Test>::try_mutate(id, |e| -> DispatchResult {
		if let Some(escrow) = e {
			escrow.status = status;
			Ok(())
		} else {
			Err(DispatchError::Other("escrow missing"))
		}
	})
}

fn assert_last_event<T: Trait>(generic_event: <T as Trait>::Event) {
	let events = frame_system::Module::<T>::events();
	let system_event: <T as frame_system::Trait>::Event = generic_event.into();
	// compare to the last event record
	let EventRecord { event, .. } = &events[events.len() - 1];
	assert_eq!(event, &system_event);
}

#[test]
fn it_creates_escrow_instance() {
	new_test_ext().execute_with(|| {
		let sender = 1;
		let escrow = store_default_escrow(0, sender);
		assert_eq!(Escrow::escrow(0), Some(escrow.clone()));
		assert_eq!(Escrow::counter(), 1);
		// Check that sender and oracles were set as trusted handlers.
		let all_handlers = vec![escrow.reputation_oracle, escrow.recording_oracle, sender];
		for handler in all_handlers {
			assert!(Escrow::is_trusted_handler(0, handler));
		}

		// Every escrow gets a new id.
		store_default_escrow(1, sender);
		assert_eq!(Escrow::counter(), 2);
		assert_ne!(Escrow::escrow(0).unwrap().account, Escrow::escrow(1).unwrap().account);
	});
}

#[test]
fn create_negative_tests() {
	new_test_ext().execute_with(|| {
		let sender = 1;
		let id = 0;
		{
			let escrow = EscrowBuilder::new()
				.id(id)
				.reputation_stake(Percent::from_percent(80))
				.recording_stake(Percent::from_percent(80))
				.build();
			assert_noop!(create_escrow(sender, &escrow), Error::<Test>::StakeOutOfBounds);
		}
		{
			let escrow = EscrowBuilder::new().id(id).manifest_hash(vec![24; 101]).build();
			assert_noop!(create_escrow(sender, &escrow), Error::<Test>::StringSize);
		}
		{
			let escrow = EscrowBuilder::new().id(id).manifest_url(vec![24; 101]).build();
			assert_noop!(create_escrow(sender, &escrow), Error::<Test>::StringSize);
		}
	});
}

#[test]
fn add_trusted_handlers_positive_test() {
	new_test_ext().execute_with(|| {
		let sender = 1;
		let id = 0;
		let _ = store_default_escrow(id, sender);
		let handlers = vec![5, 6, 7];
		for handler in handlers.iter() {
			assert!(!Escrow::is_trusted_handler(0, handler));
		}
		assert_ok!(Escrow::add_trusted_handlers(
			Origin::signed(sender),
			id,
			handlers.clone()
		));
		for handler in handlers.iter() {
			assert!(Escrow::is_trusted_handler(0, handler));
		}
	});
}

#[test]
fn add_trusted_handlers_negative_test() {
	new_test_ext().execute_with(|| {
		let sender = 1;
		let id = 0;
		let _ = store_default_escrow(id, sender);
		let handlers = vec![5, 6, 7];
		assert_noop!(Escrow::add_trusted_handlers(
			Origin::signed(8),
			id,
			handlers
		), Error::<Test>::NonTrustedAccount);
	});
}

#[test]
fn abort_positive_tests() {
	new_test_ext().execute_with(|| {
		let sender = 1;
		let id = 0;
		let escrow = store_default_escrow(id, sender);
		assert!(Escrow::is_trusted_handler(id, sender));
		assert_ok!(Balances::transfer(Origin::signed(sender), escrow.account, 100));
		let balance_before = Balances::free_balance(sender);
		assert_ok!(Escrow::store_final_results(Origin::signed(sender), id, b"some.url".to_vec(), b"0xdev".to_vec()));
		assert_ok!(Escrow::abort(Origin::signed(sender), id));
		let balance_after = Balances::free_balance(sender);

		// escrow and trusted handlers should be removed after abort
		assert_eq!(Escrow::escrow(id), None);
		assert_eq!((balance_after - balance_before), 100);
		assert!(!Escrow::is_trusted_handler(id, sender));
		assert_eq!(Escrow::final_results(id), None);
	});
}

#[test]
fn abort_negative_tests() {
	new_test_ext().execute_with(|| {
		let sender = 1;
		let _ = store_default_escrow(0, sender);
		assert_noop!(Escrow::abort(Origin::signed(8), 0), Error::<Test>::NonTrustedAccount);
		// Set the trusted handler manually to trigger missing escrow error.
		TrustedHandlers::<Test>::insert(2, sender, true);
		assert_noop!(Escrow::abort(Origin::signed(1), 2), Error::<Test>::MissingEscrow);
		set_status(0, EscrowStatus::Complete).expect("setting status should work");
		assert_noop!(Escrow::abort(Origin::signed(1), 0), Error::<Test>::EscrowClosed);
		set_status(0, EscrowStatus::Paid).expect("setting status should work");
		assert_noop!(Escrow::abort(Origin::signed(1), 0), Error::<Test>::EscrowClosed);
	});
}

#[test]
fn cancel_positive_tests() {
	new_test_ext().execute_with(|| {
		let sender = 1;
		let id = 0;
		let escrow = store_default_escrow(id, sender);
		assert_ok!(Balances::transfer(Origin::signed(1), escrow.account, 100));
		assert_ok!(Escrow::cancel(Origin::signed(1), id));
		assert_eq!(Escrow::escrow(id).unwrap().status, EscrowStatus::Cancelled);
	});
}

#[test]
fn cancel_negative_tests() {
	new_test_ext().execute_with(|| {
		let sender = 1;
		let _ = store_default_escrow(0, sender);
		assert_noop!(Escrow::cancel(Origin::signed(8), 0), Error::<Test>::NonTrustedAccount);
		// Set the trusted handler manually to trigger missing escrow error.
		TrustedHandlers::<Test>::insert(2, sender, true);
		assert_noop!(Escrow::cancel(Origin::signed(1), 2), Error::<Test>::MissingEscrow);
		assert_noop!(Escrow::cancel(Origin::signed(1), 0), Error::<Test>::OutOfFunds);
		set_status(0, EscrowStatus::Complete).expect("setting status should work");
		assert_noop!(Escrow::cancel(Origin::signed(1), 0), Error::<Test>::EscrowClosed);
		set_status(0, EscrowStatus::Paid).expect("setting status should work");
		assert_noop!(Escrow::cancel(Origin::signed(1), 0), Error::<Test>::EscrowClosed);
	});
}

#[test]
fn complete_positive_tests() {
	new_test_ext().execute_with(|| {
		let sender = 1;
		let _ = store_default_escrow(0, sender);
		set_status(0, EscrowStatus::Paid).expect("setting status should work");
		assert_ok!(Escrow::complete(Origin::signed(1), 0));
		assert_eq!(Escrow::escrow(0).unwrap().status, EscrowStatus::Complete);
	});
}

#[test]
fn complete_negative_tests() {
	new_test_ext().execute_with(|| {
		let sender = 1;
		let _ = store_default_escrow(0, sender);
		assert_noop!(Escrow::complete(Origin::signed(8), 0), Error::<Test>::NonTrustedAccount);
		// Set the trusted handler manually to trigger missing escrow error.
		TrustedHandlers::<Test>::insert(2, sender, true);
		assert_noop!(
			Escrow::complete(Origin::signed(sender), 2),
			Error::<Test>::MissingEscrow
		);
		assert_noop!(
			Escrow::complete(Origin::signed(sender), 0),
			Error::<Test>::EscrowNotPaid
		);
		Timestamp::set_timestamp(1001);
		assert_noop!(
			Escrow::complete(Origin::signed(sender), 0),
			Error::<Test>::EscrowExpired
		);
	});
}

#[test]
fn store_results_positive_tests() {
	new_test_ext().execute_with(|| {
		let sender = 1;
		let id = 0;
		let _ = store_default_escrow(id, sender);
		let url = b"results.url".to_vec();
		let hash = b"0xdev".to_vec();
		assert_ok!(Escrow::note_intermediate_results(
			Origin::signed(1),
			id,
			url.clone(),
			hash.clone()
		));
		assert_last_event::<Test>(RawEvent::<Test>::IntermediateResults(id, url, hash).into());
	});
}

#[test]
fn store_results_negative_tests() {
	new_test_ext().execute_with(|| {
		let sender = 1;
		let id = 0;
		let _ = store_default_escrow(id, sender);
		let url = b"results.url".to_vec();
		let hash = b"0xdev".to_vec();
		let long_url = vec![24; 101];
		let long_hash = vec![33; 101];
		assert_noop!(
			Escrow::note_intermediate_results(Origin::signed(8), id, url.clone(), hash.clone()),
			Error::<Test>::NonTrustedAccount
		);
		// Set the trusted handler manually to trigger missing escrow error.
		TrustedHandlers::<Test>::insert(2, sender, true);
		assert_noop!(
			Escrow::note_intermediate_results(Origin::signed(1), 2, url.clone(), hash.clone()),
			Error::<Test>::MissingEscrow
		);
		set_status(id, EscrowStatus::Cancelled).expect("setting status should work");
		assert_noop!(
			Escrow::note_intermediate_results(Origin::signed(1), id, url.clone(), hash.clone()),
			Error::<Test>::EscrowClosed
		);
		assert_noop!(
			Escrow::note_intermediate_results(Origin::signed(1), id, long_url.clone(), hash.clone()),
			Error::<Test>::StringSize
		);
		assert_noop!(
			Escrow::note_intermediate_results(Origin::signed(1), id, url.clone(), long_hash.clone()),
			Error::<Test>::StringSize
		);
		Timestamp::set_timestamp(1001);
		assert_noop!(
			Escrow::note_intermediate_results(Origin::signed(1), id, url.clone(), hash.clone()),
			Error::<Test>::EscrowExpired
		);
	});
}

#[test]
fn store_final_results_positive_tests() {
	new_test_ext().execute_with(|| {
		let sender = 1;
		let id = 0;
		let _ = store_default_escrow(id, sender);
		let url = b"results.url".to_vec();
		let hash = b"0xdev".to_vec();
		assert_ok!(Escrow::store_final_results(
			Origin::signed(sender),
			id,
			url.clone(),
			hash.clone()
		));
		let results_url = url.clone();
		let results_hash = hash.clone();
		assert_eq!(
			Escrow::final_results(id),
			Some(ResultInfo {
				results_url,
				results_hash
			})
		);
	})
}

#[test]
fn store_final_results_negative_tests() {
	new_test_ext().execute_with(|| {
		let sender = 1;
		let id = 0;
		let _ = store_default_escrow(id, sender);
		let url = b"results.url".to_vec();
		let hash = b"0xdev".to_vec();
		let long_url = vec![23; 101];
		let long_hash = vec![23; 101];
		assert_noop!(
			Escrow::note_intermediate_results(Origin::signed(8), id, url.clone(), hash.clone()),
			Error::<Test>::NonTrustedAccount
		);
		// Set the trusted handler manually to trigger missing escrow error.
		TrustedHandlers::<Test>::insert(2, sender, true);
		assert_noop!(
			Escrow::note_intermediate_results(Origin::signed(1), 2, url.clone(), hash.clone()),
			Error::<Test>::MissingEscrow
		);
		set_status(id, EscrowStatus::Cancelled).expect("setting status should work");
		assert_noop!(
			Escrow::note_intermediate_results(Origin::signed(1), id, url.clone(), hash.clone()),
			Error::<Test>::EscrowClosed
		);
		assert_noop!(
			Escrow::store_final_results(Origin::signed(1), id, url.clone(), long_hash.clone(),),
			Error::<Test>::StringSize
		);
		assert_noop!(
			Escrow::store_final_results(Origin::signed(1), id, long_url.clone(), hash.clone(),),
			Error::<Test>::StringSize
		);
		Timestamp::set_timestamp(1001);
		assert_noop!(
			Escrow::note_intermediate_results(Origin::signed(1), id, url.clone(), hash.clone()),
			Error::<Test>::EscrowExpired
		);
	});
}

#[test]
fn bulk_payout_positive_tests() {
	new_test_ext().execute_with(|| {
		let sender = 1;
		let rep_oracle = 3;
		let rec_oracle = 4;
		let recipients = vec![5, 6];
		let amounts = vec![10, 10];
		let id = 0;
		let escrow = EscrowBuilder::new()
			.id(id)
			.reputation_oracle(rep_oracle)
			.reputation_stake(Percent::from_percent(10))
			.recording_oracle(rec_oracle)
			.recording_stake(Percent::from_percent(10))
			.build();
		store_escrow(sender, &escrow);
		assert_ok!(Balances::transfer(Origin::signed(1), escrow.account, 40));
		assert_ok!(Escrow::bulk_payout(
			Origin::signed(1),
			id,
			recipients.clone(),
			amounts.clone(),
		));
		assert_last_event::<Test>(RawEvent::<Test>::BulkPayout(id).into());
		assert_eq!(Balances::free_balance(rep_oracle), 2);
		assert_eq!(Balances::free_balance(rec_oracle), 2);
		assert_eq!(Balances::free_balance(recipients[0]), 8);
		assert_eq!(Balances::free_balance(recipients[1]), 8);

		assert_eq!(Escrow::escrow(0).unwrap().status, EscrowStatus::Partial);
		assert_ok!(Escrow::bulk_payout(Origin::signed(1), id, recipients.clone(), amounts,));
		assert_eq!(Escrow::escrow(0).unwrap().status, EscrowStatus::Paid);
	});
}

#[test]
fn bulk_payout_negative_tests() {
	new_test_ext().execute_with(|| {
		let sender = 1;
		let rep_oracle = 3;
		let rec_oracle = 4;
		let mut recipients = vec![5, 6];
		let amounts = vec![10, 10];
		let id = 0;
		let escrow = EscrowBuilder::new()
			.id(id)
			.reputation_oracle(rep_oracle)
			.reputation_stake(Percent::from_percent(10))
			.recording_oracle(rec_oracle)
			.recording_stake(Percent::from_percent(10))
			.build();
		store_escrow(sender, &escrow);
		// Set the trusted handler manually to trigger missing escrow error.
		TrustedHandlers::<Test>::insert(2, sender, true);
		assert_noop!(
			Escrow::bulk_payout(Origin::signed(1), 2, recipients.clone(), amounts.clone(),),
			Error::<Test>::MissingEscrow
		);
		assert_noop!(
			Escrow::bulk_payout(Origin::signed(9), id, recipients.clone(), amounts.clone(),),
			Error::<Test>::NonTrustedAccount
		);
		assert_noop!(
			Escrow::bulk_payout(Origin::signed(1), id, recipients.clone(), amounts.clone(),),
			Error::<Test>::OutOfFunds
		);
		assert_ok!(Balances::transfer(Origin::signed(1), escrow.account, 10));
		assert_noop!(
			Escrow::bulk_payout(Origin::signed(1), id, recipients.clone(), amounts.clone(),),
			Error::<Test>::OutOfFunds
		);
		recipients.push(7);
		assert_ok!(Balances::transfer(Origin::signed(1), escrow.account, 20));
		assert_noop!(
			Escrow::bulk_payout(Origin::signed(1), id, recipients.clone(), amounts.clone(),),
			Error::<Test>::MismatchBulkTransfer
		);
		// no payout on failed bulk
		assert_eq!(Balances::free_balance(rep_oracle), 0);
		assert_eq!(Balances::free_balance(rec_oracle), 0);

		set_status(id, EscrowStatus::Paid).expect("setting status should work");
		assert_noop!(
			Escrow::bulk_payout(Origin::signed(1), id, recipients.clone(), amounts.clone(),),
			Error::<Test>::EscrowClosed
		);
		Timestamp::set_timestamp(1001);
		assert_noop!(
			Escrow::bulk_payout(Origin::signed(1), id, recipients.clone(), amounts.clone(),),
			Error::<Test>::EscrowExpired
		);
	})
}

#[test]
fn bulk_transfer_works() {
	new_test_ext().execute_with(|| {
		let amount: Balance = 10;
		let new_balance = 1000 - amount * 2;
		let from = 1;
		let first_rec = 2;
		let second_rec = 3;
		assert_ok!(Escrow::do_transfer_bulk(
			&from,
			&[first_rec, second_rec],
			&[amount, amount],
		));
		assert_eq!(Balances::free_balance(from), new_balance);
		assert_eq!(Balances::free_balance(first_rec), amount);
		assert_eq!(Balances::free_balance(second_rec), amount);
	});
}

#[test]
fn bulk_transfer_fails() {
	new_test_ext().execute_with(|| {
		let amount: Balance = 500_000_001;
		let from = 1;
		let first_rec = 2;
		let second_rec = 3;
		<Test as Trait>::Currency::make_free_balance_be(&from, 1_000_000_000);
		assert_noop!(
			Escrow::do_transfer_bulk(&from, &[first_rec], &[amount, amount],),
			Error::<Test>::MismatchBulkTransfer
		);
		assert_noop!(
			Escrow::do_transfer_bulk(&from, &[first_rec, second_rec], &[amount],),
			Error::<Test>::MismatchBulkTransfer
		);

		assert_noop!(
			Escrow::do_transfer_bulk(&from, &[first_rec; 11], &[amount; 11],),
			Error::<Test>::TooManyTos
		);
		assert_noop!(
			Escrow::do_transfer_bulk(&from, &[first_rec, second_rec], &[amount, amount],),
			Error::<Test>::TransferTooBig
		);
	});
}
