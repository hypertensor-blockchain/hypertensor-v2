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
  AccountNodeDelegateStakeShares,
  TotalNodeDelegateStakeBalance,
  TotalNodeDelegateStakeShares,
  MinStakeBalance,
};
use codec::Decode;
use sp_runtime::traits::TrailingZeroInput;
use sp_core::U256;

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
    let _ = env_logger::builder().is_test(true).try_init();

    let subnet_id = 0;
    let account_id = account(0);
    let delegate_stake_to_be_added = 1000e+18 as u128;

    let account_delegate_stake_shares: u128 = AccountSubnetDelegateStakeShares::<Test>::get(&account_id, subnet_id);
    // let total_subnet_delegated_stake_shares = TotalSubnetDelegateStakeShares::<Test>::get(subnet_id);
    let total_subnet_delegated_stake_shares = match TotalSubnetDelegateStakeShares::<Test>::get(subnet_id) {
      0 => {
        // --- Mitigate inflation attack
        // Network::increase_account_delegate_stake_shares(
        //   &<Test as frame_system::Config>::AccountId::decode(&mut TrailingZeroInput::zeroes()).unwrap(),
        //   subnet_id, 
        //   0,
        //   1000,
        // );
        TotalSubnetDelegateStakeShares::<Test>::mutate(subnet_id, |mut n| *n += 10000);
        0
      },
      shares => shares,
    };
    let total_subnet_delegated_stake_balance = TotalSubnetDelegateStakeBalance::<Test>::get(subnet_id);
    log::error!("total_subnet_delegated_stake_shares   {:?}", total_subnet_delegated_stake_shares);
    log::error!("total_subnet_delegated_stake_balance  {:?}", total_subnet_delegated_stake_balance);

    let mut delegate_stake_to_be_added_as_shares = Network::convert_to_shares(
      delegate_stake_to_be_added,
      total_subnet_delegated_stake_shares,
      total_subnet_delegated_stake_balance
    );
    log::error!("delegate_stake_to_be_added_as_shares  {:?}", delegate_stake_to_be_added_as_shares);

    Network::increase_account_delegate_stake_shares(
      &account_id,
      subnet_id, 
      delegate_stake_to_be_added,
      delegate_stake_to_be_added_as_shares,
    );

    let account_delegate_shares = AccountSubnetDelegateStakeShares::<Test>::get(&account_id, subnet_id);
    let total_subnet_delegated_stake_shares = TotalSubnetDelegateStakeShares::<Test>::get(subnet_id);
    let total_subnet_delegated_stake_balance = TotalSubnetDelegateStakeBalance::<Test>::get(subnet_id);
    
    log::error!(" ");
    log::error!("account_delegate_shares               {:?}", account_delegate_shares);
    log::error!("total_subnet_delegated_stake_shares   {:?}", total_subnet_delegated_stake_shares);
    log::error!("total_subnet_delegated_stake_balance  {:?}", total_subnet_delegated_stake_balance);

    let delegate_balance = Network::convert_to_balance(
      account_delegate_shares,
      total_subnet_delegated_stake_shares,
      total_subnet_delegated_stake_balance
    );
    
    log::error!("delegate_balance                      {:?}", delegate_balance);

    // Ensure balance is within <= 0.01% of deposited balance, and less than deposited balance
    assert!(
      (delegate_balance >= Network::percent_mul(delegate_stake_to_be_added, 990000000)) &&
      (delegate_balance < delegate_stake_to_be_added)
    );

    let delegate_balance2 = Network::convert_to_balance(
      account_delegate_shares,
      total_subnet_delegated_stake_shares + 9000,
      total_subnet_delegated_stake_balance
    );
    log::error!("delegate_balance2                     {:?}", delegate_balance2);
  });
}

