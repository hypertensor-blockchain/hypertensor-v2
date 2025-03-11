use super::mock::*;
use crate::tests::test_utils::*;
use crate::Event;
use frame_support::{
	assert_noop, assert_ok, assert_err
};
use log::info;
use frame_support::traits::{OnInitialize, Currency};
use sp_std::collections::{btree_map::BTreeMap, btree_set::BTreeSet};
use frame_support::BoundedVec;
use sp_core::OpaquePeerId as PeerId;
use crate::{
  Error,
  SubnetPaths, 
  TotalSubnetNodes,
  AccountSubnetDelegateStakeShares, 
  TotalSubnetDelegateStakeShares, 
  TotalSubnetDelegateStakeBalance,
  SubnetRemovalReason,
  StakeUnbondingLedger,
};

//
//
//
//
//
//
//
// Delegate staking
//
//
//
//
//
//
//

#[test]
fn test_delegate_math() {
  new_test_ext().execute_with(|| {
    let test1 = Network::convert_to_balance(
      1000000000000000000000,
      6000000000000000000000,
      6000000000000000000000
    );
    
    let test2 = Network::convert_to_balance(
      1000000000000000000000,
      6000000000000000000000,
      7000000000000000000000
    );

    assert_eq!(test1, 999999999000000000000);
    assert_eq!(test2, 1166666666000000000000);
  });
}

#[test]
fn test_delegate_math_with_storage() {
  new_test_ext().execute_with(|| {
    let subnet_path: Vec<u8> = "petals-team/StableBeluga2".into();
    let deposit_amount: u128 = 1000000000000000000000000;
    let amount: u128 = 1000000000000000000000;

    build_activated_subnet(subnet_path.clone(), 0, 0, deposit_amount, amount);

    let subnet_id = SubnetPaths::<Test>::get(subnet_path.clone()).unwrap();
    let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);

    let _ = Balances::deposit_creating(&account(total_subnet_nodes+1), amount + 500);
    let starting_delegator_balance = Balances::free_balance(&account(total_subnet_nodes+1));

    assert_ok!(
      Network::add_to_delegate_stake(
        RuntimeOrigin::signed(account(total_subnet_nodes+1)),
        subnet_id,
        amount,
      ) 
    );

    let post_delegator_balance = Balances::free_balance(&account(total_subnet_nodes+1));
    assert_eq!(post_delegator_balance, starting_delegator_balance - amount);

    let total_subnet_delegated_stake_shares = TotalSubnetDelegateStakeShares::<Test>::get(subnet_id);
    let total_subnet_delegated_stake_balance = TotalSubnetDelegateStakeBalance::<Test>::get(subnet_id);
    let delegate_shares = AccountSubnetDelegateStakeShares::<Test>::get(account(total_subnet_nodes+1), subnet_id);
    let delegate_balance = Network::convert_to_balance(
      delegate_shares,
      total_subnet_delegated_stake_shares,
      total_subnet_delegated_stake_balance
    );

    // assert_eq!(amount, delegate_balance);
    assert!(
      (delegate_balance >= Network::percent_mul(amount, 9999)) &&
      (delegate_balance <= amount)
    );

    let delegate_shares = AccountSubnetDelegateStakeShares::<Test>::get(account(total_subnet_nodes+1), subnet_id);

    assert_ok!(
      Network::remove_delegate_stake(
        RuntimeOrigin::signed(account(total_subnet_nodes+1)),
        subnet_id,
        delegate_shares / 2,
      )
    );

    let total_subnet_delegated_stake_shares = TotalSubnetDelegateStakeShares::<Test>::get(subnet_id);
    let total_subnet_delegated_stake_balance = TotalSubnetDelegateStakeBalance::<Test>::get(subnet_id);
    let delegate_shares = AccountSubnetDelegateStakeShares::<Test>::get(account(total_subnet_nodes+1), subnet_id);
    let delegate_balance = Network::convert_to_balance(
      delegate_shares,
      total_subnet_delegated_stake_shares,
      total_subnet_delegated_stake_balance
    );

  });
}

