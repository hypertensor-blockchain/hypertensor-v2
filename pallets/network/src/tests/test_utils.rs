use super::mock::*;
use crate::Event;
use sp_core::OpaquePeerId as PeerId;
use frame_support::assert_ok;
use log::info;
use crate::{
  SubnetNodeData, 
  TotalStake, 
  SubnetRewardsValidator,
  SubnetPaths, 
  SubnetNodeClass,
  SubnetsData,
  AccountSubnetStake, 
  MinStakeBalance,
  TotalSubnetDelegateStakeBalance,
  AccountSubnetDelegateStakeShares, 
  RegistrationSubnetData,
  StakeUnbondingLedger, 
  TotalSubnetStake, 
  MinSubnetRegistrationBlocks,
  HotkeySubnetNodeId, 
  SubnetNodeIdHotkey, 
  SubnetNodesData, 
  PeerIdSubnetNode,
  HotkeyOwner,
  MinSubnetNodes,
  LastSubnetRegistration,
  TotalSubnetNodes,
  TotalSubnetNodeUids,
  BootstrapPeerIdSubnetNode,
  SubnetNodeUniqueParam,
  SubnetPenaltyCount,
  SubnetRewardsSubmission,
  Proposals,
  SubnetRegistrationColdkeyWhitelist,
  SubnetNodeNonUniqueParamLastSet,
  SubnetNodePenalties,
  SubnetNodeRegistrationInterval,
  SubnetRegistrationEpochs,
  SubnetOwner,
  SubnetRegistrationEpoch,
};
use frame_support::traits::{OnInitialize, Currency};
use sp_std::collections::btree_set::BTreeSet;

pub type AccountIdOf<Test> = <Test as frame_system::Config>::AccountId;

pub fn account(id: u32) -> AccountIdOf<Test> {
	[id as u8; 32].into()
}

// it is possible to use `use libp2p::PeerId;` with `PeerId::random()`
// https://github.com/paritytech/substrate/blob/033d4e86cc7eff0066cd376b9375f815761d653c/frame/node-authorization/src/mock.rs#L90
// fn peer(id: u8) -> PeerId {
// 	PeerId(vec![id])
// }

pub fn peer(id: u32) -> PeerId {
   
	// let peer_id = format!("12D3KooWD3eckifWpRn9wQpMG9R9hX3sD158z7EqHWmweQAJU5SA{id}");
  let peer_id = format!("QmYyQSo1c1Ym7orWxLYvCrM2EmxFTANf8wXmmE7DWjhx5N{id}"); 
	PeerId(peer_id.into())
}
// bafzbeie5745rpv2m6tjyuugywy4d5ewrqgqqhfnf445he3omzpjbx5xqxe
// QmYyQSo1c1Ym7orWxLYvCrM2EmxFTANf8wXmmE7DWjhx5N
// 12D3KooWD3eckifWpRn9wQpMG9R9hX3sD158z7EqHWmweQAJU5SA

pub fn get_min_stake_balance() -> u128 {
	MinStakeBalance::<Test>::get()
}

pub const PERCENTAGE_FACTOR: u128 = 10000;
pub const DEFAULT_SCORE: u128 = 5000;
pub const DEFAULT_MEM_MB: u128 = 50000;
pub const MAX_SUBNET_NODES: u32 = 254;
pub const DEFAULT_REGISTRATION_BLOCKS: u32 = 130_000;
pub const DEFAULT_DELEGATE_REWARD_RATE: u128 = 100_000_000; // 10%