#[test]
fn test_delegate_math_with_storage_deposit() {
  new_test_ext().execute_with(|| {
    let subnet_path: Vec<u8> = "petals-team/StableBeluga2".into();
    let deposit_amount: u128 = 1000000000000000000000000;
    let amount: u128 = 1000000000000000000000; // 1000
    let stake_amount: u128 = MinStakeBalance::<Test>::get();

    build_activated_subnet(subnet_path.clone(), 0, 0, deposit_amount, stake_amount);

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

    // ensure removes wallet balance
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

    // Ensure balance is within <= 0.01% of deposited balance, and less than deposited balance
    assert!(
      (delegate_balance >= Network::percent_mul(amount, 990000000)) &&
      (delegate_balance < amount)
    );

    let pre_balance = Balances::free_balance(&account(total_subnet_nodes+1));

    let delegate_shares = AccountSubnetDelegateStakeShares::<Test>::get(account(total_subnet_nodes+1), subnet_id);
    let shares_to_remove = delegate_shares / 2;
    let expected_ledger_balance = Network::convert_to_balance(
      shares_to_remove,
      total_subnet_delegated_stake_shares,
      total_subnet_delegated_stake_balance
    );

    let epoch = System::block_number() / EpochLength::get();

    assert_ok!(
      Network::remove_delegate_stake(
        RuntimeOrigin::signed(account(total_subnet_nodes+1)),
        subnet_id,
        shares_to_remove,
      )
    );

    let post_balance = Balances::free_balance(&account(total_subnet_nodes+1));
    assert_eq!(pre_balance, post_balance);

    let total_subnet_delegated_stake_shares = TotalSubnetDelegateStakeShares::<Test>::get(subnet_id);
    let total_subnet_delegated_stake_balance = TotalSubnetDelegateStakeBalance::<Test>::get(subnet_id);
    let delegate_shares = AccountSubnetDelegateStakeShares::<Test>::get(account(total_subnet_nodes+1), subnet_id);
    let delegate_balance = Network::convert_to_balance(
      delegate_shares,
      total_subnet_delegated_stake_shares,
      total_subnet_delegated_stake_balance
    );

    let unbondings: BTreeMap<u32, u128> = StakeUnbondingLedger::<Test>::get(account(total_subnet_nodes+1));
    assert_eq!(unbondings.len(), 1);
    let (ledger_epoch, ledger_balance) = unbondings.iter().next().unwrap();
    assert_eq!(*ledger_epoch, &epoch + DelegateStakeCooldownEpochs::get());
    assert_eq!(*ledger_balance, expected_ledger_balance);
  });
}

#[test]
fn test_remove_claim_delegate_stake_after_remove_subnet() {
  new_test_ext().execute_with(|| {
    let subnet_path: Vec<u8> = "petals-team/StableBeluga2".into();
    let deposit_amount: u128 = 1000000000000000000000000;
    let amount: u128 = 1000000000000000000000;
    let stake_amount: u128 = MinStakeBalance::<Test>::get();

    build_activated_subnet(subnet_path.clone(), 0, 0, deposit_amount, stake_amount);

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
    let expected_ledger_balance = Network::convert_to_balance(
      delegate_shares,
      total_subnet_delegated_stake_shares,
      total_subnet_delegated_stake_balance
    );
    // assert_eq!(amount, delegate_balance);
    assert!(
      (delegate_balance >= Network::percent_mul(amount, 990000000)) &&
      (delegate_balance < amount)
    );

    assert_ok!(
      Network::do_remove_subnet(
        subnet_path.clone().into(),
        SubnetRemovalReason::MinSubnetDelegateStake,
      )
    );

    let epoch = System::block_number() / EpochLength::get();

    assert_ok!(
      Network::remove_delegate_stake(
        RuntimeOrigin::signed(account(total_subnet_nodes+1)),
        subnet_id,
        delegate_shares,
      )
    );

    let unbondings: BTreeMap<u32, u128> = StakeUnbondingLedger::<Test>::get(account(total_subnet_nodes+1));
    assert_eq!(unbondings.len(), 1);
    let (ledger_epoch, ledger_balance) = unbondings.iter().next().unwrap();
    assert_eq!(*ledger_epoch, &epoch + DelegateStakeCooldownEpochs::get());
    assert_eq!(*ledger_balance, expected_ledger_balance);

    System::set_block_number(System::block_number() + ((EpochLength::get()  + 1) * DelegateStakeCooldownEpochs::get()));

    assert_ok!(
      Network::claim_unbondings(
        RuntimeOrigin::signed(account(total_subnet_nodes+1)),
      )
    );

    let post_balance = Balances::free_balance(&account(total_subnet_nodes+1));

    assert!(
      (post_balance >= Network::percent_mul(starting_delegator_balance, 990000000)) &&
      (post_balance < starting_delegator_balance)
    );

    let unbondings: BTreeMap<u32, u128> = StakeUnbondingLedger::<Test>::get(account(total_subnet_nodes+1));
    assert_eq!(unbondings.len(), 0);
  });
}