#[test]
fn test_remove_claim_delegate_stake_after_remove_subnet() {
  new_test_ext().execute_with(|| {
    let subnet_path: Vec<u8> = "petals-team/StableBeluga2".into();
    let deposit_amount: u128 = 1000000000000000000000000;
    let amount: u128 = 1000000000000000000000;

    build_activated_subnet(subnet_path.clone(), 0, 0, deposit_amount, amount);

    let subnet_id = SubnetPaths::<Test>::get(subnet_path.clone()).unwrap();
    let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);

    let _ = Balances::deposit_creating(&account(total_subnet_nodes+1), amount + 500);
    let starting_delegator_balance = Balances::free_balance(&account(total_subnet_nodes+1));

    assert_ok!(
      Network::add_to_delegate_stake(
        RuntimeOrigin::signed(account(total_subnet_nodes+1)),
        subnet_id,
        amount,
      ) 
    );

    let post_delegator_balance = Balances::free_balance(&account(total_subnet_nodes+1));
    assert_eq!(post_delegator_balance, starting_delegator_balance - amount);

    let total_subnet_delegated_stake_shares = TotalSubnetDelegateStakeShares::<Test>::get(subnet_id);
    let total_subnet_delegated_stake_balance = TotalSubnetDelegateStakeBalance::<Test>::get(subnet_id);
    let delegate_shares = AccountSubnetDelegateStakeShares::<Test>::get(account(total_subnet_nodes+1), subnet_id);
    let delegate_balance = Network::convert_to_balance(
      delegate_shares,
      total_subnet_delegated_stake_shares,
      total_subnet_delegated_stake_balance
    );

    // assert_eq!(amount, delegate_balance);
    assert!(
      (delegate_balance >= Network::percent_mul(amount, 9999)) &&
      (delegate_balance <= amount)
    );

    assert_ok!(
      Network::deactivate_subnet(
        subnet_path.clone().into(),
        SubnetRemovalReason::MinSubnetDelegateStake,
      )
    );

    assert_ok!(
      Network::remove_delegate_stake(
        RuntimeOrigin::signed(account(total_subnet_nodes+1)),
        subnet_id,
        delegate_shares,
      )
    );

    System::set_block_number(System::block_number() + ((EpochLength::get()  + 1) * DelegateStakeCooldownEpochs::get()));

    assert_ok!(
      Network::claim_unbondings(
        RuntimeOrigin::signed(account(total_subnet_nodes+1)),
      )
    );

    let post_balance = Balances::free_balance(&account(total_subnet_nodes+1));

    assert!(
      (post_balance >= Network::percent_mul(starting_delegator_balance, 9999)) &&
      (post_balance <= starting_delegator_balance)
    );

    let unbondings: BTreeMap<u64, u128> = StakeUnbondingLedger::<Test>::get(account(total_subnet_nodes+1));
    assert_eq!(unbondings.len(), 0);
  });
}

#[test]
fn test_add_to_delegate_stake() {
  new_test_ext().execute_with(|| {
    let subnet_path: Vec<u8> = "petals-team/StableBeluga2".into();
    let deposit_amount: u128 = 10000000000000000000000;
    let amount: u128 = 1000000000000000000000;

    build_activated_subnet(subnet_path.clone(), 0, 0, deposit_amount, amount);

    let subnet_id = SubnetPaths::<Test>::get(subnet_path.clone()).unwrap();
    let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);

    let n_account = total_subnet_nodes + 1;

    let _ = Balances::deposit_creating(&account(n_account), deposit_amount);

    let subnet_id = SubnetPaths::<Test>::get(subnet_path.clone()).unwrap();

    let total_subnet_delegated_stake_shares = TotalSubnetDelegateStakeShares::<Test>::get(subnet_id);
    let total_subnet_delegated_stake_balance = TotalSubnetDelegateStakeBalance::<Test>::get(subnet_id);

    let mut delegate_stake_to_be_added_as_shares = Network::convert_to_shares(
      amount,
      total_subnet_delegated_stake_shares,
      total_subnet_delegated_stake_balance
    );

    if total_subnet_delegated_stake_shares == 0 {
      delegate_stake_to_be_added_as_shares = delegate_stake_to_be_added_as_shares.saturating_sub(1000);
    }

    System::set_block_number(System::block_number() + DelegateStakeCooldownEpochs::get() * EpochLength::get());

    let starting_total_subnet_delegated_stake_balance = TotalSubnetDelegateStakeBalance::<Test>::get(subnet_id);

    let starting_delegator_balance = Balances::free_balance(&account(n_account));

    assert_ok!(
      Network::add_to_delegate_stake(
        RuntimeOrigin::signed(account(n_account)),
        subnet_id,
        amount,
      ) 
    );

    let post_delegator_balance = Balances::free_balance(&account(n_account));
    assert_eq!(post_delegator_balance, starting_delegator_balance - amount);

    let delegate_shares = AccountSubnetDelegateStakeShares::<Test>::get(account(n_account), subnet_id);
    assert_eq!(delegate_shares, delegate_stake_to_be_added_as_shares);
    assert_ne!(delegate_shares, 0);

    let total_subnet_delegated_stake_shares = TotalSubnetDelegateStakeShares::<Test>::get(subnet_id);
    let total_subnet_delegated_stake_balance = TotalSubnetDelegateStakeBalance::<Test>::get(subnet_id);

    assert_eq!(amount + starting_total_subnet_delegated_stake_balance, total_subnet_delegated_stake_balance);

    let delegate_balance = Network::convert_to_balance(
      delegate_shares,
      total_subnet_delegated_stake_shares,
      total_subnet_delegated_stake_balance
    );
    // The first depositor will lose a percentage of their deposit depending on the size
    // https://docs.openzeppelin.com/contracts/4.x/erc4626#inflation-attack
    // assert_eq!(delegate_balance, delegate_stake_to_be_added_as_shares);

    assert!(
      (delegate_balance >= Network::percent_mul(amount, 9999)) &&
      (delegate_balance <= amount)
    );
  });
}

