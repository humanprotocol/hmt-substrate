use crate::{generate_account_id, mock::*, Error, EscrowId, EscrowInfo, EscrowStatus, Escrows, RawEvent, Trait};
use frame_support::{
	assert_noop, assert_ok,
	dispatch::{DispatchError, DispatchResult},
	storage::StorageMap,
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
	escrow_address: Option<AccountId>,
}

impl EscrowBuilder {
	pub fn new() -> Self {
		EscrowBuilder { ..Default::default() }
	}

	pub fn id(mut self, id: EscrowId) -> Self {
		self.id = Some(id);
		self
	}

	pub fn build(self) -> EscrowInfo<Moment, AccountId> {
		let status = self.status.unwrap_or(EscrowStatus::Pending);
		let canceller = self.canceller.unwrap_or(1);
		let manifest_url = self.manifest_url.unwrap_or(b"some.url".to_vec());
		let manifest_hash = self.manifest_hash.unwrap_or(b"0xdev".to_vec());
		let reputation_oracle = self.reputation_oracle.unwrap_or(3);
		let recording_oracle = self.recording_oracle.unwrap_or(4);
		let reputation_oracle_stake = self.reputation_oracle_stake.unwrap_or(Percent::from_percent(50));
		let recording_oracle_stake = self.recording_oracle_stake.unwrap_or(Percent::from_percent(50));
		let id = self.id.unwrap_or(0);
		let escrow_address = generate_account_id::<Test>(id, manifest_url.clone(), manifest_hash.clone());
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
			escrow_address,
		}
	}
}

fn create_base_escrow(id: EscrowId, sender: AccountId, handlers: Vec<AccountId>) -> EscrowInfo<Moment, AccountId> {
	let i = EscrowBuilder::new().id(id).build();
	let copy = i.clone();
	assert_ok!(Escrow::create(
		Origin::signed(sender),
		i.canceller,
		handlers,
		i.manifest_url,
		i.manifest_hash,
		i.reputation_oracle,
		i.recording_oracle,
		i.reputation_oracle_stake,
		i.recording_oracle_stake
	));
	copy
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
		let handlers = vec![1, 2];
		let escrow = create_base_escrow(0, sender, handlers.clone());
		assert_eq!(Escrow::escrow(0), Some(escrow.clone()));
		assert_eq!(Escrow::counter(), 1);
		let mut all_handlers = handlers.clone();
		all_handlers.extend(vec![
			escrow.canceller,
			escrow.reputation_oracle,
			escrow.recording_oracle,
			sender,
		]);
		for handler in all_handlers {
			assert!(Escrow::is_trusted_handler(0, handler));
		}

		create_base_escrow(1, sender, handlers);
		assert_eq!(Escrow::counter(), 2);
		assert_ne!(
			Escrow::escrow(0).unwrap().escrow_address,
			Escrow::escrow(1).unwrap().escrow_address
		);
	});
}

#[test]
fn abort_positive_tests() {
	new_test_ext().execute_with(|| {
		let sender = 1;
		let handlers = vec![1, 2];
		let id = 0;
		let escrow = create_base_escrow(id, sender, handlers);
		assert!(Escrow::is_trusted_handler(id, sender));
		assert_ok!(HmToken::transfer(Origin::signed(sender), escrow.escrow_address, 100));
		let balance_before = HmToken::balance(sender);
		assert_ok!(Escrow::abort(Origin::signed(sender), id));
		let balance_after = HmToken::balance(sender);

		assert_eq!(Escrow::escrow(id), None);
		assert_eq!((balance_after - balance_before), 100);
		assert!(!Escrow::is_trusted_handler(id, sender));
	});
}
#[test]
fn abort_negative_tests() {
	new_test_ext().execute_with(|| {
		let sender = 1;
		let handlers = vec![1, 2];
		let escrow = create_base_escrow(0, sender, handlers);
		assert_noop!(Escrow::abort(Origin::signed(8), 0), Error::<Test>::NonTrustedAccount);
		assert_noop!(Escrow::abort(Origin::signed(1), 2), Error::<Test>::MissingEscrow);
		assert_noop!(Escrow::abort(Origin::signed(1), 0), Error::<Test>::OutOfFunds);
		set_status(0, EscrowStatus::Complete).expect("setting status should work");
		assert_noop!(Escrow::abort(Origin::signed(1), 0), Error::<Test>::AlreadyComplete);
		set_status(0, EscrowStatus::Paid).expect("setting status should work");
		assert_noop!(Escrow::abort(Origin::signed(1), 0), Error::<Test>::AlreadyPaid);
	});
}