#[test]
fn test_add_to_delegate_stake() {
  new_test_ext().execute_with(|| {
    let subnet_path: Vec<u8> = "petals-team/StableBeluga2".into();
    let deposit_amount: u128 = 10000000000000000000000;
    let amount: u128 = 1000000000000000000000;
    let stake_amount: u128 = MinStakeBalance::<Test>::get();

    build_activated_subnet(subnet_path.clone(), 0, 0, deposit_amount, stake_amount);

    let subnet_id = SubnetPaths::<Test>::get(subnet_path.clone()).unwrap();
    let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);

    let n_account = total_subnet_nodes + 1;

    let _ = Balances::deposit_creating(&account(n_account), amount+500);

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
      (delegate_balance >= Network::percent_mul(amount, 990000000)) &&
      (delegate_balance < amount)
    );
  });
}

#[test]
fn test_add_to_delegate_stake_increase_pool_check_balance() {
  new_test_ext().execute_with(|| {
    let subnet_path: Vec<u8> = "petals-team/StableBeluga2".into();
    let deposit_amount: u128 = 10000000000000000000000;
    let amount: u128 = 1000000000000000000000;

    let stake_amount: u128 = MinStakeBalance::<Test>::get();

    build_activated_subnet(subnet_path.clone(), 0, 0, deposit_amount, stake_amount);

    let subnet_id = SubnetPaths::<Test>::get(subnet_path.clone()).unwrap();
    let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);

    let n_account = total_subnet_nodes + 1;

    let _ = Balances::deposit_creating(&account(n_account), amount+500);

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
    log::error!("total_subnet_delegated_stake_shares  {:?}", total_subnet_delegated_stake_shares);
    log::error!("total_subnet_delegated_stake_balance {:?}", total_subnet_delegated_stake_balance);

    let delegate_balance = Network::convert_to_balance(
      delegate_shares,
      total_subnet_delegated_stake_shares,
      total_subnet_delegated_stake_balance
    );
    log::error!("delegate_balance                     {:?}", delegate_balance);

    // The first depositor will lose a percentage of their deposit depending on the size
    // https://docs.openzeppelin.com/contracts/4.x/erc4626#inflation-attack
    // assert_eq!(delegate_balance, delegate_stake_to_be_added_as_shares);
    assert!(
      (delegate_balance >= Network::percent_mul(amount, 990000000)) &&
      (delegate_balance < amount)
    );

    let increase_delegate_stake_amount: u128 = 1000000000000000000000;

    let total_subnet_delegated_stake_shares = TotalSubnetDelegateStakeShares::<Test>::get(subnet_id);
    let total_subnet_delegated_stake_balance = TotalSubnetDelegateStakeBalance::<Test>::get(subnet_id);
    let expected_post_delegate_balance = Network::convert_to_balance(
      delegate_shares,
      total_subnet_delegated_stake_shares,
      total_subnet_delegated_stake_balance + increase_delegate_stake_amount
    );

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
    log::error!("post_delegate_balance      {:?}", post_delegate_balance);
    assert_eq!(post_delegate_balance, expected_post_delegate_balance);
  });
}