#[test]
fn test_add_to_delegate_stake_increase_pool_check_balance() {
  new_test_ext().execute_with(|| {
    let subnet_path: Vec<u8> = "petals-team/StableBeluga2".into();
    let deposit_amount: u128 = 10000000000000000000000;
    let amount: u128 = 1000000000000000000000;

    build_activated_subnet(subnet_path.clone(), 0, 0, deposit_amount, amount);

    let subnet_id = SubnetPaths::<Test>::get(subnet_path.clone()).unwrap();
    let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);

    let n_account = total_subnet_nodes + 1;

    let _ = Balances::deposit_creating(&account(n_account), deposit_amount);

    let subnet_id = SubnetPaths::<Test>::get(subnet_path.clone()).unwrap();

    let total_subnet_delegated_stake_shares = TotalSubnetDelegateStakeShares::<Test>::get(subnet_id);
    let total_subnet_delegated_stake_balance = TotalSubnetDelegateStakeBalance::<Test>::get(subnet_id);

    let mut delegate_stake_to_be_added_as_shares = Network::convert_to_shares(
      amount,
      total_subnet_delegated_stake_shares,
      total_subnet_delegated_stake_balance
    );

    if total_subnet_delegated_stake_shares == 0 {
      delegate_stake_to_be_added_as_shares = delegate_stake_to_be_added_as_shares.saturating_sub(1000);
    }

    System::set_block_number(System::block_number() + DelegateStakeCooldownEpochs::get() * EpochLength::get());

    assert_ok!(
      Network::add_to_delegate_stake(
        RuntimeOrigin::signed(account(n_account)),
        subnet_id,
        amount,
      ) 
    );

    let delegate_shares = AccountSubnetDelegateStakeShares::<Test>::get(account(n_account), subnet_id);
    assert_eq!(delegate_shares, delegate_stake_to_be_added_as_shares);
    assert_ne!(delegate_shares, 0);

    let total_subnet_delegated_stake_shares = TotalSubnetDelegateStakeShares::<Test>::get(subnet_id);
    let total_subnet_delegated_stake_balance = TotalSubnetDelegateStakeBalance::<Test>::get(subnet_id);

    let delegate_balance = Network::convert_to_balance(
      delegate_shares,
      total_subnet_delegated_stake_shares,
      total_subnet_delegated_stake_balance
    );
    // The first depositor will lose a percentage of their deposit depending on the size
    // https://docs.openzeppelin.com/contracts/4.x/erc4626#inflation-attack
    // assert_eq!(delegate_balance, delegate_stake_to_be_added_as_shares);
    assert!(
      (delegate_balance >= Network::percent_mul(amount, 9999)) &&
      (delegate_balance <= amount)
    );

    let increase_delegate_stake_amount: u128 = 1000000000000000000000;
    Network::do_increase_delegate_stake(
      subnet_id,
      increase_delegate_stake_amount,
    );

    // ensure balance has increase
    let total_subnet_delegated_stake_shares = TotalSubnetDelegateStakeShares::<Test>::get(subnet_id);
    let total_subnet_delegated_stake_balance = TotalSubnetDelegateStakeBalance::<Test>::get(subnet_id);
    
    let post_delegate_balance = Network::convert_to_balance(
      delegate_shares,
      total_subnet_delegated_stake_shares,
      total_subnet_delegated_stake_balance
    );
    assert!(delegate_balance < post_delegate_balance);
    assert_ne!(delegate_balance, post_delegate_balance);
    assert!(
      (post_delegate_balance >= Network::percent_mul(amount + increase_delegate_stake_amount, 9999)) &&
      (post_delegate_balance <= amount + increase_delegate_stake_amount)
    );
  });
}