pub fn build_activated_subnet(subnet_path: Vec<u8>, start: u32, mut end: u32, deposit_amount: u128, amount: u128) {
  let epoch_length = EpochLength::get();
  let block_number = System::block_number();
  let epoch = System::block_number().saturating_div(epoch_length);
  let next_registration_epoch = Network::get_next_registration_epoch(epoch);
  increase_epochs(next_registration_epoch.saturating_sub(epoch));

  let cost = Network::registration_cost(epoch);
  let _ = Balances::deposit_creating(&account(0), cost+1000);

  let min_nodes = MinSubnetNodes::<Test>::get();

  if end == 0 {
    end = min_nodes;
  }

  let whitelist = get_coldkey_whitelist(start, end);

  let add_subnet_data = RegistrationSubnetData {
    path: subnet_path.clone().into(),
    max_node_registration_epochs: 16,
    node_registration_interval: 0,
    node_activation_interval: 0,
    node_queue_period: 1,
    max_node_penalties: 3,
    coldkey_whitelist: whitelist,
  };

  // --- Register subnet for activation
  assert_ok!(
    Network::register_subnet(
      RuntimeOrigin::signed(account(0)),
      add_subnet_data,
    )
  );

  let subnet_id = SubnetPaths::<Test>::get(subnet_path.clone()).unwrap();
  let subnet = SubnetsData::<Test>::get(subnet_id).unwrap();
  let owner = SubnetOwner::<Test>::get(subnet_id).unwrap();
  assert_eq!(owner, account(0));

  let epoch_length = EpochLength::get();
  let epoch = System::block_number() / epoch_length;

  // --- Add subnet nodes
  let block_number = System::block_number();
  let mut amount_staked = 0;
  for n in start+1..end+1 {
    let _ = Balances::deposit_creating(&account(n), deposit_amount);
    amount_staked += amount;
    assert_ok!(
      Network::add_subnet_node(
        RuntimeOrigin::signed(account(n)),
        subnet_id,
        account(n),
        peer(n),
        peer(n),
        0,
        amount,
        None,
        None,
        None,
      ) 
    );

    // assert_eq!(
    //   *network_events().last().unwrap(),
    //   Event::SubnetNodeActivated {
    //     subnet_id: subnet_id, 
    //     subnet_node_id: n
    //   }
    // ); 

    let hotkey_subnet_node_id = HotkeySubnetNodeId::<Test>::get(subnet_id, account(n)).unwrap();

    let subnet_node_id_hotkey = SubnetNodeIdHotkey::<Test>::get(subnet_id, hotkey_subnet_node_id).unwrap();
    assert_eq!(subnet_node_id_hotkey, account(n));

    let subnet_node_data = SubnetNodesData::<Test>::try_get(subnet_id, hotkey_subnet_node_id).unwrap();
    assert_eq!(subnet_node_data.hotkey, account(n));

    let key_owner = HotkeyOwner::<Test>::get(subnet_node_data.hotkey.clone());
    assert_eq!(key_owner, account(n));

    assert_eq!(subnet_node_data.peer_id, peer(n));

    // --- Is ``Validator`` if registered before subnet activation
    assert_eq!(subnet_node_data.classification.class, SubnetNodeClass::Validator);
    assert!(subnet_node_data.has_classification(&SubnetNodeClass::Validator, epoch));

    let subnet_node_account = PeerIdSubnetNode::<Test>::get(subnet_id, peer(n));
    assert_eq!(subnet_node_account, hotkey_subnet_node_id);

    let account_subnet_stake = AccountSubnetStake::<Test>::get(account(n), subnet_id);
    assert_eq!(account_subnet_stake, amount);
  }

  let total_subnet_stake = TotalSubnetStake::<Test>::get(subnet_id);
  assert_eq!(total_subnet_stake, amount_staked);

  let total_stake = TotalStake::<Test>::get();
  assert_eq!(total_subnet_stake, amount_staked);


  let delegate_staker_account = 1000;
  // Add 100e18 to account for block increase on activation
  let min_subnet_delegate_stake = Network::get_min_subnet_delegate_stake_balance() + 100e+18 as u128;
  let _ = Balances::deposit_creating(&account(delegate_staker_account), min_subnet_delegate_stake+500);
  // --- Add the minimum required delegate stake balance to activate the subnet
  assert_ok!(
    Network::add_to_delegate_stake(
      RuntimeOrigin::signed(account(delegate_staker_account)),
      subnet_id,
      min_subnet_delegate_stake,
    ) 
  );

  let total_delegate_stake_balance = TotalSubnetDelegateStakeBalance::<Test>::get(subnet_id);
  assert_eq!(total_delegate_stake_balance, min_subnet_delegate_stake);

  // let delegate_shares = AccountSubnetDelegateStakeShares::<Test>::get(account(delegate_staker_account), subnet_id);
  // 1000 is for inflation attack mitigation
  // assert_eq!(min_subnet_delegate_stake - 1000, delegate_shares);

  // --- Increase epochs to max registration epoch
  let epochs = SubnetRegistrationEpochs::<Test>::get();
  increase_epochs(epochs + 1);

  assert_ok!(
    Network::activate_subnet(
      RuntimeOrigin::signed(account(0)),
      subnet_id,
    )
  );

  assert_eq!(
    *network_events().last().unwrap(),
    Event::SubnetActivated {
      subnet_id: subnet_id, 
    }
  );

  // --- Check validator chosen on activation
  // let next_epoch = System::block_number() / epoch_length + 1;
  // let validator = SubnetRewardsValidator::<Test>::get(subnet_id, next_epoch as u32);
  // assert!(validator != None, "Validator is None");
}