#[test]
fn test_claim_removal_of_delegate_stake() {
  new_test_ext().execute_with(|| {
    let _ = env_logger::builder().is_test(true).try_init();

    let subnet_path: Vec<u8> = "petals-team/StableBeluga2".into();
    let deposit_amount: u128 = 10000000000000000000000;
    let amount: u128 = 1000000000000000000000;

    let stake_amount: u128 = MinStakeBalance::<Test>::get();

    build_activated_subnet(subnet_path.clone(), 0, 0, deposit_amount, stake_amount);

    let subnet_id = SubnetPaths::<Test>::get(subnet_path.clone()).unwrap();
    let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);

    let n_account = total_subnet_nodes + 1;

    let _ = Balances::deposit_creating(&account(n_account), amount+500);

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
      (delegate_balance >= Network::percent_mul(amount, 990000000)) &&
      (delegate_balance < amount)
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

    let unbondings: BTreeMap<u32, u128> = StakeUnbondingLedger::<Test>::get(account(n_account));
    assert_eq!(unbondings.len(), 1);
    let (ledger_epoch, ledger_balance) = unbondings.iter().next().unwrap();
    assert_eq!(*ledger_epoch, &epoch + DelegateStakeCooldownEpochs::get());
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

    log::error!("starting_delegator_balance {:?}", starting_delegator_balance);
    log::error!("after_claim_balance        {:?}", after_claim_balance);
    log::error!("post_balance               {:?}", post_balance);
    log::error!("ledger_balance             {:?}", ledger_balance);

    let unbondings: BTreeMap<u32, u128> = StakeUnbondingLedger::<Test>::get(account(n_account));
    assert_eq!(unbondings.len(), 0);
  });
}

// // #[test]
// // fn test_remove_to_delegate_stake_max_unlockings_per_epoch_err() {
// //   new_test_ext().execute_with(|| {
// //     let subnet_path: Vec<u8> = "petals-team/StableBeluga2".into();
// //     let deposit_amount: u128 = 10000000000000000000000;
// //     let amount: u128 = 1000000000000000000000;

// //     let stake_amount: u128 = MinStakeBalance::<Test>::get();

// //     build_activated_subnet(subnet_path.clone(), 0, 0, deposit_amount, stake_amount);

// //     let subnet_id = SubnetPaths::<Test>::get(subnet_path.clone()).unwrap();
// //     let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);

// //     let n_account = total_subnet_nodes + 1;

// //     let _ = Balances::deposit_creating(&account(n_account), amount+500);

// //     let subnet_id = SubnetPaths::<Test>::get(subnet_path.clone()).unwrap();

// //     let total_subnet_delegated_stake_shares = TotalSubnetDelegateStakeShares::<Test>::get(subnet_id);
// //     let total_subnet_delegated_stake_balance = TotalSubnetDelegateStakeBalance::<Test>::get(subnet_id);

// //     let mut delegate_stake_to_be_added_as_shares = Network::convert_to_shares(
// //       amount,
// //       total_subnet_delegated_stake_shares,
// //       total_subnet_delegated_stake_balance
// //     );

// //     if total_subnet_delegated_stake_shares == 0 {
// //       delegate_stake_to_be_added_as_shares = delegate_stake_to_be_added_as_shares.saturating_sub(1000);
// //     }

// //     System::set_block_number(System::block_number() + DelegateStakeCooldownEpochs::get() * EpochLength::get());

// //     let starting_delegator_balance = Balances::free_balance(&account(n_account));

// //     assert_ok!(
// //       Network::add_to_delegate_stake(
// //         RuntimeOrigin::signed(account(n_account)),
// //         subnet_id,
// //         amount,
// //       ) 
// //     );

// //     let delegate_shares = AccountSubnetDelegateStakeShares::<Test>::get(account(n_account), subnet_id);

// //     assert_ok!(
// //       Network::remove_delegate_stake(
// //         RuntimeOrigin::signed(account(n_account)),
// //         subnet_id,
// //         delegate_shares/2,
// //       )
// //     );
// //     let unbondings: BTreeMap<u32, u128> = StakeUnbondingLedger::<Test>::get(account(n_account));
// //     assert_eq!(unbondings.len(), 1);

