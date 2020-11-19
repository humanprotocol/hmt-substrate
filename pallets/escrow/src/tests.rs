use crate::{
	mock::*,
	Error, EscrowId, EscrowInfo, EscrowStatus, generate_account_id,
};
use sp_runtime::Percent;
use frame_support::{assert_noop, assert_ok};

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
		EscrowBuilder {
			..Default::default()
		}
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
	assert_ok!(Escrow::create(Origin::signed(sender), i.canceller, handlers, i.manifest_url, i.manifest_hash, i.reputation_oracle, i.recording_oracle, i.reputation_oracle_stake, i.recording_oracle_stake));
	copy
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
		all_handlers.extend(vec![escrow.canceller, escrow.reputation_oracle, escrow.recording_oracle, sender]);
		for handler in all_handlers {
			assert!(Escrow::is_trusted_handler(0, handler));
		}

		create_base_escrow(1, sender, handlers);
		assert_eq!(Escrow::counter(), 2);
		assert_ne!(Escrow::escrow(0).unwrap().escrow_address, Escrow::escrow(1).unwrap().escrow_address);
	});
}

#[test]
fn abort_positive_tests() {
	new_test_ext().execute_with(|| {
		let sender = 1;
		let handlers = vec![1, 2];
		let escrow = create_base_escrow(0, sender, handlers);
		assert_ok!(HmToken::transfer(Origin::signed(1), escrow.escrow_address, 100));
		let balance_before = HmToken::balance(1);
		assert_ok!(Escrow::abort(Origin::signed(1), 0));
		let balance_after = HmToken::balance(1);

		assert_eq!(Escrow::escrow(0), None);
		assert_eq!((balance_after - balance_before), 100);

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

		//TODO add tests for escrow status complete and paid
	
	});
}

#[test]
fn cancel_positive_tests() {
	new_test_ext().execute_with(|| {
		let sender = 1;
		let handlers = vec![1, 2];
		let escrow = create_base_escrow(0, sender, handlers);
		assert_ok!(HmToken::transfer(Origin::signed(1), escrow.escrow_address, 100));
		assert_ok!(Escrow::cancel(Origin::signed(1), 0));
		assert_eq!(Escrow::escrow(0).unwrap().status, EscrowStatus::Cancelled);		
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

		//TODO add tests for escrow status complete and paid
		
	});
}	