#[test]
fn test_claim_removal_of_delegate_stake() {
  new_test_ext().execute_with(|| {
    let subnet_path: Vec<u8> = "petals-team/StableBeluga2".into();
    let deposit_amount: u128 = 10000000000000000000000;
    let amount: u128 = 1000000000000000000000;

    build_activated_subnet(subnet_path.clone(), 0, 0, deposit_amount, amount);

    let subnet_id = SubnetPaths::<Test>::get(subnet_path.clone()).unwrap();
    let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);

    let n_account = total_subnet_nodes + 1;

    let _ = Balances::deposit_creating(&account(n_account), deposit_amount);

    let subnet_id = SubnetPaths::<Test>::get(subnet_path.clone()).unwrap();

    let total_subnet_delegated_stake_shares = TotalSubnetDelegateStakeShares::<Test>::get(subnet_id);
    let total_subnet_delegated_stake_balance = TotalSubnetDelegateStakeBalance::<Test>::get(subnet_id);

    let mut delegate_stake_to_be_added_as_shares = Network::convert_to_shares(
      amount,
      total_subnet_delegated_stake_shares,
      total_subnet_delegated_stake_balance
    );

    if total_subnet_delegated_stake_shares == 0 {
      delegate_stake_to_be_added_as_shares = delegate_stake_to_be_added_as_shares.saturating_sub(1000);
    }

    let starting_delegator_balance = Balances::free_balance(&account(n_account));

    assert_ok!(
      Network::add_to_delegate_stake(
        RuntimeOrigin::signed(account(n_account)),
        subnet_id,
        amount,
      ) 
    );

    let delegate_shares = AccountSubnetDelegateStakeShares::<Test>::get(account(n_account), subnet_id);
    assert_eq!(delegate_shares, delegate_stake_to_be_added_as_shares);
    assert_ne!(delegate_shares, 0);

    let total_subnet_delegated_stake_shares = TotalSubnetDelegateStakeShares::<Test>::get(subnet_id);
    let total_subnet_delegated_stake_balance = TotalSubnetDelegateStakeBalance::<Test>::get(subnet_id);

    let mut delegate_balance = Network::convert_to_balance(
      delegate_shares,
      total_subnet_delegated_stake_shares,
      total_subnet_delegated_stake_balance
    );
    // The first depositor will lose a percentage of their deposit depending on the size
    // https://docs.openzeppelin.com/contracts/4.x/erc4626#inflation-attack
    // assert_eq!(delegate_balance, delegate_stake_to_be_added_as_shares);
    assert!(
      (delegate_balance >= Network::percent_mul(amount, 9999)) &&
      (delegate_balance <= amount)
    );

    let epoch_length = EpochLength::get();
    let cooldown_epochs = DelegateStakeCooldownEpochs::get();

    System::set_block_number(System::block_number() + epoch_length * cooldown_epochs);

    let balance = Balances::free_balance(&account(n_account));
    let epoch = System::block_number() / epoch_length;

    assert_ok!(
      Network::remove_delegate_stake(
        RuntimeOrigin::signed(account(n_account)),
        subnet_id,
        delegate_shares,
      )
    );
    let post_balance = Balances::free_balance(&account(n_account));
    assert_eq!(post_balance, balance);

    let unbondings: BTreeMap<u64, u128> = StakeUnbondingLedger::<Test>::get(account(n_account));
    assert_eq!(unbondings.len(), 1);
    let (ledger_epoch, ledger_balance) = unbondings.iter().next().unwrap();
    assert_eq!(ledger_epoch, &epoch);
    assert!(*ledger_balance <= delegate_balance);

    assert_err!(
      Network::claim_unbondings(
        RuntimeOrigin::signed(account(n_account)),
      ),
      Error::<Test>::NoStakeUnbondingsOrCooldownNotMet
    );

    System::set_block_number(System::block_number() + ((epoch_length  + 1) * cooldown_epochs));

    let pre_claim_balance = Balances::free_balance(&account(n_account));

    assert_ok!(
      Network::claim_unbondings(
        RuntimeOrigin::signed(account(n_account)),
      )
    );

    let after_claim_balance = Balances::free_balance(&account(n_account));

    assert_eq!(after_claim_balance, pre_claim_balance + *ledger_balance);

    assert!(
      (post_balance >= Network::percent_mul(starting_delegator_balance, 9999)) &&
      (post_balance <= starting_delegator_balance)
    );

    let unbondings: BTreeMap<u64, u128> = StakeUnbondingLedger::<Test>::get(account(n_account));
    assert_eq!(unbondings.len(), 0);
  });
}

