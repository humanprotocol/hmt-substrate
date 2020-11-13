use crate::{mock::*, Error, RawEvent};
use frame_support::{assert_noop, assert_ok};

fn last_event() -> TestEvent {
    frame_system::Module::<Test>::events()
        .pop()
        .expect("Event expected")
        .event
}

#[test]
fn config_values_are_initiated() {
    new_test_ext().execute_with(|| {
        assert_eq!(HMToken::total_supply(), 1_000);
        assert_eq!(HMToken::decimals(), 15);
        assert_eq!(HMToken::name(), b"Human Protocol Token".to_vec());
        assert_eq!(HMToken::symbol(), b"HMT".to_vec());
        assert_eq!(HMToken::balance(1), HMToken::total_supply());
    });
}

#[test]
fn transfer_passes() {
    new_test_ext().execute_with(|| {
        let amount_to_transfer: u128 = 10;
        let new_balance = HMToken::total_supply() - amount_to_transfer;
        let from = 1;
        let to = 2;
        assert_ok!(HMToken::transfer(
            Origin::signed(from),
            to,
            amount_to_transfer
        ));
        assert_eq!(HMToken::balance(from), new_balance);
        assert_eq!(HMToken::balance(to), amount_to_transfer);
        assert_eq!(
            last_event(),
            TestEvent::HMTokenPallet(RawEvent::Transferred(from, to, amount_to_transfer))
        );
    });
}

#[test]
fn transfer_fails() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            HMToken::transfer(Origin::signed(2), 3, 10),
            Error::<Test>::BalanceLow
        );
        assert_noop!(
            HMToken::transfer(Origin::signed(1), 2, 0),
            Error::<Test>::AmountZero
        );
    })
}

#[test]
fn bulk_transfer_works() {
    new_test_ext().execute_with(|| {
        let amount: u128 = 10;
        let new_balance = HMToken::total_supply() - amount * 2;
        let from = 1;
        let first_rec = 2;
        let second_rec = 3;
        let id = 42;
        assert_ok!(HMToken::transfer_bulk(
            Origin::signed(from),
            vec![first_rec, second_rec],
            vec![amount, amount],
            id
        ));
        assert_eq!(HMToken::balance(from), new_balance);
        assert_eq!(HMToken::balance(first_rec), amount);
        assert_eq!(HMToken::balance(second_rec), amount);
        assert_eq!(
            last_event(),
            TestEvent::HMTokenPallet(RawEvent::BulkTransfer(id, 2, 0))
        );
    });
}

#[test]
fn bulk_transfer_fails_and_passes() {
    new_test_ext().execute_with(|| {
        let amount: u128 = HMToken::total_supply() - 10;
        let new_balance = 0;
        let from = 1;
        let first_rec = 2;
        let second_rec = 3;
        let id = 42;
        // Transfer some funds away to make sure that one of the bulk transfers will fail.
        assert_ok!(HMToken::transfer(Origin::signed(from), 5, 10));
        assert_ok!(HMToken::transfer_bulk(
            Origin::signed(from),
            vec![first_rec, second_rec],
            vec![amount, 1],
            id
        ));
        assert_eq!(HMToken::balance(from), new_balance);
        assert_eq!(HMToken::balance(first_rec), amount);
        assert_eq!(HMToken::balance(second_rec), 0);

        assert_eq!(
            last_event(),
            TestEvent::HMTokenPallet(RawEvent::BulkTransfer(id, 1, 1))
        );
    });
}

#[test]
fn bulk_transfer_fails() {
    new_test_ext().execute_with(|| {
        let amount: u128 = 500;
        let new_balance = HMToken::total_supply() - amount * 2;
        let from = 1;
        let first_rec = 2;
        let second_rec = 3;
        let id = 42;
        assert_noop!(
            HMToken::transfer_bulk(
                Origin::signed(from),
                vec![first_rec],
                vec![amount, amount],
                id
            ),
            Error::<Test>::MismatchBulkTransfer
        );
        assert_noop!(
            HMToken::transfer_bulk(
                Origin::signed(from),
                vec![first_rec, second_rec],
                vec![amount],
                id
            ),
            Error::<Test>::MismatchBulkTransfer
        );

        assert_noop!(
            HMToken::transfer_bulk(
                Origin::signed(from),
                vec![first_rec; 11],
                vec![amount; 11],
                id
            ),
            Error::<Test>::TooManyTos
        );
        assert_noop!(
            HMToken::transfer_bulk(
                Origin::signed(from),
                vec![first_rec, second_rec],
                vec![amount, amount],
                id
            ),
            Error::<Test>::TransferTooBig
        );
    });
}