// //     assert_err!(
// //       Network::remove_delegate_stake(
// //         RuntimeOrigin::signed(account(n_account)),
// //         subnet_id,
// //         delegate_shares/2,
// //       ),
// //       Error::<Test>::MaxUnlockingsPerEpochReached
// //     );
// //   });
// // }

#[test]
fn test_remove_to_delegate_stake_max_unlockings_reached_err() {
  new_test_ext().execute_with(|| {
    let subnet_path: Vec<u8> = "petals-team/StableBeluga2".into();
    let deposit_amount: u128 = 10000000000000000000000;
    let amount: u128 = 1000000000000000000000;

    let stake_amount: u128 = MinStakeBalance::<Test>::get();

    build_activated_subnet(subnet_path.clone(), 0, 0, deposit_amount, stake_amount);

    let subnet_id = SubnetPaths::<Test>::get(subnet_path.clone()).unwrap();
    let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);

    let n_account = total_subnet_nodes + 1;

    let _ = Balances::deposit_creating(&account(n_account), amount+500);

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
    for n in 1..max_unlockings+2 {
      System::set_block_number(System::block_number() + EpochLength::get() + 1);
      if n > max_unlockings {
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
        let unbondings: BTreeMap<u32, u128> = StakeUnbondingLedger::<Test>::get(account(n_account));
        assert_eq!(unbondings.len() as u32, n);
      }
    }
  });
}

#[test]
fn test_switch_delegate_stake() {
  new_test_ext().execute_with(|| {
    let deposit_amount: u128 = 10000000000000000000000;
    let amount: u128 = 1000000000000000000000;
    let stake_amount: u128 = MinStakeBalance::<Test>::get();

    let from_subnet_path: Vec<u8> = "petals-team/StableBeluga2".into();
    build_activated_subnet(from_subnet_path.clone(), 0, 0, deposit_amount, stake_amount);
    let from_subnet_id = SubnetPaths::<Test>::get(from_subnet_path.clone()).unwrap();

    let to_subnet_path: Vec<u8> = "petals-team/StableBeluga3".into();
    build_activated_subnet(to_subnet_path.clone(), 0, 0, deposit_amount, stake_amount);
    let to_subnet_id = SubnetPaths::<Test>::get(to_subnet_path.clone()).unwrap();

    let n_account = 255;

    let _ = Balances::deposit_creating(&account(n_account), amount+500);

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
      (to_delegate_balance >= Network::percent_mul(from_delegate_balance, 990000000)) &&
      (to_delegate_balance < from_delegate_balance)
    );
  });
}

#[test]
fn test_switch_delegate_stake_not_enough_stake_err() {
  new_test_ext().execute_with(|| {
    let _ = env_logger::builder().is_test(true).try_init();

    let deposit_amount: u128 = 10000000000000000000000;
    let amount: u128 = 1000000000000000000000;
    let stake_amount: u128 = MinStakeBalance::<Test>::get();

    let from_subnet_path: Vec<u8> = "petals-team/StableBeluga2".into();
    build_activated_subnet(from_subnet_path.clone(), 0, 0, deposit_amount, stake_amount);
    let from_subnet_id = SubnetPaths::<Test>::get(from_subnet_path.clone()).unwrap();

    let to_subnet_path: Vec<u8> = "petals-team/StableBeluga3".into();
    build_activated_subnet(to_subnet_path.clone(), 0, 0, deposit_amount, stake_amount);
    let to_subnet_id = SubnetPaths::<Test>::get(to_subnet_path.clone()).unwrap();

    // let n_account = 255;

    // let _ = Balances::deposit_creating(&account(n_account), amount+500);

    // assert_err!(
    //   Network::transfer_delegate_stake(
    //     RuntimeOrigin::signed(account(n_account)),
    //     from_subnet_id,
    //     to_subnet_id,
    //     0,
    //   ),
    //   Error::<Test>::NotEnoughStakeToWithdraw
    // );

    // assert_err!(
    //   Network::transfer_delegate_stake(
    //     RuntimeOrigin::signed(account(n_account)),
    //     from_subnet_id,
    //     to_subnet_id,
    //     1000,
    //   ),
    //   Error::<Test>::NotEnoughStakeToWithdraw
    // );
  });
}