pub fn build_activated_subnet_with_delegator_rewards(
  subnet_path: Vec<u8>, 
  start: u32, 
  mut end: u32, 
  deposit_amount: u128, 
  amount: u128,
  delegate_reward_rate: u128,
) {
  let epoch_length = EpochLength::get();
  let block_number = System::block_number();
  let epoch = System::block_number().saturating_div(epoch_length);
  let next_registration_epoch = Network::get_next_registration_epoch(epoch);
  increase_epochs(next_registration_epoch.saturating_sub(epoch));

  let cost = Network::registration_cost(0);
  let _ = Balances::deposit_creating(&account(0), cost+1000);

  let min_nodes = MinSubnetNodes::<Test>::get();

  if end == 0 {
    end = min_nodes;
  }

  let whitelist = get_coldkey_whitelist(start, end);

  let add_subnet_data = RegistrationSubnetData {
    path: subnet_path.clone().into(),
    max_node_registration_epochs: 16,
    node_registration_interval: 0,
    node_activation_interval: 0,
    node_queue_period: 1,
    max_node_penalties: 3,
    coldkey_whitelist: whitelist,
  };

  // --- Register subnet for activation
  assert_ok!(
    Network::register_subnet(
      RuntimeOrigin::signed(account(0)),
      add_subnet_data,
    )
  );

  let subnet_id = SubnetPaths::<Test>::get(subnet_path.clone()).unwrap();
  let subnet = SubnetsData::<Test>::get(subnet_id).unwrap();
  let owner = SubnetOwner::<Test>::get(subnet_id).unwrap();
  assert_eq!(owner, account(0));

  let epoch_length = EpochLength::get();
  let epoch = System::block_number() / epoch_length;

  // --- Add subnet nodes
  let block_number = System::block_number();
  let mut amount_staked = 0;
  for n in start+1..end+1 {
    let _ = Balances::deposit_creating(&account(n), amount+500);
    amount_staked += amount;
    assert_ok!(
      Network::add_subnet_node(
        RuntimeOrigin::signed(account(n)),
        subnet_id,
        account(n),
        peer(n),
        peer(n),
        delegate_reward_rate,
        amount,
        None,
        None,
        None,
      ) 
    );

    
    let hotkey_subnet_node_id = HotkeySubnetNodeId::<Test>::get(subnet_id, account(n)).unwrap();

    let subnet_node_id_hotkey = SubnetNodeIdHotkey::<Test>::get(subnet_id, hotkey_subnet_node_id).unwrap();
    assert_eq!(subnet_node_id_hotkey, account(n));

    let subnet_node_data = SubnetNodesData::<Test>::try_get(subnet_id, hotkey_subnet_node_id).unwrap();
    assert_eq!(subnet_node_data.hotkey, account(n));
    assert_eq!(subnet_node_data.delegate_reward_rate, delegate_reward_rate);

    let key_owner = HotkeyOwner::<Test>::get(subnet_node_data.hotkey.clone());
    assert_eq!(key_owner, account(n));

    assert_eq!(subnet_node_data.peer_id, peer(n));

    // --- Is ``Validator`` if registered before subnet activation
    assert_eq!(subnet_node_data.classification.class, SubnetNodeClass::Validator);
    assert!(subnet_node_data.has_classification(&SubnetNodeClass::Validator, epoch));

    let subnet_node_account = PeerIdSubnetNode::<Test>::get(subnet_id, peer(n));
    assert_eq!(subnet_node_account, hotkey_subnet_node_id);

    let account_subnet_stake = AccountSubnetStake::<Test>::get(account(n), subnet_id);
    assert_eq!(account_subnet_stake, amount);
  }

  let total_subnet_stake = TotalSubnetStake::<Test>::get(subnet_id);
  assert_eq!(total_subnet_stake, amount_staked);

  let total_stake = TotalStake::<Test>::get();
  assert_eq!(total_subnet_stake, amount_staked);

  let delegate_staker_account = 1000;
  // Add 100e18 to account for block increase on activation
  let min_subnet_delegate_stake = Network::get_min_subnet_delegate_stake_balance() + 100e+18 as u128;
  let _ = Balances::deposit_creating(&account(delegate_staker_account), min_subnet_delegate_stake+500);
  // --- Add the minimum required delegate stake balance to activate the subnet
  assert_ok!(
    Network::add_to_delegate_stake(
      RuntimeOrigin::signed(account(delegate_staker_account)),
      subnet_id,
      min_subnet_delegate_stake,
    ) 
  );

  let total_delegate_stake_balance = TotalSubnetDelegateStakeBalance::<Test>::get(subnet_id);
  assert_eq!(total_delegate_stake_balance, min_subnet_delegate_stake);

  let delegate_shares = AccountSubnetDelegateStakeShares::<Test>::get(account(delegate_staker_account), subnet_id);
  // 1000 is for inflation attack mitigation
  // assert_eq!(min_subnet_delegate_stake - 1000, delegate_shares);

  // --- Increase epochs to max registration epoch
  let epochs = SubnetRegistrationEpochs::<Test>::get();
  increase_epochs(epochs + 1);
  
  assert_ok!(
    Network::activate_subnet(
      RuntimeOrigin::signed(account(0)),
      subnet_id,
    )
  );

  assert_eq!(
    *network_events().last().unwrap(),
    Event::SubnetActivated {
      subnet_id: subnet_id, 
    }
  );
}

