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
  SubnetRewardsValidator,
  SubnetRewardsSubmission,
  SubnetNodePenalties,
  BaseRewardPerMB,
  DelegateStakeRewardsPercentage,
  BaseValidatorReward,
  SubnetNodesData,
  TotalNodeDelegateStakeShares,
  AccountSubnetStake,
  HotkeySubnetNodeId,
  SubnetNodeIdHotkey,
  SubnetNodeClass,
  AccountNodeDelegateStakeShares,
  TotalNodeDelegateStakeBalance,
};

//
//
//
//
//
//
//
// Node delegate staking
//
//
//
//
//
//
//

#[test]
fn test_add_to_node_delegate_stake() {
  new_test_ext().execute_with(|| {
    let subnet_path: Vec<u8> = "petals-team/StableBeluga2".into();
    let deposit_amount: u128 = 10000000000000000000000;
    let amount: u128 = 1000000000000000000000;

    build_activated_subnet_with_delegator_rewards(
      subnet_path.clone(), 
      0, 
      16, 
      deposit_amount, 
      amount,
      DEFAULT_DELEGATE_REWARD_RATE,
    );

    let subnet_id = SubnetPaths::<Test>::get(subnet_path.clone()).unwrap();
    let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);

    let _ = Balances::deposit_creating(&account(total_subnet_nodes+1), amount+500);

    assert_ok!(
      Network::add_to_node_delegate_stake(
        RuntimeOrigin::signed(account(total_subnet_nodes+1)), 
        subnet_id,
        0,
        amount,
      )
    );

    let account_node_delegate_stake_shares = AccountNodeDelegateStakeShares::<Test>::get((account(total_subnet_nodes+1), subnet_id, 0));
    let total_node_delegate_stake_balance = TotalNodeDelegateStakeBalance::<Test>::get(subnet_id, 0);
    let total_node_delegate_stake_shares = TotalNodeDelegateStakeShares::<Test>::get(subnet_id, 0);

    let account_node_delegate_stake_balance = Network::convert_to_balance(
      account_node_delegate_stake_shares,
      total_node_delegate_stake_shares,
      total_node_delegate_stake_balance
    );

    assert!(
      (account_node_delegate_stake_balance >= Network::percent_mul(amount, 9999)) &&
      (account_node_delegate_stake_balance <= amount)
    );
  })
}