// // #[test]
// // fn test_remove_to_delegate_stake_epochs_not_met_err() {
// //   new_test_ext().execute_with(|| {
// //     let subnet_path: Vec<u8> = "petals-team/StableBeluga2".into();

// //     build_subnet(subnet_path.clone());
// //     let deposit_amount: u128 = 10000000000000000000000;
// //     let amount: u128 = 1000000000000000000000;
// //     let _ = Balances::deposit_creating(&account(0), amount+500);

// //     let subnet_id = SubnetPaths::<Test>::get(subnet_path.clone()).unwrap();

// //     let total_subnet_delegated_stake_shares = TotalSubnetDelegateStakeShares::<Test>::get(subnet_id);
// //     let total_subnet_delegated_stake_balance = TotalSubnetDelegateStakeBalance::<Test>::get(subnet_id);

// //     let mut delegate_stake_to_be_added_as_shares = Network::convert_to_shares(
// //       amount,
// //       total_subnet_delegated_stake_shares,
// //       total_subnet_delegated_stake_balance
// //     );

// //     if total_subnet_delegated_stake_shares == 0 {
// //       delegate_stake_to_be_added_as_shares = delegate_stake_to_be_added_as_shares.saturating_sub(1000);
// //     }

// //     System::set_block_number(System::block_number() + DelegateStakeCooldownEpochs::get() * EpochLength::get());

// //     assert_ok!(
// //       Network::add_to_delegate_stake(
// //         RuntimeOrigin::signed(account(0)),
// //         subnet_id,
// //         amount,
// //       ) 
// //     );

// //     let delegate_shares = AccountSubnetDelegateStakeShares::<Test>::get(account(0), subnet_id);
// //     assert_eq!(delegate_shares, delegate_stake_to_be_added_as_shares);
// //     assert_ne!(delegate_shares, 0);

// //     let total_subnet_delegated_stake_shares = TotalSubnetDelegateStakeShares::<Test>::get(subnet_id);
// //     let total_subnet_delegated_stake_balance = TotalSubnetDelegateStakeBalance::<Test>::get(subnet_id);

// //     let mut delegate_balance = Network::convert_to_balance(
// //       delegate_shares,
// //       total_subnet_delegated_stake_shares,
// //       total_subnet_delegated_stake_balance
// //     );
// //     // The first depositor will lose a percentage of their deposit depending on the size
// //     // https://docs.openzeppelin.com/contracts/4.x/erc4626#inflation-attack
// //     assert_eq!(delegate_balance, delegate_stake_to_be_added_as_shares);
// //     assert!(
// //       (delegate_balance >= Network::percent_mul(amount, 990000000)) &&
// //       (delegate_balance < amount)
// //     );

// //     // assert_err!(
// //     //   Network::remove_delegate_stake(
// //     //     RuntimeOrigin::signed(account(0)),
// //     //     subnet_id,
// //     //     delegate_shares,
// //     //   ),
// //     //   Error::<Test>::InsufficientCooldown
// //     // );
// //   });
// // }