pub fn get_coldkey_whitelist(start: u32, end: u32) -> BTreeSet<AccountId> {
  let mut whitelist = BTreeSet::new();
  for n in start+1..end+1 {
    whitelist.insert(account(n));
  }
  whitelist
}

// Returns total staked on subnet
pub fn build_subnet_nodes(subnet_id: u32, start: u32, end: u32, deposit_amount: u128, amount: u128) -> u128 {
  let mut amount_staked = 0;
  for n in start+1..end+1 {
    let _ = Balances::deposit_creating(&account(n), deposit_amount);
    amount_staked += amount;
    assert_ok!(
      Network::add_subnet_node(
        RuntimeOrigin::signed(account(n)),
        subnet_id,
        account(n),
        peer(n),
        peer(n),
        0,
        amount,
        None,
        None,
        None,
      ) 
    );
    post_successful_add_subnet_node_asserts(n, subnet_id, amount);
  }
  amount_staked
}

pub fn post_subnet_removal_ensures(subnet_id: u32, path: Vec<u8>, start: u32, end: u32) {
  assert_eq!(SubnetsData::<Test>::try_get(subnet_id), Err(()));
  assert_eq!(SubnetPaths::<Test>::try_get(path), Err(()));
  assert_eq!(LastSubnetRegistration::<Test>::try_get(subnet_id), Err(()));
  assert_eq!(SubnetRegistrationEpoch::<Test>::try_get(subnet_id), Err(()));
  assert_eq!(SubnetRegistrationColdkeyWhitelist::<Test>::try_get(subnet_id), Err(()));
  assert_eq!(SubnetNodesData::<Test>::iter_prefix(subnet_id).count(), 0);
  assert_eq!(TotalSubnetNodes::<Test>::contains_key(subnet_id), false);
  assert_eq!(TotalSubnetNodeUids::<Test>::contains_key(subnet_id), false);
  assert_eq!(PeerIdSubnetNode::<Test>::iter_prefix(subnet_id).count(), 0);
  assert_eq!(BootstrapPeerIdSubnetNode::<Test>::iter_prefix(subnet_id).count(), 0);
  assert_eq!(SubnetNodeUniqueParam::<Test>::iter_prefix(subnet_id).count(), 0);
  assert_eq!(HotkeySubnetNodeId::<Test>::iter_prefix(subnet_id).count(), 0);
  assert_eq!(SubnetNodeIdHotkey::<Test>::iter_prefix(subnet_id).count(), 0);
  assert_eq!(SubnetPenaltyCount::<Test>::contains_key(subnet_id), false);
  assert_eq!(SubnetRewardsValidator::<Test>::iter_prefix(subnet_id).count(), 0);
  assert_eq!(SubnetRewardsSubmission::<Test>::iter_prefix(subnet_id).count(), 0);
  assert_eq!(Proposals::<Test>::iter_prefix(subnet_id).count(), 0);
  assert_eq!(SubnetNodeNonUniqueParamLastSet::<Test>::iter_prefix(subnet_id).count(), 0);
  assert_eq!(SubnetNodePenalties::<Test>::iter_prefix(subnet_id).count(), 0);
  assert_eq!(SubnetNodeRegistrationInterval::<Test>::contains_key(subnet_id), false);
  

  for n in start+1..end+1 {
    assert_eq!(HotkeySubnetNodeId::<Test>::get(subnet_id, account(n)), None);
    assert_eq!(PeerIdSubnetNode::<Test>::try_get(subnet_id, peer(n)), Err(()));
  
    let stake_balance = AccountSubnetStake::<Test>::get(account(n), subnet_id);
    assert_ok!(
      Network::remove_stake(
        RuntimeOrigin::signed(account(n)),
        subnet_id,
        account(n),
        stake_balance,
      ) 
    );

    let delegate_shares = AccountSubnetDelegateStakeShares::<Test>::get(account(n), subnet_id);
    if delegate_shares != 0 {
      // increase epoch becuse must have only one unstaking per epoch
      increase_epochs(1);

      assert_ok!(
        Network::remove_delegate_stake(
          RuntimeOrigin::signed(account(n)),
          subnet_id,
          delegate_shares,
        )
      );  
    }
  }

  let epoch_length = EpochLength::get();
  let stake_cooldown_epochs = StakeCooldownEpochs::get();

  let starting_block_number = System::block_number();

  

  // --- Ensure unstaking is stable
  for n in start+1..end+1 {
    System::set_block_number(System::block_number() + ((epoch_length  + 1) * stake_cooldown_epochs));
    let starting_balance = Balances::free_balance(&account(n));
    let unbondings = StakeUnbondingLedger::<Test>::get(account(n));
    // assert_eq!(unbondings.len(), 1);
    // let (ledger_epoch, ledger_balance) = unbondings.iter().next().unwrap();
    let ledger_balance: u128 = unbondings.values().copied().sum();
    assert_ok!(
      Network::claim_unbondings(
        RuntimeOrigin::signed(account(n)),
      )
    );
    let ending_balance = Balances::free_balance(&account(n));
    assert_eq!(starting_balance + ledger_balance, ending_balance);
    System::set_block_number(starting_block_number);
  }
}