// #[test]
// fn test_remove_to_delegate_stake_max_unlockings_per_epoch_err() {
//   new_test_ext().execute_with(|| {
//     let subnet_path: Vec<u8> = "petals-team/StableBeluga2".into();
//     let deposit_amount: u128 = 10000000000000000000000;
//     let amount: u128 = 1000000000000000000000;

//     build_activated_subnet(subnet_path.clone(), 0, 0, deposit_amount, amount);

//     let subnet_id = SubnetPaths::<Test>::get(subnet_path.clone()).unwrap();
//     let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);

//     let n_account = total_subnet_nodes + 1;

//     let _ = Balances::deposit_creating(&account(n_account), deposit_amount);

//     let subnet_id = SubnetPaths::<Test>::get(subnet_path.clone()).unwrap();

//     let total_subnet_delegated_stake_shares = TotalSubnetDelegateStakeShares::<Test>::get(subnet_id);
//     let total_subnet_delegated_stake_balance = TotalSubnetDelegateStakeBalance::<Test>::get(subnet_id);

//     let mut delegate_stake_to_be_added_as_shares = Network::convert_to_shares(
//       amount,
//       total_subnet_delegated_stake_shares,
//       total_subnet_delegated_stake_balance
//     );

//     if total_subnet_delegated_stake_shares == 0 {
//       delegate_stake_to_be_added_as_shares = delegate_stake_to_be_added_as_shares.saturating_sub(1000);
//     }

//     System::set_block_number(System::block_number() + DelegateStakeCooldownEpochs::get() * EpochLength::get());

//     let starting_delegator_balance = Balances::free_balance(&account(n_account));

//     assert_ok!(
//       Network::add_to_delegate_stake(
//         RuntimeOrigin::signed(account(n_account)),
//         subnet_id,
//         amount,
//       ) 
//     );

//     let delegate_shares = AccountSubnetDelegateStakeShares::<Test>::get(account(n_account), subnet_id);

//     assert_ok!(
//       Network::remove_delegate_stake(
//         RuntimeOrigin::signed(account(n_account)),
//         subnet_id,
//         delegate_shares/2,
//       )
//     );
//     let unbondings: BTreeMap<u64, u128> = StakeUnbondingLedger::<Test>::get(account(n_account));
//     assert_eq!(unbondings.len(), 1);

//     assert_err!(
//       Network::remove_delegate_stake(
//         RuntimeOrigin::signed(account(n_account)),
//         subnet_id,
//         delegate_shares/2,
//       ),
//       Error::<Test>::MaxUnlockingsPerEpochReached
//     );
//   });
// }

#[test]
fn test_remove_to_delegate_stake_max_unlockings_reached_err() {
  new_test_ext().execute_with(|| {
    let subnet_path: Vec<u8> = "petals-team/StableBeluga2".into();
    let deposit_amount: u128 = 10000000000000000000000;
    let amount: u128 = 1000000000000000000000;

    build_activated_subnet(subnet_path.clone(), 0, 0, deposit_amount, amount);

    let subnet_id = SubnetPaths::<Test>::get(subnet_path.clone()).unwrap();
    let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);

    let n_account = total_subnet_nodes + 1;

    let _ = Balances::deposit_creating(&account(n_account), deposit_amount);

    let subnet_id = SubnetPaths::<Test>::get(subnet_path.clone()).unwrap();

    let total_subnet_delegated_stake_shares = TotalSubnetDelegateStakeShares::<Test>::get(subnet_id);
    let total_subnet_delegated_stake_balance = TotalSubnetDelegateStakeBalance::<Test>::get(subnet_id);

    let mut delegate_stake_to_be_added_as_shares = Network::convert_to_shares(
      amount,
      total_subnet_delegated_stake_shares,
      total_subnet_delegated_stake_balance
    );

    if total_subnet_delegated_stake_shares == 0 {
      delegate_stake_to_be_added_as_shares = delegate_stake_to_be_added_as_shares.saturating_sub(1000);
    }

    System::set_block_number(System::block_number() + DelegateStakeCooldownEpochs::get() * EpochLength::get());

    let starting_delegator_balance = Balances::free_balance(&account(n_account));

    assert_ok!(
      Network::add_to_delegate_stake(
        RuntimeOrigin::signed(account(n_account)),
        subnet_id,
        amount,
      ) 
    );

    let max_unlockings = MaxDelegateStakeUnlockings::get();
    for n in 0..max_unlockings+1 {
      System::set_block_number(System::block_number() + EpochLength::get() + 1);
      if n+1 > max_unlockings {
        assert_err!(
          Network::remove_delegate_stake(
            RuntimeOrigin::signed(account(n_account)),
            subnet_id,
            1000,
          ),
          Error::<Test>::MaxUnlockingsReached
        );    
      } else {
        assert_ok!(
          Network::remove_delegate_stake(
            RuntimeOrigin::signed(account(n_account)),
            subnet_id,
            1000,
          )
        );
        let unbondings: BTreeMap<u64, u128> = StakeUnbondingLedger::<Test>::get(account(n_account));
        assert_eq!(unbondings.len() as u32, n+1);
      }
    }
  });
}