#[test]
fn cancel_positive_tests() {
	new_test_ext().execute_with(|| {
		let sender = 1;
		let handlers = vec![1, 2];
		let id = 0;
		let escrow = create_base_escrow(id, sender, handlers);
		assert_ok!(HmToken::transfer(Origin::signed(1), escrow.escrow_address, 100));
		assert_ok!(Escrow::cancel(Origin::signed(1), id));
		assert_eq!(Escrow::escrow(id).unwrap().status, EscrowStatus::Cancelled);
	});
}

#[test]
fn cancel_negative_tests() {
	new_test_ext().execute_with(|| {
		let sender = 1;
		let handlers = vec![1, 2];
		let escrow = create_base_escrow(0, sender, handlers);
		assert_noop!(Escrow::cancel(Origin::signed(8), 0), Error::<Test>::NonTrustedAccount);
		assert_noop!(Escrow::cancel(Origin::signed(1), 2), Error::<Test>::MissingEscrow);
		assert_noop!(Escrow::cancel(Origin::signed(1), 0), Error::<Test>::OutOfFunds);
		set_status(0, EscrowStatus::Complete).expect("setting status should work");
		assert_noop!(Escrow::cancel(Origin::signed(1), 0), Error::<Test>::AlreadyComplete);
		set_status(0, EscrowStatus::Paid).expect("setting status should work");
		assert_noop!(Escrow::cancel(Origin::signed(1), 0), Error::<Test>::AlreadyPaid);
	});
}

#[test]
fn complete_positive_tests() {
	new_test_ext().execute_with(|| {
		let sender = 1;
		let handlers = vec![1, 2];
		let escrow = create_base_escrow(0, sender, handlers);
		set_status(0, EscrowStatus::Paid).expect("setting status should work");
		assert_ok!(Escrow::complete(Origin::signed(1), 0));
		assert_eq!(Escrow::escrow(0).unwrap().status, EscrowStatus::Complete);
	});
}

#[test]
fn complete_negative_tests() {
	new_test_ext().execute_with(|| {
		let sender = 1;
		let handlers = vec![1, 2];
		let escrow = create_base_escrow(0, sender, handlers);
		assert_noop!(Escrow::complete(Origin::signed(8), 0), Error::<Test>::NonTrustedAccount);
		assert_noop!(Escrow::complete(Origin::signed(1), 2), Error::<Test>::MissingEscrow);
		assert_noop!(Escrow::complete(Origin::signed(1), 0), Error::<Test>::EscrowNotPaid);
	});
}

#[test]
fn store_results_positive_tests() {
	new_test_ext().execute_with(|| {
		let sender = 1;
		let handlers = vec![1, 2];
		let id = 0;
		let escrow = create_base_escrow(id, sender, handlers);
		let url = b"results.url".to_vec();
		let hash = b"0xdev".to_vec();
		assert_ok!(Escrow::store_results(Origin::signed(1), id, url.clone(), hash.clone()));
		assert_last_event::<Test>(RawEvent::<Test>::IntermediateStorage(id, url, hash).into());
	});
}

#[test]
fn store_results_negative_tests() {
	new_test_ext().execute_with(|| {
		let sender = 1;
		let handlers = vec![1, 2];
		let id = 0;
		let escrow = create_base_escrow(id, sender, handlers);
		let url = b"results.url".to_vec();
		let hash = b"0xdev".to_vec();
		assert_noop!(
			Escrow::store_results(Origin::signed(8), id, url.clone(), hash.clone()),
			Error::<Test>::NonTrustedAccount
		);
		assert_noop!(
			Escrow::store_results(Origin::signed(1), 2, url.clone(), hash.clone()),
			Error::<Test>::MissingEscrow
		);
		set_status(id, EscrowStatus::Cancelled).expect("setting status should work");
		assert_noop!(
			Escrow::store_results(Origin::signed(1), id, url.clone(), hash.clone()),
			Error::<Test>::EscrowClosed
		);
	});
}