#[test]
fn test_remove_delegate_stake_after_subnet_remove() {
  new_test_ext().execute_with(|| {
    let subnet_path: Vec<u8> = "petals-team/StableBeluga2".into();
    let deposit_amount: u128 = 10000000000000000000000;
    let amount: u128 = 1000000000000000000000;

    let stake_amount: u128 = MinStakeBalance::<Test>::get();

    build_activated_subnet(subnet_path.clone(), 0, 0, deposit_amount, stake_amount);

    let subnet_id = SubnetPaths::<Test>::get(subnet_path.clone()).unwrap();
    let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);

    let n_account = total_subnet_nodes + 1;

    let _ = Balances::deposit_creating(&account(n_account), amount+500);

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
      (delegate_balance >= Network::percent_mul(amount, 990000000)) &&
      (delegate_balance < amount)
    );

    let epoch_length = EpochLength::get();
    let cooldown_epochs = DelegateStakeCooldownEpochs::get();

    assert_ok!(
      Network::do_remove_subnet( 
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

    let unbondings: BTreeMap<u32, u128> = StakeUnbondingLedger::<Test>::get(account(n_account));
    assert_eq!(unbondings.len(), 1);
    let (ledger_epoch, ledger_balance) = unbondings.iter().next().unwrap();
    assert_eq!(*ledger_epoch, &epoch + DelegateStakeCooldownEpochs::get());
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
      (post_balance >= Network::percent_mul(starting_delegator_balance, 990000000)) &&
      (post_balance < starting_delegator_balance)
    );

    let unbondings: BTreeMap<u32, u128> = StakeUnbondingLedger::<Test>::get(account(n_account));
    assert_eq!(unbondings.len(), 0);
  });
}

#[test]
fn test_switch_delegate_stake_node_to_subnet() {
  new_test_ext().execute_with(|| {
    let subnet_path: Vec<u8> = "petals-team/StableBeluga2".into();
    let deposit_amount: u128 = 10000000000000000000000;
    let amount: u128 =         1000000000000000000000;

    let stake_amount: u128 = MinStakeBalance::<Test>::get();

    build_activated_subnet_with_delegator_rewards(
      subnet_path.clone(), 
      0, 
      16, 
      deposit_amount, 
      stake_amount,
      DEFAULT_DELEGATE_REWARD_RATE,
    );

    let from_subnet_id = SubnetPaths::<Test>::get(subnet_path.clone()).unwrap();
    let total_from_subnet_nodes = TotalSubnetNodes::<Test>::get(from_subnet_id);

    let to_subnet_path: Vec<u8> = "petals-team/StableBeluga3".into();

    build_activated_subnet_with_delegator_rewards(
      to_subnet_path.clone(), 
      0, 
      16, 
      deposit_amount, 
      stake_amount,
      DEFAULT_DELEGATE_REWARD_RATE,
    );

    let to_subnet_id = SubnetPaths::<Test>::get(to_subnet_path.clone()).unwrap();

    let _ = Balances::deposit_creating(&account(total_from_subnet_nodes+1), amount+500);

    assert_ok!(
      Network::add_to_node_delegate_stake(
        RuntimeOrigin::signed(account(total_from_subnet_nodes+1)), 
        from_subnet_id,
        1,
        amount,
      )
    );

    let account_node_delegate_stake_shares = AccountNodeDelegateStakeShares::<Test>::get((account(total_from_subnet_nodes+1), from_subnet_id, 1));
    let total_node_delegate_stake_balance = TotalNodeDelegateStakeBalance::<Test>::get(from_subnet_id, 1);
    let total_node_delegate_stake_shares = TotalNodeDelegateStakeShares::<Test>::get(from_subnet_id, 1);

    let account_node_delegate_stake_balance = Network::convert_to_balance(
      account_node_delegate_stake_shares,
      total_node_delegate_stake_shares,
      total_node_delegate_stake_balance
    );

    assert!(
      (account_node_delegate_stake_balance >= Network::percent_mul(amount, 990000000)) &&
      (account_node_delegate_stake_balance < amount)
    );

    let account_node_delegate_stake_shares_to_be_removed = account_node_delegate_stake_shares / 2;

    let expected_balance_to_be_removed = Network::convert_to_balance(
      account_node_delegate_stake_shares_to_be_removed,
      total_node_delegate_stake_shares,
      total_node_delegate_stake_balance
    );

    let before_transfer_tensor = Balances::free_balance(&account(total_from_subnet_nodes+1));

    let unbondings: BTreeMap<u32, u128> = StakeUnbondingLedger::<Test>::get(account(total_from_subnet_nodes+1));
    assert_eq!(unbondings.len(), 0);

    assert_ok!(
      Network::transfer_from_node_to_subnet(
        RuntimeOrigin::signed(account(total_from_subnet_nodes+1)),
        from_subnet_id,
        1,
        to_subnet_id,
        account_node_delegate_stake_shares_to_be_removed,
      )
    );

    let unbondings: BTreeMap<u32, u128> = StakeUnbondingLedger::<Test>::get(account(total_from_subnet_nodes+1));
    assert_eq!(unbondings.len(), 0);

    let after_transfer_tensor = Balances::free_balance(&account(total_from_subnet_nodes+1));
    assert_eq!(after_transfer_tensor, before_transfer_tensor);

    let from_delegate_shares = AccountSubnetDelegateStakeShares::<Test>::get(account(total_from_subnet_nodes+1), from_subnet_id);
    assert_eq!(from_delegate_shares, 0);

    let to_delegate_shares = AccountSubnetDelegateStakeShares::<Test>::get(account(total_from_subnet_nodes+1), to_subnet_id);
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
      (to_delegate_balance >= Network::percent_mul(expected_balance_to_be_removed, 990000000)) &&
      (to_delegate_balance < expected_balance_to_be_removed)
    );
  });
}