#[test]
fn test_switch_delegate_stake() {
  new_test_ext().execute_with(|| {
    let deposit_amount: u128 = 10000000000000000000000;
    let amount: u128 = 1000000000000000000000;

    let from_subnet_path: Vec<u8> = "petals-team/StableBeluga2".into();
    build_activated_subnet(from_subnet_path.clone(), 0, 0, deposit_amount, amount);
    let from_subnet_id = SubnetPaths::<Test>::get(from_subnet_path.clone()).unwrap();

    let to_subnet_path: Vec<u8> = "petals-team/StableBeluga3".into();
    build_activated_subnet(to_subnet_path.clone(), 0, 0, deposit_amount, amount);
    let to_subnet_id = SubnetPaths::<Test>::get(to_subnet_path.clone()).unwrap();

    let n_account = 255;

    let _ = Balances::deposit_creating(&account(n_account), deposit_amount);

    let total_subnet_delegated_stake_shares = TotalSubnetDelegateStakeShares::<Test>::get(from_subnet_id);
    let total_subnet_delegated_stake_balance = TotalSubnetDelegateStakeBalance::<Test>::get(from_subnet_id);

    let mut delegate_stake_to_be_added_as_shares = Network::convert_to_shares(
      amount,
      total_subnet_delegated_stake_shares,
      total_subnet_delegated_stake_balance
    );

    if total_subnet_delegated_stake_shares == 0 {
      delegate_stake_to_be_added_as_shares = delegate_stake_to_be_added_as_shares.saturating_sub(1000);
    }

    System::set_block_number(System::block_number() + DelegateStakeCooldownEpochs::get() * EpochLength::get());

    let starting_delegator_balance = Balances::free_balance(&account(n_account));

    assert_ok!(
      Network::add_to_delegate_stake(
        RuntimeOrigin::signed(account(n_account)),
        from_subnet_id,
        amount,
      ) 
    );

    let delegate_shares = AccountSubnetDelegateStakeShares::<Test>::get(account(n_account), from_subnet_id);
    assert_eq!(delegate_shares, delegate_stake_to_be_added_as_shares);
    assert_ne!(delegate_shares, 0);

    let total_subnet_delegated_stake_shares = TotalSubnetDelegateStakeShares::<Test>::get(from_subnet_id);
    let total_subnet_delegated_stake_balance = TotalSubnetDelegateStakeBalance::<Test>::get(from_subnet_id);

    let mut from_delegate_balance = Network::convert_to_balance(
      delegate_shares,
      total_subnet_delegated_stake_shares,
      total_subnet_delegated_stake_balance
    );
    // The first depositor will lose a percentage of their deposit depending on the size
    // https://docs.openzeppelin.com/contracts/4.x/erc4626#inflation-attack
    // assert_eq!(from_delegate_balance, delegate_stake_to_be_added_as_shares);

    assert_ok!(
      Network::transfer_delegate_stake(
        RuntimeOrigin::signed(account(n_account)),
        from_subnet_id,
        to_subnet_id,
        delegate_shares,
      ) 
    );
    let from_delegate_shares = AccountSubnetDelegateStakeShares::<Test>::get(account(n_account), from_subnet_id);
    assert_eq!(from_delegate_shares, 0);

    let to_delegate_shares = AccountSubnetDelegateStakeShares::<Test>::get(account(n_account), to_subnet_id);
    // assert_eq!(to_delegate_shares, delegate_stake_to_be_added_as_shares);
    assert_ne!(to_delegate_shares, 0);

    let total_subnet_delegated_stake_shares = TotalSubnetDelegateStakeShares::<Test>::get(to_subnet_id);
    let total_subnet_delegated_stake_balance = TotalSubnetDelegateStakeBalance::<Test>::get(to_subnet_id);

    let mut to_delegate_balance = Network::convert_to_balance(
      to_delegate_shares,
      total_subnet_delegated_stake_shares,
      total_subnet_delegated_stake_balance
    );
    // The first depositor will lose a percentage of their deposit depending on the size
    // https://docs.openzeppelin.com/contracts/4.x/erc4626#inflation-attack
    // Will lose about .01% of the transfer value on first transfer into a pool
    // The balance should be about ~99% of the ``from`` subnet to the ``to`` subnet
    assert!(
      (to_delegate_balance >= Network::percent_mul(from_delegate_balance, 9999)) &&
      (to_delegate_balance <= from_delegate_balance)
    );
  });
}