// pub fn build_for_submit_consensus_data(subnet_id: u32, start: u32, end: u32, start_data: u32, end_data: u32) {
//   let subnet_node_data_vec = subnet_node_data(start_data, end_data);

//   for n in start+1..end+1 {
//     assert_ok!(
//       Network::submit_consensus_data(
//         RuntimeOrigin::signed(account(n)),
//         subnet_id,
//         subnet_node_data_vec.clone(),
//       ) 
//     );
//   }
// }

pub fn increase_epochs(epochs: u32) {
  if epochs == 0 {
    return
  }

  let block = System::block_number();

  let epoch_length = EpochLength::get();

  let next_epoch_start_block = (epoch_length * epochs) + block - (block % (epoch_length * epochs));
  System::set_block_number(next_epoch_start_block);
}

pub fn set_epoch(epoch: u32) {
  let epoch_length = EpochLength::get();
  System::set_block_number(epoch * epoch_length);
}

pub fn get_epoch() -> u32 {
  let current_block = System::block_number();
  let epoch_length: u32 = EpochLength::get();
  current_block.saturating_div(epoch_length)
}

pub fn make_subnet_submittable() {
  // increase blocks
  // let epoch_length = Network::EpochLength::get();
  // let epoch_length = EpochLength::get();
  

  // let min_required_subnet_consensus_submit_epochs: u32 = MinRequiredSubnetConsensusSubmitEpochs::<Test>::get();
  // System::set_block_number(System::block_number() + epoch_length * min_required_subnet_consensus_submit_epochs);
}

// // increase the blocks past the consensus steps and remove subnet peer blocks span
// pub fn make_consensus_data_submittable() {
//   // increase blocks
//   let current_block_number = System::block_number();
//   // let subnet_node_removal_percentage = RemoveSubnetNodeEpochPercentage::<Test>::get();
//   let epoch_length = EpochLength::get();

//   let start_block_can_remove_peer = epoch_length as u128 * subnet_node_removal_percentage / PERCENTAGE_FACTOR;

//   let max_remove_subnet_node_block = start_block_can_remove_peer + (current_block_number - (current_block_number % epoch_length));

//   if current_block_number < max_remove_subnet_node_block {
//     System::set_block_number(max_remove_subnet_node_block + 1);
//   }
// }

// pub fn make_subnet_node_included() {
//   let epoch_length = EpochLength::get();
// 	let epochs = SubnetNodeClassEpochs::<Test>::get(SubnetNodeClass::Included);
//   System::set_block_number(System::block_number() + epoch_length * epochs);
// }