#[test]
fn test_switch_delegate_stake_subnet_to_node() {
  new_test_ext().execute_with(|| {
    let deposit_amount: u128 = 10000000000000000000000;
    let amount: u128 = 1000000000000000000000;
    let stake_amount: u128 = MinStakeBalance::<Test>::get();

    let from_subnet_path: Vec<u8> = "petals-team/StableBeluga2".into();
    build_activated_subnet(from_subnet_path.clone(), 0, 16, deposit_amount, stake_amount);
    let from_subnet_id = SubnetPaths::<Test>::get(from_subnet_path.clone()).unwrap();

    let to_subnet_path: Vec<u8> = "petals-team/StableBeluga3".into();
    build_activated_subnet(to_subnet_path.clone(), 0, 16, deposit_amount, stake_amount);
    let to_subnet_id = SubnetPaths::<Test>::get(to_subnet_path.clone()).unwrap();

    let n_account = 255;

    let _ = Balances::deposit_creating(&account(n_account), amount+500);

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

    let unbondings: BTreeMap<u32, u128> = StakeUnbondingLedger::<Test>::get(account(n_account));
    assert_eq!(unbondings.len(), 0);
    let before_transfer_tensor = Balances::free_balance(&account(n_account));

    assert_ok!(
      Network::transfer_from_subnet_to_node(
        RuntimeOrigin::signed(account(n_account)),
        from_subnet_id,
        to_subnet_id,
        1,
        delegate_shares,
      )
    );

    let unbondings: BTreeMap<u32, u128> = StakeUnbondingLedger::<Test>::get(account(n_account));
    assert_eq!(unbondings.len(), 0);
    let after_transfer_tensor = Balances::free_balance(&account(n_account));
    assert_eq!(after_transfer_tensor, before_transfer_tensor);

    let account_node_delegate_stake_shares = AccountNodeDelegateStakeShares::<Test>::get((account(n_account), to_subnet_id, 1));
    let total_node_delegate_stake_balance = TotalNodeDelegateStakeBalance::<Test>::get(to_subnet_id, 1);
    let total_node_delegate_stake_shares = TotalNodeDelegateStakeShares::<Test>::get(to_subnet_id, 1);

    let account_node_delegate_stake_balance = Network::convert_to_balance(
      account_node_delegate_stake_shares,
      total_node_delegate_stake_shares,
      total_node_delegate_stake_balance
    );

    assert_ne!(account_node_delegate_stake_balance, 0);

    assert!(
      (account_node_delegate_stake_balance >= Network::percent_mul(from_delegate_balance, 990000000)) &&
      (account_node_delegate_stake_balance < from_delegate_balance)
    );
  });
}