#[test]
fn test_switch_delegate_stake_not_enough_stake_err() {
  new_test_ext().execute_with(|| {
    let deposit_amount: u128 = 10000000000000000000000;
    let amount: u128 = 1000000000000000000000;

    let from_subnet_path: Vec<u8> = "petals-team/StableBeluga2".into();
    build_activated_subnet(from_subnet_path.clone(), 0, 0, deposit_amount, amount);
    let from_subnet_id = SubnetPaths::<Test>::get(from_subnet_path.clone()).unwrap();

    let to_subnet_path: Vec<u8> = "petals-team/StableBeluga3".into();
    build_activated_subnet(to_subnet_path.clone(), 0, 0, deposit_amount, amount);
    let to_subnet_id = SubnetPaths::<Test>::get(to_subnet_path.clone()).unwrap();

    let n_account = 255;

    let _ = Balances::deposit_creating(&account(n_account), deposit_amount);

    assert_err!(
      Network::transfer_delegate_stake(
        RuntimeOrigin::signed(account(n_account)),
        from_subnet_id,
        to_subnet_id,
        0,
      ),
      Error::<Test>::NotEnoughStakeToWithdraw
    );

    assert_err!(
      Network::transfer_delegate_stake(
        RuntimeOrigin::signed(account(n_account)),
        from_subnet_id,
        to_subnet_id,
        1000,
      ),
      Error::<Test>::NotEnoughStakeToWithdraw
    );
  });
}

// #[test]
// fn test_remove_to_delegate_stake_epochs_not_met_err() {
//   new_test_ext().execute_with(|| {
//     let subnet_path: Vec<u8> = "petals-team/StableBeluga2".into();

//     build_subnet(subnet_path.clone());
//     let deposit_amount: u128 = 10000000000000000000000;
//     let amount: u128 = 1000000000000000000000;
//     let _ = Balances::deposit_creating(&account(0), deposit_amount);

//     let subnet_id = SubnetPaths::<Test>::get(subnet_path.clone()).unwrap();

//     let total_subnet_delegated_stake_shares = TotalSubnetDelegateStakeShares::<Test>::get(subnet_id);
//     let total_subnet_delegated_stake_balance = TotalSubnetDelegateStakeBalance::<Test>::get(subnet_id);

//     let mut delegate_stake_to_be_added_as_shares = Network::convert_to_shares(
//       amount,
//       total_subnet_delegated_stake_shares,
//       total_subnet_delegated_stake_balance
//     );

//     if total_subnet_delegated_stake_shares == 0 {
//       delegate_stake_to_be_added_as_shares = delegate_stake_to_be_added_as_shares.saturating_sub(1000);
//     }

//     System::set_block_number(System::block_number() + DelegateStakeCooldownEpochs::get() * EpochLength::get());

//     assert_ok!(
//       Network::add_to_delegate_stake(
//         RuntimeOrigin::signed(account(0)),
//         subnet_id,
//         amount,
//       ) 
//     );

//     let delegate_shares = AccountSubnetDelegateStakeShares::<Test>::get(account(0), subnet_id);
//     assert_eq!(delegate_shares, delegate_stake_to_be_added_as_shares);
//     assert_ne!(delegate_shares, 0);

//     let total_subnet_delegated_stake_shares = TotalSubnetDelegateStakeShares::<Test>::get(subnet_id);
//     let total_subnet_delegated_stake_balance = TotalSubnetDelegateStakeBalance::<Test>::get(subnet_id);

//     let mut delegate_balance = Network::convert_to_balance(
//       delegate_shares,
//       total_subnet_delegated_stake_shares,
//       total_subnet_delegated_stake_balance
//     );
//     // The first depositor will lose a percentage of their deposit depending on the size
//     // https://docs.openzeppelin.com/contracts/4.x/erc4626#inflation-attack
//     assert_eq!(delegate_balance, delegate_stake_to_be_added_as_shares);
//     assert!(
//       (delegate_balance >= Network::percent_mul(amount, 9999)) &&
//       (delegate_balance <= amount)
//     );

//     // assert_err!(
//     //   Network::remove_delegate_stake(
//     //     RuntimeOrigin::signed(account(0)),
//     //     subnet_id,
//     //     delegate_shares,
//     //   ),
//     //   Error::<Test>::InsufficientCooldown
//     // );
//   });
// }