#[test]
fn test_remove_node_delegate_stake() {
  new_test_ext().execute_with(|| {
    let subnet_path: Vec<u8> = "petals-team/StableBeluga2".into();
    let deposit_amount: u128 = 10000000000000000000000;
    let amount: u128 =         1000000000000000000000;

    build_activated_subnet_with_delegator_rewards(
      subnet_path.clone(), 
      0, 
      16, 
      deposit_amount, 
      amount,
      DEFAULT_DELEGATE_REWARD_RATE,
    );

    let subnet_id = SubnetPaths::<Test>::get(subnet_path.clone()).unwrap();
    let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);

    let _ = Balances::deposit_creating(&account(total_subnet_nodes+1), amount+500);

    assert_ok!(
      Network::add_to_node_delegate_stake(
        RuntimeOrigin::signed(account(total_subnet_nodes+1)), 
        subnet_id,
        0,
        amount,
      )
    );

    let account_node_delegate_stake_shares = AccountNodeDelegateStakeShares::<Test>::get((account(total_subnet_nodes+1), subnet_id, 0));
    let total_node_delegate_stake_balance = TotalNodeDelegateStakeBalance::<Test>::get(subnet_id, 0);
    let total_node_delegate_stake_shares = TotalNodeDelegateStakeShares::<Test>::get(subnet_id, 0);
    log::error!("account_node_delegate_stake_shares {:?}", account_node_delegate_stake_shares);
    log::error!("total_node_delegate_stake_balance  {:?}", total_node_delegate_stake_balance);
    log::error!("total_node_delegate_stake_shares   {:?}", total_node_delegate_stake_shares);

    let account_node_delegate_stake_balance = Network::convert_to_balance(
      account_node_delegate_stake_shares,
      total_node_delegate_stake_shares,
      total_node_delegate_stake_balance
    );
    log::error!("account_node_delegate_stake_balance      {:?}", account_node_delegate_stake_balance);

    assert!(
      (account_node_delegate_stake_balance >= Network::percent_mul(amount, 9999)) &&
      (account_node_delegate_stake_balance <= amount)
    );

    let account_node_delegate_stake_shares_to_be_removed = account_node_delegate_stake_shares / 2;
    log::error!("account_node_delegate_stake_shares_to_be_removed {:?}", account_node_delegate_stake_shares_to_be_removed);

    let expected_balance_to_be_removed = Network::convert_to_balance(
      account_node_delegate_stake_shares_to_be_removed,
      total_node_delegate_stake_shares,
      total_node_delegate_stake_balance
    );
    log::error!("expected_balance_to_be_removed           {:?}", expected_balance_to_be_removed);

    let expected_post_balance = account_node_delegate_stake_balance - expected_balance_to_be_removed;
    log::error!("expected_post_balance                    {:?}", expected_post_balance);

    assert_ok!(
      Network::remove_node_delegate_stake(
        RuntimeOrigin::signed(account(total_subnet_nodes+1)), 
        subnet_id,
        0,
        account_node_delegate_stake_shares_to_be_removed,
      )
    );

    let account_node_delegate_stake_shares = AccountNodeDelegateStakeShares::<Test>::get((account(total_subnet_nodes+1), subnet_id, 0));
    let total_node_delegate_stake_balance = TotalNodeDelegateStakeBalance::<Test>::get(subnet_id, 0);
    let total_node_delegate_stake_shares = TotalNodeDelegateStakeShares::<Test>::get(subnet_id, 0);
    log::error!("post account_node_delegate_stake_shares  {:?}", account_node_delegate_stake_shares);
    log::error!("post total_node_delegate_stake_balance   {:?}", total_node_delegate_stake_balance);
    log::error!("post total_node_delegate_stake_shares    {:?}", total_node_delegate_stake_shares);

    assert_eq!(account_node_delegate_stake_shares, account_node_delegate_stake_shares_to_be_removed);

    let post_account_node_delegate_stake_balance = Network::convert_to_balance(
      account_node_delegate_stake_shares,
      total_node_delegate_stake_shares,
      total_node_delegate_stake_balance
    );
    log::error!("post_account_node_delegate_stake_balance {:?}", post_account_node_delegate_stake_balance);

    // assert_eq!(expected_post_balance, post_account_node_delegate_stake_balance);

    // let expected_balance = Network::percent_mul(account_node_delegate_stake_balance/2, 9999);
    // log::error!("expected_balance                         {:?}", expected_balance);

    // assert!(
    //   post_account_node_delegate_stake_balance >= expected_balance
    // );

    // assert!(
    //   (post_account_node_delegate_stake_balance >= expected_balance) &&
    //   (post_account_node_delegate_stake_balance <= (account_node_delegate_stake_balance/2))
    // );

  })
}

