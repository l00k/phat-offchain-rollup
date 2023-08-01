use ink::prelude::vec::Vec;
use openbrush::contracts::access_control;
use openbrush::contracts::access_control::AccessControl;
use openbrush::test_utils::{accounts, change_caller};
use phat_rollup_anchor_ink::traits::message_queue::*;
use phat_rollup_anchor_ink::traits::rollup_anchor::*;
use scale::Encode;

mod contract;
use contract::test_contract::MyContract;

#[ink::test]
fn test_conditions() {
    let accounts = accounts();
    let mut contract = MyContract::new(accounts.alice);

    // no condition, no update, no action => it should work
    assert_eq!(contract.rollup_cond_eq(vec![], vec![], vec![]), Ok(true));

    // test with correct condition
    let conditions = vec![(123u8.encode(), None)];
    assert_eq!(
        contract.rollup_cond_eq(conditions, vec![], vec![]),
        Ok(true)
    );

    // update a value
    let updates = vec![(123u8.encode(), Some(456u128.encode()))];
    assert_eq!(contract.rollup_cond_eq(vec![], updates, vec![]), Ok(true));

    // test with the correct condition
    let conditions = vec![(123u8.encode(), Some(456u128.encode()))];
    assert_eq!(
        contract.rollup_cond_eq(conditions, vec![], vec![]),
        Ok(true)
    );

    // test with incorrect condition (incorrect value)
    let conditions = vec![(123u8.encode(), Some(789u128.encode()))];
    assert_eq!(
        contract.rollup_cond_eq(conditions, vec![], vec![]),
        Err(RollupAnchorError::ConditionNotMet)
    );

    // test with incorrect condition (incorrect value)
    let conditions = vec![(123u8.encode(), None)];
    assert_eq!(
        contract.rollup_cond_eq(conditions, vec![], vec![]),
        Err(RollupAnchorError::ConditionNotMet)
    );

    // test with incorrect condition (incorrect key)
    let conditions = vec![
        (123u8.encode(), Some(456u128.encode())),
        (124u8.encode(), Some(456u128.encode())),
    ];
    assert_eq!(
        contract.rollup_cond_eq(conditions, vec![], vec![]),
        Err(RollupAnchorError::ConditionNotMet)
    );
}

#[ink::test]
fn test_actions_missing_data() {
    let accounts = accounts();
    let mut contract = MyContract::new(accounts.alice);

    let actions = vec![HandleActionInput {
        action_type: ACTION_SET_QUEUE_HEAD,
        id: None, // missing data
        action: None,
        address: None,
    }];
    assert_eq!(
        contract.rollup_cond_eq(vec![], vec![], actions),
        Err(RollupAnchorError::MissingData)
    );

    let actions = vec![HandleActionInput {
        action_type: ACTION_REPLY,
        id: None,
        action: None, // missing data
        address: None,
    }];
    assert_eq!(
        contract.rollup_cond_eq(vec![], vec![], actions),
        Err(RollupAnchorError::MissingData)
    );
}

#[ink::test]
fn test_action_pop_to() {
    let accounts = accounts();
    let mut contract = MyContract::new(accounts.alice);

    // no condition, no update, no action
    let mut actions = Vec::new();
    actions.push(HandleActionInput {
        action_type: ACTION_SET_QUEUE_HEAD,
        id: Some(2),
        action: None,
        address: None,
    });

    assert_eq!(
        contract.rollup_cond_eq(vec![], vec![], actions.clone()),
        Err(RollupAnchorError::MessageQueueError(
            MessageQueueError::InvalidPopTarget
        ))
    );

    let message = 4589u16;
    contract.push_message(&message).unwrap();
    contract.push_message(&message).unwrap();

    assert_eq!(contract.rollup_cond_eq(vec![], vec![], actions), Ok(true));
}

#[ink::test]
fn test_action_reply() {
    let accounts = accounts();
    let mut contract = MyContract::new(accounts.alice);

    let actions = vec![HandleActionInput {
        action_type: ACTION_REPLY,
        id: Some(2),
        action: Some(012u8.encode()),
        address: None,
    }];

    assert_eq!(contract.rollup_cond_eq(vec![], vec![], actions), Ok(true));
}

#[ink::test]
fn test_grant_role() {
    let accounts = accounts();
    let mut contract = MyContract::new(accounts.alice);

    // bob cannot grant the role
    change_caller(accounts.bob);
    assert_eq!(
        Err(access_control::AccessControlError::MissingRole),
        contract.grant_role(ATTESTOR_ROLE, Some(accounts.bob))
    );

    // alice, the owner, can do it
    change_caller(accounts.alice);
    assert_eq!(
        Ok(()),
        contract.grant_role(ATTESTOR_ROLE, Some(accounts.bob))
    );
}

#[ink::test]
fn test_rollup_cond_eq_role_attestor() {
    let accounts = accounts();
    let mut contract = MyContract::new(accounts.alice);

    change_caller(accounts.bob);

    assert_eq!(
        Err(RollupAnchorError::AccessControlError(
            access_control::AccessControlError::MissingRole
        )),
        contract.rollup_cond_eq(vec![], vec![], vec![])
    );

    change_caller(accounts.alice);
    contract
        .grant_role(ATTESTOR_ROLE, Some(accounts.bob))
        .expect("Error when grant the role Attestor");

    change_caller(accounts.bob);
    assert_eq!(Ok(true), contract.rollup_cond_eq(vec![], vec![], vec![]));
}