#[test]
fn test_remove_delegate_stake_after_subnet_remove() {
  new_test_ext().execute_with(|| {
    let subnet_path: Vec<u8> = "petals-team/StableBeluga2".into();
    let deposit_amount: u128 = 10000000000000000000000;
    let amount: u128 = 1000000000000000000000;

    build_activated_subnet(subnet_path.clone(), 0, 0, deposit_amount, amount);

    let subnet_id = SubnetPaths::<Test>::get(subnet_path.clone()).unwrap();
    let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);

    let n_account = total_subnet_nodes + 1;

    let _ = Balances::deposit_creating(&account(n_account), deposit_amount);

    let subnet_id = SubnetPaths::<Test>::get(subnet_path.clone()).unwrap();

    let total_subnet_delegated_stake_shares = TotalSubnetDelegateStakeShares::<Test>::get(subnet_id);
    let total_subnet_delegated_stake_balance = TotalSubnetDelegateStakeBalance::<Test>::get(subnet_id);

    let mut delegate_stake_to_be_added_as_shares = Network::convert_to_shares(
      amount,
      total_subnet_delegated_stake_shares,
      total_subnet_delegated_stake_balance
    );

    if total_subnet_delegated_stake_shares == 0 {
      delegate_stake_to_be_added_as_shares = delegate_stake_to_be_added_as_shares.saturating_sub(1000);
    }

    System::set_block_number(System::block_number() + DelegateStakeCooldownEpochs::get() * EpochLength::get());

    let starting_delegator_balance = Balances::free_balance(&account(n_account));

    assert_ok!(
      Network::add_to_delegate_stake(
        RuntimeOrigin::signed(account(n_account)),
        subnet_id,
        amount,
      ) 
    );

    let delegate_shares = AccountSubnetDelegateStakeShares::<Test>::get(account(n_account), subnet_id);
    assert_eq!(delegate_shares, delegate_stake_to_be_added_as_shares);
    assert_ne!(delegate_shares, 0);

    let total_subnet_delegated_stake_shares = TotalSubnetDelegateStakeShares::<Test>::get(subnet_id);
    let total_subnet_delegated_stake_balance = TotalSubnetDelegateStakeBalance::<Test>::get(subnet_id);

    let mut delegate_balance = Network::convert_to_balance(
      delegate_shares,
      total_subnet_delegated_stake_shares,
      total_subnet_delegated_stake_balance
    );
    // The first depositor will lose a percentage of their deposit depending on the size
    // https://docs.openzeppelin.com/contracts/4.x/erc4626#inflation-attack
    // assert_eq!(delegate_balance, delegate_stake_to_be_added_as_shares);
    assert!(
      (delegate_balance >= Network::percent_mul(amount, 9999)) &&
      (delegate_balance <= amount)
    );

    let epoch_length = EpochLength::get();
    let cooldown_epochs = DelegateStakeCooldownEpochs::get();

    assert_ok!(
      Network::deactivate_subnet( 
        subnet_path.clone().into(),
        SubnetRemovalReason::MinSubnetDelegateStake,
      )
    );

    // System::set_block_number(System::block_number() + epoch_length * cooldown_epochs);

    let balance = Balances::free_balance(&account(n_account));
    let epoch = System::block_number() / epoch_length;

    assert_ok!(
      Network::remove_delegate_stake(
        RuntimeOrigin::signed(account(n_account)),
        subnet_id,
        delegate_shares,
      )
    );
    let post_balance = Balances::free_balance(&account(n_account));
    assert_eq!(post_balance, balance);

    let unbondings: BTreeMap<u64, u128> = StakeUnbondingLedger::<Test>::get(account(n_account));
    assert_eq!(unbondings.len(), 1);
    let (ledger_epoch, ledger_balance) = unbondings.iter().next().unwrap();
    assert_eq!(ledger_epoch, &epoch);
    assert!(*ledger_balance <= delegate_balance);


    assert_err!(
      Network::claim_unbondings(
        RuntimeOrigin::signed(account(n_account)),
      ),
      Error::<Test>::NoStakeUnbondingsOrCooldownNotMet
    );

    System::set_block_number(System::block_number() + ((epoch_length  + 1) * cooldown_epochs));

    assert_ok!(
      Network::claim_unbondings(
        RuntimeOrigin::signed(account(n_account)),
      )
    );

    let post_balance = Balances::free_balance(&account(n_account));

    assert!(
      (post_balance >= Network::percent_mul(starting_delegator_balance, 9999)) &&
      (post_balance <= starting_delegator_balance)
    );

    let unbondings: BTreeMap<u64, u128> = StakeUnbondingLedger::<Test>::get(account(n_account));
    assert_eq!(unbondings.len(), 0);
  });
}