#[test]
fn test_validate_with_delegate_rewards_rate() {
  new_test_ext().execute_with(|| {
    let subnet_path: Vec<u8> = "petals-team/StableBeluga2".into();
    let deposit_amount: u128 = 10000000000000000000000;
    let amount: u128 = 1000000000000000000000;

    build_activated_subnet_with_delegator_rewards(
      subnet_path.clone(), 
      0, 
      16, 
      deposit_amount, 
      amount,
      DEFAULT_DELEGATE_REWARD_RATE,
    );

    let subnet_id = SubnetPaths::<Test>::get(subnet_path.clone()).unwrap();
    let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);

    let _ = Balances::deposit_creating(&account(total_subnet_nodes+1), amount+500);

    assert_ok!(
      Network::add_to_node_delegate_stake(
        RuntimeOrigin::signed(account(total_subnet_nodes+1)), 
        subnet_id,
        0,
        amount,
      )
    );

    increase_epochs(1);

    let epoch_length = EpochLength::get();
    let epoch = System::block_number() / epoch_length;

    let subnet_node_data_vec = subnet_node_data(0, total_subnet_nodes);
  
    // --- Insert validator
    SubnetRewardsValidator::<Test>::insert(subnet_id, epoch as u32, 0);

    // validate without n-1
    assert_ok!(
      Network::validate(
        RuntimeOrigin::signed(account(0)), 
        subnet_id,
        subnet_node_data_vec.clone(),
        None,
      )
    );

    // Attest without n-1
    for n in 1..total_subnet_nodes {
      assert_ok!(
        Network::attest(
          RuntimeOrigin::signed(account(n)), 
          subnet_id,
        )
      );
    }
    
    // --- Get submission data and count before node is removed
    // Check rewards
    // Ensure only attestors, validators, and validated get rewards
    let submission = SubnetRewardsSubmission::<Test>::get(subnet_id, epoch as u32).unwrap();

    // --- Any removals impact the following epochs attestation data unless removed ahead of rewards
    let submission_nodes: BTreeSet<<Test as frame_system::Config>::AccountId> = Network::get_classified_hotkeys(subnet_id, &SubnetNodeClass::Validator, epoch);
    let submission_nodes_count = submission_nodes.len() as u128;

    Network::reward_subnets(System::block_number(), epoch as u32);
    let node_absent_count = SubnetNodePenalties::<Test>::get(subnet_id, total_subnet_nodes-1);
    assert_eq!(node_absent_count, 0); 
          
    let base_reward_per_mb: u128 = BaseRewardPerMB::<Test>::get();
    let delegate_stake_rewards_percentage: u128 = DelegateStakeRewardsPercentage::<Test>::get();
    let overall_subnet_reward: u128 = Network::percent_mul(base_reward_per_mb, DEFAULT_MEM_MB);
    let delegate_stake_reward: u128 = Network::percent_mul(overall_subnet_reward, delegate_stake_rewards_percentage);
    let subnet_reward: u128 = overall_subnet_reward.saturating_sub(delegate_stake_reward);

    let sum = submission.data.iter().fold(0, |acc, x| acc + x.score);
    let reward_ratio: u128 = Network::percent_div(DEFAULT_SCORE, sum);
    let account_reward: u128 = Network::percent_mul(reward_ratio, subnet_reward);

    let base_reward = BaseValidatorReward::<Test>::get();

    let submission_attestations: u128 = submission.attests.len() as u128;
    let attestation_percentage: u128 = Network::percent_div(submission_attestations, submission_nodes_count);

    // check each subnet nodes balance increased
    for n in 0..total_subnet_nodes {
      let hotkey_subnet_node_id = HotkeySubnetNodeId::<Test>::get(subnet_id, account(n)).unwrap();
      let subnet_node_id_hotkey = SubnetNodeIdHotkey::<Test>::get(subnet_id, hotkey_subnet_node_id).unwrap();
      let subnet_node = SubnetNodesData::<Test>::get(subnet_id, hotkey_subnet_node_id);

      if n == 0 {
        // validator
        let stake_balance: u128 = AccountSubnetStake::<Test>::get(&account(n), subnet_id);
        let validator_reward: u128 = Network::percent_mul(base_reward, attestation_percentage);
        let mut validator_total_reward: u128 = (account_reward as u128) + (validator_reward as u128);

        // --- Subtract node delegator rewards
        if subnet_node.delegate_reward_rate != 0 {
          let total_node_delegated_stake_shares = TotalNodeDelegateStakeShares::<Test>::get(subnet_id, hotkey_subnet_node_id);
          if total_node_delegated_stake_shares != 0 {
            let node_delegate_reward = Network::percent_mul(validator_total_reward, subnet_node.delegate_reward_rate);
            validator_total_reward = validator_total_reward - node_delegate_reward;
          }
        }

        assert_eq!(stake_balance, amount + validator_total_reward);
      } else {
        // attestors
        let stake_balance: u128 = AccountSubnetStake::<Test>::get(&account(n), subnet_id);
        let mut reward: u128 = account_reward;

        if subnet_node.delegate_reward_rate != 0 {
          let total_node_delegated_stake_shares = TotalNodeDelegateStakeShares::<Test>::get(subnet_id, hotkey_subnet_node_id);
          if total_node_delegated_stake_shares != 0 {
            let node_delegate_reward = Network::percent_mul(reward, subnet_node.delegate_reward_rate);
            reward = reward - node_delegate_reward;
          }
        }

        assert!(stake_balance == amount + reward, "Invalid subnet node staking rewards")  
      }
    }
  });
}