// pub fn make_subnet_node_consensus_data_submittable() {
//   // increase blocks
//   let epoch_length = EpochLength::get();
// 	let epochs = SubnetNodeClassEpochs::<Test>::get(SubnetNodeClass::Validator);
//   System::set_block_number(System::block_number() + epoch_length * epochs);
//   // make_consensus_data_submittable();
// }

// pub fn make_subnet_node_dishonesty_consensus_proposable() {
//   // increase blocks
//   let epoch_length = EpochLength::get();
// 	let epochs = SubnetNodeClassEpochs::<Test>::get(SubnetNodeClass::Accountant);
//   System::set_block_number(System::block_number() + epoch_length * epochs);
// }

pub fn subnet_node_data(start: u32, end: u32) -> Vec<SubnetNodeData> {
  // initialize peer consensus data array
  let mut subnet_node_data: Vec<SubnetNodeData> = Vec::new();
  for n in start+1..end+1 {
    // let peer_subnet_node_data: SubnetNodeData<<Test as frame_system::Config>::AccountId> = SubnetNodeData {
    //   // account_id: account(n),
    //   peer_id: peer(n),
    //   score: DEFAULT_SCORE,
    // };
    let peer_subnet_node_data: SubnetNodeData = SubnetNodeData {
      peer_id: peer(n),
      score: DEFAULT_SCORE,
    };

    subnet_node_data.push(peer_subnet_node_data);
  }
  subnet_node_data
}

pub fn subnet_node_data_invalid_scores(start: u32, end: u32) -> Vec<SubnetNodeData> {
  // initialize peer consensus data array
  // let mut subnet_node_data: Vec<SubnetNodeData<<Test as frame_system::Config>::AccountId>> = Vec::new();
  let mut subnet_node_data: Vec<SubnetNodeData> = Vec::new();
  for n in start+1..end+1 {
    // let peer_subnet_node_data: SubnetNodeData<<Test as frame_system::Config>::AccountId> = SubnetNodeData {
    //   // account_id: account(n),
    //   peer_id: peer(n),
    //   score: 10000000000,
    // };
    let peer_subnet_node_data: SubnetNodeData = SubnetNodeData {
      peer_id: peer(n),
      score: 10000000000,
    };
    subnet_node_data.push(peer_subnet_node_data);
  }
  subnet_node_data
}

pub fn post_successful_add_subnet_node_asserts(
  n: u32, 
  subnet_id: u32, 
  amount: u128
) {
  assert_eq!(Network::account_subnet_stake(account(n), subnet_id), amount);
  // assert_eq!(Network::total_account_stake(account(n)), amount);    
  assert_eq!(Network::total_subnet_nodes(subnet_id), (n + 1) as u32);
}

// check data after adding multiple peers
// each peer must have equal staking amount per subnet
pub fn post_successful_add_subnet_nodes_asserts(
  total_peers: u32,
  stake_per_peer: u128,  
  subnet_id: u32, 
) {
  let amount_staked = total_peers as u128 * stake_per_peer;
  assert_eq!(Network::total_subnet_stake(subnet_id), amount_staked);
}

pub fn post_remove_subnet_node_ensures(n: u32, subnet_id: u32) {
  // ensure SubnetNodesData removed
  let subnet_node_id = HotkeySubnetNodeId::<Test>::try_get(subnet_id, account(n));
  assert_eq!(subnet_node_id, Err(()));

  assert_eq!(SubnetNodesData::<Test>::iter_prefix(subnet_id).count(), 0);
  // assert_eq!(subnet_node_hotkey, Err(()));

  // ensure PeerIdSubnetNode removed
  let subnet_node_account = PeerIdSubnetNode::<Test>::try_get(subnet_id, peer(n));
  assert_eq!(subnet_node_account, Err(()));
}

pub fn post_remove_unstake_ensures(n: u32, subnet_id: u32) {
}

pub fn add_subnet_node(
  account_id: u32, 
  subnet_id: u32,
  peer_id: u32,
  ip: String,
  port: u16,
  amount: u128
) -> Result<(), sp_runtime::DispatchError> {
  Network::add_subnet_node(
    RuntimeOrigin::signed(account(account_id)),
    subnet_id,
    account(account_id),
    peer(peer_id),
    peer(peer_id),
    0,
    amount,
    None,
    None,
    None,
  )
}
