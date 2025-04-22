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
  TotalStake, 
  SubnetRewardsValidator,
  SubnetPaths, 
  TotalSubnetNodes,
  SubnetNodeClass,
  SubnetsData,
  AccountSubnetStake,
  RegistrationSubnetData,
  StakeUnbondingLedger, 
  TotalSubnetStake, 
  MinSubnetRegistrationBlocks,
  DefaultSubnetNodeUniqueParamLimit,
  HotkeyOwner, 
  TotalSubnetNodeUids, 
  HotkeySubnetNodeId, 
  SubnetNodeIdHotkey, 
  SubnetNodesData, 
  PeerIdSubnetNode,
  DeactivationLedger, 
  SubnetNodeDeactivation, 
  MaxRewardRateDecrease,
  RewardRateUpdatePeriod,
  SubnetRegistrationEpochs,
  MinStakeBalance,
  RegisteredStakeCooldownEpochs,
};

///
///
///
///
///
///
///
/// Subnet Nodes Add/Remove
///
///
///
///
///
///
///

#[test]
fn test_register_subnet_node() {
  new_test_ext().execute_with(|| {
    let subnet_path: Vec<u8> = "petals-team/StableBeluga2".into();
    
    let deposit_amount: u128 = 10000000000000000000000;
    let amount: u128 = 1000000000000000000000;

    let stake_amount: u128 = MinStakeBalance::<Test>::get();

    build_activated_subnet(subnet_path.clone(), 0, 0, deposit_amount, stake_amount);

    let subnet_id = SubnetPaths::<Test>::get(subnet_path.clone()).unwrap();
    let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);

    let _ = Balances::deposit_creating(&account(total_subnet_nodes+1), deposit_amount);

    assert_ok!(
      Network::register_subnet_node(
        RuntimeOrigin::signed(account(total_subnet_nodes+1)),
        subnet_id,
        account(total_subnet_nodes+1),
        peer(total_subnet_nodes+1),
        peer(total_subnet_nodes+1),
        0,
        amount,
        None,
        None,
        None,
      )
    );

    let total_subnet_node_uids = TotalSubnetNodeUids::<Test>::get(subnet_id);
    let hotkey_subnet_node_id = HotkeySubnetNodeId::<Test>::get(subnet_id, account(total_subnet_nodes+1)).unwrap();
    assert_eq!(total_subnet_node_uids, hotkey_subnet_node_id);

    let new_total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);
    assert_eq!(new_total_subnet_nodes, total_subnet_nodes + 1);

    let subnet_node = SubnetNodesData::<Test>::get(subnet_id, hotkey_subnet_node_id);
    // assert_eq!(subnet_node.coldkey, account(total_subnet_nodes+1));
    assert_eq!(subnet_node.hotkey, account(total_subnet_nodes+1));
    assert_eq!(subnet_node.peer_id, peer(total_subnet_nodes+1));
    assert_eq!(subnet_node.classification.class, SubnetNodeClass::Registered);

    let subnet_node_account = PeerIdSubnetNode::<Test>::get(subnet_id, peer(total_subnet_nodes+1));
    assert_eq!(subnet_node_account, hotkey_subnet_node_id);

    let account_subnet_stake = AccountSubnetStake::<Test>::get(account(total_subnet_nodes+1), subnet_id);
    assert_eq!(account_subnet_stake, amount);
  })
}

#[test]
fn test_update_coldkey() {
  new_test_ext().execute_with(|| {
    let subnet_path: Vec<u8> = "petals-team/StableBeluga2".into();
    
    let deposit_amount: u128 = 10000000000000000000000;
    let amount: u128 = 1000000000000000000000;

    let stake_amount: u128 = MinStakeBalance::<Test>::get();

    build_activated_subnet(subnet_path.clone(), 0, 16, deposit_amount, stake_amount);

    let subnet_id = SubnetPaths::<Test>::get(subnet_path.clone()).unwrap();
    let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);

    let hotkey_subnet_node_id = HotkeySubnetNodeId::<Test>::get(subnet_id, account(1)).unwrap();
    let starting_account_subnet_stake = AccountSubnetStake::<Test>::get(account(1), subnet_id);

    // add extra stake and then add to ledger to check if it swapped
    let add_stake_amount = 1000000000000000000000;
    let _ = Balances::deposit_creating(&account(1), deposit_amount);

    //
    //
    // Coldkey = 1
    // Hotkey  = 1
    //
    //

    assert_ok!(
      Network::add_to_stake(
        RuntimeOrigin::signed(account(1)),
        subnet_id,
        hotkey_subnet_node_id,
        account(1),
        add_stake_amount,
      )
    );

    let stake_balance = AccountSubnetStake::<Test>::get(&account(1), subnet_id);
    assert_eq!(stake_balance, starting_account_subnet_stake + add_stake_amount);

    assert_ok!(
      Network::remove_stake(
        RuntimeOrigin::signed(account(1)),
        subnet_id,
        account(1),
        amount,
      )
    );

    let original_unbondings: BTreeMap<u32, u128> = StakeUnbondingLedger::<Test>::get(account(1));
    let original_ledger_balance: u128 = original_unbondings.values().copied().sum();
    assert_eq!(original_unbondings.len() as u32, 1);  
    assert_eq!(original_ledger_balance, amount);  

    /// Update the coldkey to unused key
    //
    //
    // Coldkey = total_subnet_nodes+1
    // Hotkey  = 1
    //
    //

    assert_ok!(
      Network::update_coldkey(
        RuntimeOrigin::signed(account(1)),
        account(1),
        account(total_subnet_nodes+1),
      )
    );

    // check old coldkey balance is now removed because it was swapped to the new one
    let unbondings: BTreeMap<u32, u128> = StakeUnbondingLedger::<Test>::get(account(1));
    let ledger_balance: u128 = unbondings.values().copied().sum();
    assert_eq!(unbondings.len() as u32, 0);  
    assert_eq!(ledger_balance, 0);  

    // check new coldkey balance matches original
    let new_unbondings: BTreeMap<u32, u128> = StakeUnbondingLedger::<Test>::get(account(total_subnet_nodes+1));
    let new_ledger_balance: u128 = new_unbondings.values().copied().sum();
    assert_eq!(new_unbondings.len() as u32, original_unbondings.len() as u32);  
    assert_eq!(new_ledger_balance, original_ledger_balance);  

    let subnet_node_id_hotkey = SubnetNodeIdHotkey::<Test>::get(subnet_id, hotkey_subnet_node_id).unwrap();
    assert_eq!(subnet_node_id_hotkey, account(1));

    let subnet_node_data = SubnetNodesData::<Test>::try_get(subnet_id, hotkey_subnet_node_id).unwrap();
    assert_eq!(subnet_node_data.hotkey, account(1));

    let key_owner = HotkeyOwner::<Test>::get(account(1));
    assert_eq!(key_owner, account(total_subnet_nodes+1));

    // Cold key is updated, shouldn't be able to make changes anywhere using coldkey

    let add_stake_amount: u128 = 1000000000000000000000;
    let _ = Balances::deposit_creating(&account(1), add_stake_amount);

    assert_err!(
      Network::add_to_stake(
        RuntimeOrigin::signed(account(1)),
        subnet_id,
        hotkey_subnet_node_id,
        account(1),
        add_stake_amount,
      ),
      Error::<Test>::NotKeyOwner,
    );

    assert_err!(
      Network::remove_stake(
        RuntimeOrigin::signed(account(1)),
        subnet_id,
        account(1),
        1000,
      ),
      Error::<Test>::NotKeyOwner
    );    
    
    // `do_deactivate_subnet_node` allows both hotkey and coldkey
    assert_err!(
      Network::do_deactivate_subnet_node(
        RuntimeOrigin::signed(account(2)),
        subnet_id,
        hotkey_subnet_node_id
      ),
      Error::<Test>::NotKeyOwner
    );

    assert_err!(
      Network::update_coldkey(
        RuntimeOrigin::signed(account(1)),
        account(2),
        account(total_subnet_nodes+1),
      ),
      Error::<Test>::NotKeyOwner
    );

    assert_err!(
      Network::update_hotkey(
        RuntimeOrigin::signed(account(1)),
        account(2),
        account(total_subnet_nodes+1),
      ),
      Error::<Test>::NotKeyOwner
    );


    // Use new coldkey
    let add_stake_amount: u128 = 1000000000000000000000;
    let _ = Balances::deposit_creating(&account(total_subnet_nodes+1), add_stake_amount + 500);

    assert_ok!(
      Network::add_to_stake(
        RuntimeOrigin::signed(account(total_subnet_nodes+1)),
        subnet_id,
        hotkey_subnet_node_id,
        account(1),
        add_stake_amount,
      )
    );

    assert_ok!(
      Network::remove_stake(
        RuntimeOrigin::signed(account(total_subnet_nodes+1)),
        subnet_id,
        account(1),
        add_stake_amount,
      )
    );

    // `do_deactivate_subnet_node` allows both hotkey and coldkey
    assert_ok!(
      Network::do_deactivate_subnet_node(
        RuntimeOrigin::signed(account(total_subnet_nodes+1)),
        subnet_id,
        hotkey_subnet_node_id
      )
    );

    assert_ok!(
      Network::update_hotkey(
        RuntimeOrigin::signed(account(total_subnet_nodes+1)),
        account(1),
        account(total_subnet_nodes+15),
      )
    );

    assert_ok!(
      Network::update_coldkey(
        RuntimeOrigin::signed(account(total_subnet_nodes+1)),
        account(total_subnet_nodes+15),
        account(total_subnet_nodes+2),
      )
    );

    assert_err!(
      Network::update_coldkey(
        RuntimeOrigin::signed(account(total_subnet_nodes+1)),
        account(total_subnet_nodes+15),
        account(total_subnet_nodes+2),
      ),
      Error::<Test>::NotKeyOwner
    );    
  })
}

#[test]
fn test_update_coldkey_key_taken_err() {
  new_test_ext().execute_with(|| {
    let subnet_path: Vec<u8> = "petals-team/StableBeluga2".into();
    let deposit_amount: u128 = 10000000000000000000000;
    let amount: u128 = 1000000000000000000000;

    let n_peers = 8;
    let stake_amount: u128 = MinStakeBalance::<Test>::get();

    build_activated_subnet(subnet_path.clone(), 0, n_peers, deposit_amount, stake_amount);

    let subnet_id = SubnetPaths::<Test>::get(subnet_path.clone()).unwrap();
    let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);

    assert_err!(
      Network::update_coldkey(
        RuntimeOrigin::signed(account(1)),
        account(2),
        account(1),
      ),
      Error::<Test>::NotKeyOwner
    );
  });
}

#[test]
fn test_update_hotkey() {
  new_test_ext().execute_with(|| {
    let subnet_path: Vec<u8> = "petals-team/StableBeluga2".into();
    
    let deposit_amount: u128 = 10000000000000000000000;
    let amount: u128 = 1000000000000000000000;

    let stake_amount: u128 = MinStakeBalance::<Test>::get();

    build_activated_subnet(subnet_path.clone(), 0, 0, deposit_amount, stake_amount);

    let subnet_id = SubnetPaths::<Test>::get(subnet_path.clone()).unwrap();
    let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);

    let hotkey_subnet_node_id = HotkeySubnetNodeId::<Test>::get(subnet_id, account(1)).unwrap();
    let starting_account_subnet_stake = AccountSubnetStake::<Test>::get(account(1), subnet_id);

    assert_ok!(
      Network::update_hotkey(
        RuntimeOrigin::signed(account(1)),
        account(1),
        account(total_subnet_nodes+1),
      )
    );

    let subnet_node_id_hotkey = SubnetNodeIdHotkey::<Test>::get(subnet_id, hotkey_subnet_node_id).unwrap();
    assert_eq!(subnet_node_id_hotkey, account(total_subnet_nodes+1));

    let subnet_node_data = SubnetNodesData::<Test>::try_get(subnet_id, hotkey_subnet_node_id).unwrap();
    assert_eq!(subnet_node_data.hotkey, account(total_subnet_nodes+1));

    let key_owner = HotkeyOwner::<Test>::get(account(total_subnet_nodes+1));
    assert_eq!(key_owner, account(1));

    let account_subnet_stake = AccountSubnetStake::<Test>::get(account(1), subnet_id);
    assert_eq!(account_subnet_stake, 0);

    let account_subnet_stake = AccountSubnetStake::<Test>::get(account(total_subnet_nodes+1), subnet_id);
    assert_eq!(account_subnet_stake, starting_account_subnet_stake);
  })
}

#[test]
fn test_register_subnet_node_subnet_registering_or_activated_error() {
  new_test_ext().execute_with(|| {
    let _ = env_logger::builder().is_test(true).try_init();

    let deposit_amount: u128 = 10000000000000000000000;
    let amount: u128 = 1000000000000000000000;

    let epoch_length = EpochLength::get();
    let block_number = System::block_number();
    let epoch = System::block_number().saturating_div(epoch_length);
  
    let cost = Network::registration_cost(epoch);
  
    let _ = Balances::deposit_creating(&account(1), cost+1000);
  
    let subnet_path: Vec<u8> = "petals-team/StableBeluga2".into();

    let whitelist = get_coldkey_whitelist(0, 1);

    let add_subnet_data = RegistrationSubnetData {
      path: subnet_path.clone().into(),
      max_node_registration_epochs: 16,
      node_registration_interval: 0,
      node_activation_interval: 0,
      node_queue_period: 1,
      max_node_penalties: 3,
      coldkey_whitelist: whitelist,
      // coldkey_whitelist: None,
    };
  
    let epoch_length = EpochLength::get();
    let block_number = System::block_number();
    let epoch = System::block_number().saturating_div(epoch_length);
    let next_registration_epoch = Network::get_next_registration_epoch(epoch);
    increase_epochs(next_registration_epoch - epoch);

    // --- Register subnet for activation
    assert_ok!(
      Network::register_subnet(
        RuntimeOrigin::signed(account(1)),
        add_subnet_data,
      )
    );
  
    let subnet_id = SubnetPaths::<Test>::get(subnet_path.clone()).unwrap();
    let subnet = SubnetsData::<Test>::get(subnet_id).unwrap();
    
    // push out of registration period and into enactment period
    let epochs = SubnetRegistrationEpochs::<Test>::get();
    increase_epochs(epochs + 1);

    assert_err!(
      Network::register_subnet_node(
        RuntimeOrigin::signed(account(1)),
        subnet_id,
        account(1),
        peer(1),
        peer(1),
        0,
        amount,
        None,
        None,
        None,
      ),
      Error::<Test>::SubnetMustBeRegisteringOrActivated
    );
  })
}

#[test]
fn test_register_subnet_node_then_activate() {
  new_test_ext().execute_with(|| {

    let deposit_amount: u128 = 10000000000000000000000;
    let amount: u128 = 1000000000000000000000;

    let epoch_length = EpochLength::get();
    let block_number = System::block_number();
    let epoch = System::block_number().saturating_div(epoch_length);
  
    let cost = Network::registration_cost(epoch);
  
    let _ = Balances::deposit_creating(&account(1), cost+deposit_amount);
  
    let subnet_path: Vec<u8> = "petals-team/StableBeluga2".into();

    let whitelist = get_coldkey_whitelist(0, 1);

    let add_subnet_data = RegistrationSubnetData {
      path: subnet_path.clone().into(),
      max_node_registration_epochs: 16,
      node_registration_interval: 0,
      node_activation_interval: 0,
      node_queue_period: 1,
      max_node_penalties: 3,
      coldkey_whitelist: whitelist,
    };
  
    let epoch_length = EpochLength::get();
    let block_number = System::block_number();
    let epoch = System::block_number().saturating_div(epoch_length);
    let next_registration_epoch = Network::get_next_registration_epoch(epoch);
    increase_epochs(next_registration_epoch - epoch);

    // --- Register subnet for activation
    assert_ok!(
      Network::register_subnet(
        RuntimeOrigin::signed(account(1)),
        add_subnet_data,
      )
    );
  
    let subnet_id = SubnetPaths::<Test>::get(subnet_path.clone()).unwrap();
    let subnet = SubnetsData::<Test>::get(subnet_id).unwrap();
      
    assert_ok!(
      Network::register_subnet_node(
        RuntimeOrigin::signed(account(1)),
        subnet_id,
        account(1),
        peer(1),
        peer(1),
        0,
        amount,
        None,
        None,
        None,
      ),
    );

    let subnet_node_id = HotkeySubnetNodeId::<Test>::get(subnet_id, account(1)).unwrap();

    assert_ok!(
      Network::activate_subnet_node(
        RuntimeOrigin::signed(account(1)),
        subnet_id,
        subnet_node_id
      ),
    );
  })
}

#[test]
fn test_activate_subnet_then_register_subnet_node_then_activate() {
  new_test_ext().execute_with(|| {
    let subnet_path: Vec<u8> = "petals-team/StableBeluga2".into();
    
    let deposit_amount: u128 = 10000000000000000000000;
    let amount: u128 = 1000000000000000000000;

    let stake_amount: u128 = MinStakeBalance::<Test>::get();

    build_activated_subnet(subnet_path.clone(), 0, 0, deposit_amount, stake_amount);

    let subnet_id = SubnetPaths::<Test>::get(subnet_path.clone()).unwrap();
    let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);
    let n_account = total_subnet_nodes + 1;
       
    let _ = Balances::deposit_creating(&account(n_account), deposit_amount);

    assert_ok!(
      Network::register_subnet_node(
        RuntimeOrigin::signed(account(n_account)),
        subnet_id,
        account(n_account),
        peer(n_account),
        peer(n_account),
        0,
        amount,
        None,
        None,
        None,
      ),
    );

    let subnet_node_id = HotkeySubnetNodeId::<Test>::get(subnet_id, account(n_account)).unwrap();

    assert_ok!(
      Network::activate_subnet_node(
        RuntimeOrigin::signed(account(n_account)),
        subnet_id,
        subnet_node_id,
      ),
    );
  })
}

#[test]
fn test_activate_subnet_node_subnet_registering_or_activated_error() {
  new_test_ext().execute_with(|| {

    let deposit_amount: u128 = 10000000000000000000000;
    let amount: u128 = 1000000000000000000000;

    let epoch_length = EpochLength::get();
    let block_number = System::block_number();
    let epoch = System::block_number().saturating_div(epoch_length);
  
    let cost = Network::registration_cost(epoch);
  
    let _ = Balances::deposit_creating(&account(1), cost+1000+deposit_amount);
  
    let subnet_path: Vec<u8> = "petals-team/StableBeluga2".into();

    let whitelist = get_coldkey_whitelist(0, 1);

    let add_subnet_data = RegistrationSubnetData {
      path: subnet_path.clone().into(),
      max_node_registration_epochs: 16,
      node_registration_interval: 0,
      node_activation_interval: 0,
      node_queue_period: 1,
      max_node_penalties: 3,
      coldkey_whitelist: whitelist,
      // coldkey_whitelist: None,
    };
  
    let epoch_length = EpochLength::get();
    let block_number = System::block_number();
    let epoch = System::block_number().saturating_div(epoch_length);
    let next_registration_epoch = Network::get_next_registration_epoch(epoch);
    increase_epochs(next_registration_epoch - epoch);

    // --- Register subnet for activation
    assert_ok!(
      Network::register_subnet(
        RuntimeOrigin::signed(account(1)),
        add_subnet_data,
      )
    );
  
    let subnet_id = SubnetPaths::<Test>::get(subnet_path.clone()).unwrap();
    let subnet = SubnetsData::<Test>::get(subnet_id).unwrap();
  
    assert_ok!(
      Network::register_subnet_node(
        RuntimeOrigin::signed(account(1)),
        subnet_id,
        account(1),
        peer(1),
        peer(1),
        0,
        amount,
        None,
        None,
        None,
      )
    );

    let subnet_node_id = HotkeySubnetNodeId::<Test>::get(subnet_id, account(1)).unwrap();

    // assert_err!(
    //   Network::activate_subnet_node(
    //     RuntimeOrigin::signed(account(1)),
    //     subnet_id,
    //     subnet_node_id,
    //   ),
    //   Error::<Test>::SubnetMustBeRegisteringOrActivated
    // );
  })
}


#[test]
fn test_register_subnet_node_activate_subnet_node() {
  new_test_ext().execute_with(|| {
    let subnet_path: Vec<u8> = "petals-team/StableBeluga2".into();
    
    let deposit_amount: u128 = 10000000000000000000000;
    let amount: u128 = 1000000000000000000000;

    let stake_amount: u128 = MinStakeBalance::<Test>::get();

    build_activated_subnet(subnet_path.clone(), 0, 0, deposit_amount, stake_amount);

    let subnet_id = SubnetPaths::<Test>::get(subnet_path.clone()).unwrap();
    let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);

    let _ = Balances::deposit_creating(&account(total_subnet_nodes+1), deposit_amount);

    assert_ok!(
      Network::register_subnet_node(
        RuntimeOrigin::signed(account(total_subnet_nodes+1)),
        subnet_id,
        account(total_subnet_nodes+1),
        peer(total_subnet_nodes+1),
        peer(total_subnet_nodes+1),
        0,
        amount,
        None,
        None,
        None,
      )
    );

    let subnet_node_id = HotkeySubnetNodeId::<Test>::get(subnet_id, account(total_subnet_nodes+1)).unwrap();


    let new_total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);
    assert_eq!(new_total_subnet_nodes, total_subnet_nodes + 1);

    let subnet_node = SubnetNodesData::<Test>::get(subnet_id, subnet_node_id);
    assert_eq!(subnet_node.hotkey, account(total_subnet_nodes+1));
    assert_eq!(subnet_node.peer_id, peer(total_subnet_nodes+1));
    assert_eq!(subnet_node.classification.class, SubnetNodeClass::Registered);

    let subnet_node_account = PeerIdSubnetNode::<Test>::get(subnet_id, peer(total_subnet_nodes+1));
    assert_eq!(subnet_node_account, subnet_node_id);

    let account_subnet_stake = AccountSubnetStake::<Test>::get(account(total_subnet_nodes+1), subnet_id);
    assert_eq!(account_subnet_stake, amount);

    let epoch_length = EpochLength::get();
    let block_number = System::block_number();
    let epoch = System::block_number() / epoch_length;

    assert_ok!(
      Network::activate_subnet_node(
        RuntimeOrigin::signed(account(total_subnet_nodes+1)),
        subnet_id,
        subnet_node_id,
      )
    );

    let subnet_node = SubnetNodesData::<Test>::get(subnet_id, subnet_node_id);

    assert_eq!(subnet_node.classification.class, SubnetNodeClass::Queue);
    assert_eq!(subnet_node.classification.start_epoch, epoch + 1);
  })
}

#[test]
fn test_deactivate_subnet_node_reactivate() {
  new_test_ext().execute_with(|| {
    let subnet_path: Vec<u8> = "petals-team/StableBeluga2".into();
    
    let deposit_amount: u128 = 10000000000000000000000;
    let amount: u128 = 1000000000000000000000;

    let stake_amount: u128 = MinStakeBalance::<Test>::get();

    build_activated_subnet(subnet_path.clone(), 0, 0, deposit_amount, stake_amount);

    let subnet_id = SubnetPaths::<Test>::get(subnet_path.clone()).unwrap();
    let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);

    let subnet_node_id = HotkeySubnetNodeId::<Test>::get(subnet_id, account(1)).unwrap();

    let subnet_node = SubnetNodesData::<Test>::get(subnet_id, subnet_node_id);
    assert_eq!(subnet_node.classification.class, SubnetNodeClass::Validator);    

    let epoch_length = EpochLength::get();
    let epoch = System::block_number() / epoch_length;

    assert_ok!(
      Network::deactivate_subnet_node(
        RuntimeOrigin::signed(account(1)),
        subnet_id,
        subnet_node_id,
      )
    );

    let subnet_node = SubnetNodesData::<Test>::get(subnet_id, subnet_node_id);
    assert_eq!(subnet_node.classification.class, SubnetNodeClass::Deactivated);    
    assert_eq!(subnet_node.classification.start_epoch, epoch);    

    let epoch_length = EpochLength::get();
    let epoch = System::block_number() / epoch_length;

    assert_ok!(
      Network::activate_subnet_node(
        RuntimeOrigin::signed(account(1)),
        subnet_id,
        subnet_node_id,
      )
    );

    let subnet_node = SubnetNodesData::<Test>::get(subnet_id, subnet_node_id);
    assert_eq!(subnet_node.classification.class, SubnetNodeClass::Validator);    
    assert_eq!(subnet_node.classification.start_epoch, epoch + 1);    
  })
}

#[test]
fn test_add_subnet_node_subnet_err() {
  new_test_ext().execute_with(|| {
    let subnet_id = 0;

    let amount: u128 = 1000;
    assert_err!(
      Network::add_subnet_node(
        RuntimeOrigin::signed(account(1)),
        subnet_id,
        account(1),
        peer(1),
        peer(1),
        0,
        amount,
        None,
        None,
        None,
      ),
      Error::<Test>::SubnetNotExist
    );

    let subnet_id = 1;

    assert_err!(Network::add_subnet_node(
        RuntimeOrigin::signed(account(1)),
        subnet_id,
        account(1),
        peer(1),
        peer(1),
        0,
        amount,
        None,
        None,
        None,
      ),
      Error::<Test>::SubnetNotExist
    );
  })
}

#[test]
fn test_get_classification_subnet_nodes() {
  new_test_ext().execute_with(|| {
    let subnet_path: Vec<u8> = "petals-team/StableBeluga2".into();
    
    let deposit_amount: u128 = 10000000000000000000000;
    let amount: u128 = 1000000000000000000000;

    let stake_amount: u128 = MinStakeBalance::<Test>::get();

    build_activated_subnet(subnet_path.clone(), 0, 0, deposit_amount, stake_amount);

    let subnet_id = SubnetPaths::<Test>::get(subnet_path.clone()).unwrap();
    let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);
    let epoch_length = EpochLength::get();
    let epoch = System::block_number() / epoch_length;
  
    let submittable = Network::get_classified_subnet_nodes(subnet_id, &SubnetNodeClass::Validator, epoch);

    assert_eq!(submittable.len() as u32, total_subnet_nodes);
  })
}

#[test]
fn test_add_subnet_node_not_exists_err() {
  new_test_ext().execute_with(|| {
    let subnet_path: Vec<u8> = "petals-team/StableBeluga2".into();
    
    let deposit_amount: u128 = 10000000000000000000000;
    let amount: u128 = 1000000000000000000000;

    let stake_amount: u128 = MinStakeBalance::<Test>::get();

    build_activated_subnet(subnet_path.clone(), 0, 0, deposit_amount, stake_amount);

    let subnet_id = SubnetPaths::<Test>::get(subnet_path.clone()).unwrap();
    let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);

    // add new peer_id under same account error
    assert_err!(
      Network::add_subnet_node(
        RuntimeOrigin::signed(account(1)),
        subnet_id,
        account(1),
        peer(1),
        peer(1),
        0,
        amount,
        None,
        None,
        None,
      ),
      Error::<Test>::SubnetNodeExist
    );

    assert_eq!(Network::total_subnet_nodes(subnet_id), total_subnet_nodes);

    // add same peer_id under new account error
    assert_err!(
      Network::add_subnet_node(
        RuntimeOrigin::signed(account(total_subnet_nodes+1)),
        subnet_id,
        account(total_subnet_nodes+1),
        peer(1),
        peer(1),
        0,
        amount,
        None,
        None,
        None,
      ),
      Error::<Test>::PeerIdExist
    );

    assert_eq!(Network::total_subnet_nodes(subnet_id), total_subnet_nodes);

    // add new peer_id under same account error
    assert_err!(
      Network::add_subnet_node(
        RuntimeOrigin::signed(account(1)),
        subnet_id,
        account(1),
        peer(1),
        peer(1),
        0,
        amount,
        None,
        None,
        None,
      ),
      Error::<Test>::SubnetNodeExist
    );

    assert_eq!(Network::total_subnet_nodes(subnet_id), total_subnet_nodes);
  })
}

#[test]
fn test_add_subnet_node_stake_err() {
  new_test_ext().execute_with(|| {
    let subnet_path: Vec<u8> = "petals-team/StableBeluga2".into();
    
    let deposit_amount: u128 = 10000000000000000000000;
    let amount: u128 = 1000000000000000000000;

    let stake_amount: u128 = MinStakeBalance::<Test>::get();

    build_activated_subnet(subnet_path.clone(), 0, 0, deposit_amount, stake_amount);

    let deposit_amount: u128 = 100000;
    let amount: u128 = 1;

    let _ = Balances::deposit_creating(&account(1), deposit_amount);
    let subnet_id = SubnetPaths::<Test>::get(subnet_path.clone()).unwrap();
    let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);

    assert_err!(
      Network::add_subnet_node(
        RuntimeOrigin::signed(account(total_subnet_nodes+1)),
        subnet_id,
        account(total_subnet_nodes+1),
        peer(total_subnet_nodes+1),
        peer(total_subnet_nodes+1),
        0,
        amount,
        None,
        None,
        None,
      ),
      Error::<Test>::MinStakeNotReached
    );
  })
}

#[test]
fn test_add_subnet_node_stake_not_enough_balance_err() {
  new_test_ext().execute_with(|| {
    let subnet_path: Vec<u8> = "petals-team/StableBeluga2".into();

    let deposit_amount: u128 = 10000000000000000000000;
    let amount: u128 = 1000000000000000000000;

    let stake_amount: u128 = MinStakeBalance::<Test>::get();

    build_activated_subnet(subnet_path.clone(), 0, 0, deposit_amount, stake_amount);

    let deposit_amount: u128 = 999999999999999999999;

    let subnet_id = SubnetPaths::<Test>::get(subnet_path.clone()).unwrap();
    let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);
    let _ = Balances::deposit_creating(&account(total_subnet_nodes+1), deposit_amount);

    assert_err!(
      Network::add_subnet_node(
        RuntimeOrigin::signed(account(total_subnet_nodes+1)),
        subnet_id,
        account(total_subnet_nodes+1),
        peer(total_subnet_nodes+1),
        peer(total_subnet_nodes+1),
        0,
        amount,
        None,
        None,
        None,
      ),
      Error::<Test>::NotEnoughBalanceToStake
    );
  })
}

#[test]
fn test_add_subnet_node_invalid_peer_id_err() {
  new_test_ext().execute_with(|| {
    let subnet_path: Vec<u8> = "petals-team/StableBeluga2".into();

    let deposit_amount: u128 = 10000000000000000000000;
    let amount: u128 = 1000000000000000000000;
    let stake_amount: u128 = MinStakeBalance::<Test>::get();

    build_activated_subnet(subnet_path.clone(), 0, 0, deposit_amount, stake_amount);

    let subnet_id = SubnetPaths::<Test>::get(subnet_path.clone()).unwrap();
    let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);

    let _ = Balances::deposit_creating(&account(total_subnet_nodes+1), deposit_amount);

    let peer_id = format!("2");
    let peer: PeerId = PeerId(peer_id.clone().into());
    let bootstrap_peer: PeerId = PeerId(peer_id.clone().into());
    assert_err!(
      Network::add_subnet_node(
        RuntimeOrigin::signed(account(total_subnet_nodes+1)),
        subnet_id,
        account(total_subnet_nodes+1),
        peer,
        bootstrap_peer,
        0,
        amount,
        None,
        None,
        None,
      ),
      Error::<Test>::InvalidPeerId
    );
  })
}

// // #[test]
// // fn test_add_subnet_node_remove_readd_err() {
// //   new_test_ext().execute_with(|| {
// //     let subnet_path: Vec<u8> = "petals-team/StableBeluga2".into();

// //     let deposit_amount: u128 = 10000000000000000000000;
// //     let amount: u128 = 1000000000000000000000;
// //     build_activated_subnet(subnet_path.clone(), 0, 0, deposit_amount, amount);

// //     let subnet_id = SubnetPaths::<Test>::get(subnet_path.clone()).unwrap();
// //     let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);

// //     let _ = Balances::deposit_creating(&account(total_subnet_nodes+1), deposit_amount);

// //     System::set_block_number(System::block_number() + 1);

// //     assert_ok!(
// //       Network::add_subnet_node(
// //         RuntimeOrigin::signed(account(total_subnet_nodes+1)),
// // account(total_subnet_nodes+1),
// //         subnet_id,
// //         peer(total_subnet_nodes+1),
// //         amount,
// //       )
// //     );

// //     assert_ok!(
// //       Network::remove_subnet_node(
// //         RuntimeOrigin::signed(account(total_subnet_nodes+1)),
// //         subnet_id,
// //       )
// //     );

// //     assert_err!(
// //       Network::add_subnet_node(
// //         RuntimeOrigin::signed(account(total_subnet_nodes+1)),
// // account(total_subnet_nodes+1),
// //         subnet_id,
// //         peer(total_subnet_nodes+1),
// //         amount,
// //       ), 
// //       Error::<Test>::RequiredUnstakeEpochsNotMet
// //     );
// //   });
// // }

#[test]
fn test_add_subnet_node_remove_readd() {
  new_test_ext().execute_with(|| {
    let subnet_path: Vec<u8> = "petals-team/StableBeluga2".into();

    let deposit_amount: u128 = 10000000000000000000000;
    let amount: u128 = 1000000000000000000000;
    let stake_amount: u128 = MinStakeBalance::<Test>::get();

    build_activated_subnet(subnet_path.clone(), 0, 16, deposit_amount, stake_amount);

    let subnet_id = SubnetPaths::<Test>::get(subnet_path.clone()).unwrap();
    let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);

    let deposit_amount: u128 = 1000000000000000000000000;

    let _ = Balances::deposit_creating(&account(total_subnet_nodes+1), deposit_amount);

    assert_ok!(
      Network::add_subnet_node(
        RuntimeOrigin::signed(account(total_subnet_nodes+1)),
        subnet_id,
        account(total_subnet_nodes+1),
        peer(total_subnet_nodes+1),
        peer(total_subnet_nodes+1),
        0,
        amount,
        None,
        None,
        None,
      )
    );

    let subnet_node_id = HotkeySubnetNodeId::<Test>::get(subnet_id, account(total_subnet_nodes+1)).unwrap();

    assert_ok!(
      Network::remove_subnet_node(
        RuntimeOrigin::signed(account(total_subnet_nodes+1)),
        subnet_id,
        subnet_node_id,
      )
    );

    let account_subnet_stake = AccountSubnetStake::<Test>::get(&account(total_subnet_nodes+1), subnet_id);

    assert_ok!(
      Network::remove_stake(
        RuntimeOrigin::signed(account(total_subnet_nodes+1)),
        subnet_id,
        account(total_subnet_nodes+1),
        account_subnet_stake,
      )
    );

    assert_ok!(
      Network::add_subnet_node(
        RuntimeOrigin::signed(account(total_subnet_nodes+1)),
        subnet_id,
        account(total_subnet_nodes+1),
        peer(total_subnet_nodes+1),
        peer(total_subnet_nodes+1),
        0,
        amount,
        None,
        None,
        None,
      )
    );
  });
}

#[test]
fn test_add_subnet_node_not_key_owner() {
  new_test_ext().execute_with(|| {
    let subnet_path: Vec<u8> = "petals-team/StableBeluga2".into();

    let deposit_amount: u128 = 10000000000000000000000;
    let amount: u128 = 1000000000000000000000;
    let stake_amount: u128 = MinStakeBalance::<Test>::get();

    build_activated_subnet(subnet_path.clone(), 0, 0, deposit_amount, stake_amount);

    let subnet_id = SubnetPaths::<Test>::get(subnet_path.clone()).unwrap();
    let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);

    let deposit_amount: u128 = 1000000000000000000000000;

    let _ = Balances::deposit_creating(&account(total_subnet_nodes+1), deposit_amount);

    assert_ok!(
      Network::add_subnet_node(
        RuntimeOrigin::signed(account(total_subnet_nodes+1)),
        subnet_id,
        account(total_subnet_nodes+1),
        peer(total_subnet_nodes+1),
        peer(total_subnet_nodes+1),
        0,
        amount,
        None,
        None,
        None,
      )
    );

    let subnet_node_id = HotkeySubnetNodeId::<Test>::get(subnet_id, account(total_subnet_nodes+1)).unwrap();

    assert_err!(
      Network::remove_subnet_node(
        RuntimeOrigin::signed(account(total_subnet_nodes+1)),
        subnet_id,
        1,
      ),
      Error::<Test>::NotKeyOwner
    );

  });
}

#[test]
fn test_add_subnet_node_remove_readd_must_unstake_error() {
  new_test_ext().execute_with(|| {
    let subnet_path: Vec<u8> = "petals-team/StableBeluga2".into();

    let deposit_amount: u128 = 10000000000000000000000;
    let amount: u128 = 1000000000000000000000;
    let stake_amount: u128 = MinStakeBalance::<Test>::get();

    build_activated_subnet(subnet_path.clone(), 0, 16, deposit_amount, stake_amount);

    let subnet_id = SubnetPaths::<Test>::get(subnet_path.clone()).unwrap();
    let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);

    let deposit_amount: u128 = 1000000000000000000000000;

    let _ = Balances::deposit_creating(&account(total_subnet_nodes+1), deposit_amount);

    assert_ok!(
      Network::add_subnet_node(
        RuntimeOrigin::signed(account(total_subnet_nodes+1)),
        subnet_id,
        account(total_subnet_nodes+1),
        peer(total_subnet_nodes+1),
        peer(total_subnet_nodes+1),
        0,
        amount,
        None,
        None,
        None,
      )
    );

    let subnet_node_id = HotkeySubnetNodeId::<Test>::get(subnet_id, account(total_subnet_nodes+1)).unwrap();

    assert_ok!(
      Network::remove_subnet_node(
        RuntimeOrigin::signed(account(total_subnet_nodes+1)),
        subnet_id,
        subnet_node_id,
      )
    );

    assert_err!(
      Network::add_subnet_node(
        RuntimeOrigin::signed(account(total_subnet_nodes+1)),
        subnet_id,
        account(total_subnet_nodes+1),
        peer(total_subnet_nodes+1),
        peer(total_subnet_nodes+1),
        0,
        amount,
        None,
        None,
        None,
      ),
      Error::<Test>::MustUnstakeToRegister
    );
  });
}

#[test]
fn test_add_subnet_node_remove_stake_partial_readd() {
  new_test_ext().execute_with(|| {
    let subnet_path: Vec<u8> = "petals-team/StableBeluga2".into();

    let deposit_amount: u128 = 10000000000000000000000;
    let amount: u128 = 1000000000000000000000;
    let stake_amount: u128 = MinStakeBalance::<Test>::get();

    build_activated_subnet(subnet_path.clone(), 0, 16, deposit_amount, stake_amount);

    let subnet_id = SubnetPaths::<Test>::get(subnet_path.clone()).unwrap();
    let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);

    let deposit_amount: u128 = 1000000000000000000000000;

    let _ = Balances::deposit_creating(&account(total_subnet_nodes+1), deposit_amount);

    assert_ok!(
      Network::add_subnet_node(
        RuntimeOrigin::signed(account(total_subnet_nodes+1)),
        subnet_id,
        account(total_subnet_nodes+1),
        peer(total_subnet_nodes+1),
        peer(total_subnet_nodes+1),
        0,
        amount,
        None,
        None,
        None,
      )
    );

    // increase account subnet stake to simulate rewards
    AccountSubnetStake::<Test>::insert(&account(total_subnet_nodes+1), subnet_id, amount + 100);

    let subnet_node_id = HotkeySubnetNodeId::<Test>::get(subnet_id, account(total_subnet_nodes+1)).unwrap();

    assert_ok!(
      Network::remove_subnet_node(
        RuntimeOrigin::signed(account(total_subnet_nodes+1)),
        subnet_id,
        subnet_node_id,
      )
    );

    // once blocks have been increased, account can either remove stake in part or in full or readd subnet peer
    let epoch_length = EpochLength::get();
    let min_required_unstake_epochs = StakeCooldownEpochs::get();

    // System::set_block_number(System::block_number() + epoch_length * min_required_unstake_epochs);
    increase_epochs(min_required_unstake_epochs);

    let account_subnet_stake = AccountSubnetStake::<Test>::get(&account(total_subnet_nodes+1), subnet_id);

    assert_ok!(
      Network::remove_stake(
        RuntimeOrigin::signed(account(total_subnet_nodes+1)),
        subnet_id,
        account(total_subnet_nodes+1),
        account_subnet_stake,
      )
    );

    // should be able to readd after unstaking
    assert_ok!(
      Network::add_subnet_node(
        RuntimeOrigin::signed(account(total_subnet_nodes+1)),
        subnet_id,
        account(total_subnet_nodes+1),
        peer(total_subnet_nodes+1),
        peer(total_subnet_nodes+1),
        0,
        amount,
        None,
        None,
        None,
      ) 
    );
  });
}

#[test]
fn test_add_subnet_node_remove_stake_readd() {
  new_test_ext().execute_with(|| {
    let subnet_path: Vec<u8> = "petals-team/StableBeluga2".into();
    let deposit_amount: u128 = 1000000000000000000000000;
    let amount: u128 = 1000000000000000000000;

    let stake_amount: u128 = MinStakeBalance::<Test>::get();

    build_activated_subnet(subnet_path.clone(), 0, 16, deposit_amount, stake_amount);

    let subnet_id = SubnetPaths::<Test>::get(subnet_path.clone()).unwrap();
    let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);

    let _ = Balances::deposit_creating(&account(total_subnet_nodes+1), deposit_amount);

    assert_ok!(
      Network::add_subnet_node(
        RuntimeOrigin::signed(account(total_subnet_nodes+1)),
        subnet_id,
        account(total_subnet_nodes+1),
        peer(total_subnet_nodes+1),
        peer(total_subnet_nodes+1),
        0,
        amount,
        None,
        None,
        None,
      )
    );

    let subnet_node_id = HotkeySubnetNodeId::<Test>::get(subnet_id, account(total_subnet_nodes+1)).unwrap();

    assert_ok!(
      Network::remove_subnet_node(
        RuntimeOrigin::signed(account(total_subnet_nodes+1)),
        subnet_id,
        subnet_node_id,
      )
    );

    // once blocks have been increased, account can either remove stake in part or in full or readd subnet peer
    let epoch_length = EpochLength::get();
    let min_required_unstake_epochs = StakeCooldownEpochs::get();
    // System::set_block_number(System::block_number() + epoch_length * min_required_unstake_epochs);
    increase_epochs(min_required_unstake_epochs);

    let remaining_account_stake_balance: u128 = AccountSubnetStake::<Test>::get(&account(1), subnet_id);

    assert_ok!(
      Network::remove_stake(
        RuntimeOrigin::signed(account(total_subnet_nodes+1)),
        subnet_id,
        account(total_subnet_nodes+1),
        remaining_account_stake_balance,
      )
    );

    // should be able to readd after unstaking
    assert_ok!(
      Network::add_subnet_node(
        RuntimeOrigin::signed(account(total_subnet_nodes+1)),
        subnet_id,
        account(total_subnet_nodes+1),
        peer(total_subnet_nodes+1),
        peer(total_subnet_nodes+1),
        0,
        amount,
        None,
        None,
        None,
      ) 
    );
  });
}

#[test]
fn test_register_subnet_node_with_a_param() {
  new_test_ext().execute_with(|| {
    let subnet_path: Vec<u8> = "petals-team/StableBeluga2".into();
    
    let deposit_amount: u128 = 10000000000000000000000;
    let amount: u128 = 1000000000000000000000;

    let stake_amount: u128 = MinStakeBalance::<Test>::get();

    build_activated_subnet(subnet_path.clone(), 0, 0, deposit_amount, stake_amount);

    let subnet_id = SubnetPaths::<Test>::get(subnet_path.clone()).unwrap();
    let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);

    let _ = Balances::deposit_creating(&account(total_subnet_nodes+1), deposit_amount);

    let a: Vec<u8> = "a".into();
    let bounded_a: BoundedVec<u8, DefaultSubnetNodeUniqueParamLimit> = a.try_into().expect("String too long");

    assert_ok!(
      Network::register_subnet_node(
        RuntimeOrigin::signed(account(total_subnet_nodes+1)),
        subnet_id,
        account(total_subnet_nodes+1),
        peer(total_subnet_nodes+1),
        peer(total_subnet_nodes+1),
        0,
        amount,
        Some(bounded_a.clone()),
        None,
        None,
      )
    );

    let subnet_node_id = HotkeySubnetNodeId::<Test>::get(subnet_id, account(total_subnet_nodes+1)).unwrap();

    let subnet_node = SubnetNodesData::<Test>::get(subnet_id, subnet_node_id);
    assert_eq!(subnet_node.a, Some(bounded_a.clone()));
  })
}

#[test]
fn test_register_subnet_node_and_then_update_a_param() {
  new_test_ext().execute_with(|| {
    let subnet_path: Vec<u8> = "petals-team/StableBeluga2".into();
    
    let deposit_amount: u128 = 10000000000000000000000;
    let amount: u128 = 1000000000000000000000;

    let stake_amount: u128 = MinStakeBalance::<Test>::get();

    build_activated_subnet(subnet_path.clone(), 0, 0, deposit_amount, stake_amount);

    let subnet_id = SubnetPaths::<Test>::get(subnet_path.clone()).unwrap();
    let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);

    let _ = Balances::deposit_creating(&account(total_subnet_nodes+1), deposit_amount);

    assert_ok!(
      Network::register_subnet_node(
        RuntimeOrigin::signed(account(total_subnet_nodes+1)),
        subnet_id,
        account(total_subnet_nodes+1),
        peer(total_subnet_nodes+1),
        peer(total_subnet_nodes+1),
        0,
        amount,
        None,
        None,
        None,
      )
    );

    let subnet_node_id = HotkeySubnetNodeId::<Test>::get(subnet_id, account(total_subnet_nodes+1)).unwrap();

    let subnet_node = SubnetNodesData::<Test>::get(subnet_id, subnet_node_id);
    assert_eq!(subnet_node.a, None);

    
    let a: Vec<u8> = "a".into();
    let bounded_a: BoundedVec<u8, DefaultSubnetNodeUniqueParamLimit> = a.try_into().expect("String too long");

    assert_ok!(
      Network::register_subnet_node_a_parameter(
        RuntimeOrigin::signed(account(total_subnet_nodes+1)),
        subnet_id,
        subnet_node_id,
        bounded_a.clone(),
      )
    );

    let subnet_node = SubnetNodesData::<Test>::get(subnet_id, subnet_node_id);
    assert_eq!(subnet_node.a, Some(bounded_a.clone()));

    assert_err!(
      Network::register_subnet_node_a_parameter(
        RuntimeOrigin::signed(account(total_subnet_nodes+1)),
        subnet_id,
        subnet_node_id,
        bounded_a.clone(),
      ),
      Error::<Test>::SubnetNodeUniqueParamTaken
    );

    let subnet_node = SubnetNodesData::<Test>::get(subnet_id, subnet_node_id);
    assert_eq!(subnet_node.a, Some(bounded_a.clone()));

    let a_v2: Vec<u8> = "a_v2".into();
    let bounded_a_v2: BoundedVec<u8, DefaultSubnetNodeUniqueParamLimit> = a_v2.try_into().expect("String too long");

    assert_err!(
      Network::register_subnet_node_a_parameter(
        RuntimeOrigin::signed(account(total_subnet_nodes+1)),
        subnet_id,
        subnet_node_id,
        bounded_a_v2.clone(),
      ),
      Error::<Test>::SubnetNodeUniqueParamIsSet
    );

    let subnet_node = SubnetNodesData::<Test>::get(subnet_id, subnet_node_id);
    assert_eq!(subnet_node.a, Some(bounded_a.clone()));

  })
}

#[test]
fn test_register_subnet_node_with_non_unique_param() {
  new_test_ext().execute_with(|| {
    let subnet_path: Vec<u8> = "petals-team/StableBeluga2".into();
    
    let deposit_amount: u128 = 10000000000000000000000;
    let amount: u128 = 1000000000000000000000;

    let stake_amount: u128 = MinStakeBalance::<Test>::get();

    build_activated_subnet(subnet_path.clone(), 0, 0, deposit_amount, stake_amount);

    let subnet_id = SubnetPaths::<Test>::get(subnet_path.clone()).unwrap();
    let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);

    let _ = Balances::deposit_creating(&account(total_subnet_nodes+1), deposit_amount);

    let b: Vec<u8> = "b".into();
    let bounded_b: BoundedVec<u8, DefaultSubnetNodeUniqueParamLimit> = b.try_into().expect("String too long");

    let c: Vec<u8> = "c".into();
    let bounded_c: BoundedVec<u8, DefaultSubnetNodeUniqueParamLimit> = c.try_into().expect("String too long");

    assert_ok!(
      Network::register_subnet_node(
        RuntimeOrigin::signed(account(total_subnet_nodes+1)),
        subnet_id,
        account(total_subnet_nodes+1),
        peer(total_subnet_nodes+1),
        peer(total_subnet_nodes+1),
        0,
        amount,
        None,
        Some(bounded_b.clone()),
        Some(bounded_c.clone()),
      )
    );

    let subnet_node_id = HotkeySubnetNodeId::<Test>::get(subnet_id, account(total_subnet_nodes+1)).unwrap();

    let subnet_node = SubnetNodesData::<Test>::get(subnet_id, subnet_node_id);
    assert_eq!(subnet_node.a, None);
    assert_eq!(subnet_node.b, Some(bounded_b.clone()));
    assert_eq!(subnet_node.c, Some(bounded_c.clone()));
  })
}

#[test]
fn test_update_subnet_node_with_non_unique_param() {
  new_test_ext().execute_with(|| {
    let subnet_path: Vec<u8> = "petals-team/StableBeluga2".into();
    
    let deposit_amount: u128 = 10000000000000000000000;
    let amount: u128 = 1000000000000000000000;

    let stake_amount: u128 = MinStakeBalance::<Test>::get();

    build_activated_subnet(subnet_path.clone(), 0, 0, deposit_amount, stake_amount);

    let subnet_id = SubnetPaths::<Test>::get(subnet_path.clone()).unwrap();

    let _ = Balances::deposit_creating(&account(1), deposit_amount);

    let b: Vec<u8> = "b".into();
    let bounded_b: BoundedVec<u8, DefaultSubnetNodeUniqueParamLimit> = b.try_into().expect("String too long");

    let c: Vec<u8> = "c".into();
    let bounded_c: BoundedVec<u8, DefaultSubnetNodeUniqueParamLimit> = c.try_into().expect("String too long");
    let subnet_node_id = HotkeySubnetNodeId::<Test>::get(subnet_id, account(1)).unwrap();

    increase_epochs(1);

    assert_ok!(
      Network::set_subnet_node_non_unique_parameter(
        RuntimeOrigin::signed(account(1)),
        subnet_id,
        subnet_node_id,
        Some(bounded_b.clone()),
        Some(bounded_c.clone()),
      )
    );

    let subnet_node_id = HotkeySubnetNodeId::<Test>::get(subnet_id, account(1)).unwrap();

    let subnet_node = SubnetNodesData::<Test>::get(subnet_id, subnet_node_id);
    assert_eq!(subnet_node.b, Some(bounded_b.clone()));
    assert_eq!(subnet_node.c, Some(bounded_c.clone()));

    assert_err!(
      Network::set_subnet_node_non_unique_parameter(
        RuntimeOrigin::signed(account(1)),
        subnet_id,
        subnet_node_id,
        Some(bounded_b.clone()),
        Some(bounded_c.clone()),
      ),
      Error::<Test>::SubnetNodeNonUniqueParamUpdateIntervalNotReached
    );

    increase_epochs(1);

    assert_err!(
      Network::set_subnet_node_non_unique_parameter(
        RuntimeOrigin::signed(account(1)),
        subnet_id,
        subnet_node_id,
        None,
        None,
      ),
      Error::<Test>::SubnetNodeNonUniqueParamMustBeSome
    );

    let b2: Vec<u8> = "b".into();
    let bounded_b2: BoundedVec<u8, DefaultSubnetNodeUniqueParamLimit> = b2.try_into().expect("String too long");

    let c2: Vec<u8> = "c".into();
    let bounded_c2: BoundedVec<u8, DefaultSubnetNodeUniqueParamLimit> = c2.try_into().expect("String too long");

    assert_ok!(
      Network::set_subnet_node_non_unique_parameter(
        RuntimeOrigin::signed(account(1)),
        subnet_id,
        subnet_node_id,
        Some(bounded_b2.clone()),
        Some(bounded_c2.clone()),
      )
    );

    let subnet_node = SubnetNodesData::<Test>::get(subnet_id, subnet_node_id);
    assert_eq!(subnet_node.b, Some(bounded_b2.clone()));
    assert_eq!(subnet_node.c, Some(bounded_c2.clone()));

  })
}

// // #[test]
// // fn test_remove_peer_error() {
// //   new_test_ext().execute_with(|| {
// //     let subnet_path: Vec<u8> = "petals-team/StableBeluga2".into();
// //     let deposit_amount: u128 = 1000000000000000000000000;
// //     let amount: u128 = 1000000000000000000000;

// //     build_activated_subnet(subnet_path.clone(), 0, 0, deposit_amount, amount);

// //     let subnet_id = SubnetPaths::<Test>::get(subnet_path.clone()).unwrap();
// //     let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);

// //     let _ = Balances::deposit_creating(&account(total_subnet_nodes+1), deposit_amount);

// //     let subnet_id = SubnetPaths::<Test>::get(subnet_path.clone()).unwrap();

// //     assert_ok!(
// //       Network::add_subnet_node(
// //         RuntimeOrigin::signed(account(total_subnet_nodes+1)),
// // account(total_subnet_nodes+1),
// //         subnet_id,
// //         peer(total_subnet_nodes+1),
// //         amount,
// //       ) 
// //     );
// //     // post_successful_add_subnet_node_asserts(0, subnet_id, amount);

// //     // post_successful_add_subnet_nodes_asserts(
// //     //   1,
// //     //   amount,
// //     //   subnet_id,
// //     // );

// //     // assert_eq!(Network::total_stake(), amount);

// //     assert_err!(
// //       Network::remove_subnet_node(
// //         RuntimeOrigin::signed(account(total_subnet_nodes+1)),
// //         subnet_id,
// //       ),
// //       Error::<Test>::SubnetNodeNotExist
// //     );
// //   });
// // }

// // // #[test]
// // // fn test_remove_peer_unstake_epochs_err() {
// // //   new_test_ext().execute_with(|| {
// // //     let subnet_path: Vec<u8> = "petals-team/StableBeluga2".into();

// // //     build_subnet(subnet_path.clone());
// // //     let deposit_amount: u128 = 1000000000000000000000000;
// // //     let amount: u128 = 1000000000000000000000;
// // //     let _ = Balances::deposit_creating(&account(1), deposit_amount);

// // //     let subnet_id = SubnetPaths::<Test>::get(subnet_path.clone()).unwrap();

// // //     let epoch_length = EpochLength::get();

// // //     System::set_block_number(System::block_number() + epoch_length);

// // //     assert_ok!(
// // //       Network::add_subnet_node(
// // //         RuntimeOrigin::signed(account(1)),
// // account(1),
// // //         subnet_id,
// // //         peer(1),
// // //         amount,
// // //       ) 
// // //     );
// // //     post_successful_add_subnet_node_asserts(0, subnet_id, amount);
// // //     assert_eq!(Network::total_subnet_nodes(1), 1);
// // //     assert_eq!(Network::account_subnet_stake(account(1), 1), amount);
// // //     assert_eq!(Network::total_account_stake(account(1)), amount);
// // //     assert_eq!(Network::total_stake(), amount);
// // //     assert_eq!(Network::total_subnet_stake(1), amount);

// // //     // make_subnet_node_removable();


// // //     System::set_block_number(System::block_number() + epoch_length);

// // //     assert_ok!(
// // //       Network::remove_subnet_node(
// // //         RuntimeOrigin::signed(account(1)),
// // //         subnet_id,
// // //       ) 
// // //     );

// // //     post_remove_subnet_node_ensures(0, subnet_id);

// // //     assert_eq!(Network::total_subnet_nodes(1), 0);

// // //     assert_err!(
// // //       Network::remove_stake(
// // //         RuntimeOrigin::signed(account(1)),
// // account(1),
// // //         subnet_id,
// // //         amount,
// // //       ),
// // //       Error::<Test>::RequiredUnstakeEpochsNotMet,
// // //     );
    
// // //     let epoch_length = EpochLength::get();
// // //     let min_required_unstake_epochs = StakeCooldownEpochs::get();
// // //     System::set_block_number(System::block_number() + epoch_length * min_required_unstake_epochs);
    
// // //     assert_ok!(
// // //       Network::remove_stake(
// // //         RuntimeOrigin::signed(account(1)),
// // account(1),
// // //         subnet_id,
// // //         amount,
// // //       )
// // //     );
// // //   });
// // // }

#[test]
fn test_remove_peer_unstake_total_balance() {
  new_test_ext().execute_with(|| {
    let subnet_path: Vec<u8> = "petals-team/StableBeluga2".into();
    let deposit_amount: u128 = 1000000000000000000000000;
    let amount: u128 = 1000000000000000000000;

    let stake_amount: u128 = MinStakeBalance::<Test>::get();

    build_activated_subnet(subnet_path.clone(), 0, 0, deposit_amount, stake_amount);

    let subnet_id = SubnetPaths::<Test>::get(subnet_path.clone()).unwrap();
    let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);
    let _ = Balances::deposit_creating(&account(total_subnet_nodes+1), deposit_amount);

    assert_ok!(
      Network::add_subnet_node(
        RuntimeOrigin::signed(account(total_subnet_nodes+1)),
        subnet_id,
        account(total_subnet_nodes+1),
        peer(total_subnet_nodes+1),
        peer(total_subnet_nodes+1),
        0,
        amount,
        None,
        None,
        None,
      ) 
    );
    // post_successful_add_subnet_node_asserts(0, subnet_id, amount);

    assert_eq!(Network::total_subnet_nodes(subnet_id), total_subnet_nodes+1);
    assert_eq!(Network::account_subnet_stake(account(1), subnet_id), amount);
    // assert_eq!(Network::total_account_stake(account(1)), amount);
    assert_eq!(Network::total_stake(), amount * (total_subnet_nodes as u128 +1));
    assert_eq!(Network::total_subnet_stake(subnet_id), amount * (total_subnet_nodes as u128 +1));

    let subnet_node_id = HotkeySubnetNodeId::<Test>::get(subnet_id, account(total_subnet_nodes+1)).unwrap();

    assert_ok!(
      Network::remove_subnet_node(
        RuntimeOrigin::signed(account(total_subnet_nodes+1)),
        subnet_id,
        subnet_node_id,
      ) 
    );

    // post_remove_subnet_node_ensures(0, subnet_id);

    assert_eq!(Network::total_subnet_nodes(subnet_id), total_subnet_nodes);
    
    let epoch_length = EpochLength::get();
    let min_required_unstake_epochs = StakeCooldownEpochs::get();
    // System::set_block_number(System::block_number() + epoch_length * min_required_unstake_epochs);
    increase_epochs(min_required_unstake_epochs + 1);
    
    let remaining_account_stake_balance: u128 = AccountSubnetStake::<Test>::get(&account(total_subnet_nodes+1), subnet_id);

    assert_ok!(
      Network::remove_stake(
        RuntimeOrigin::signed(account(total_subnet_nodes+1)),
        subnet_id,
        account(total_subnet_nodes+1),
        remaining_account_stake_balance,
      )
    );

    // post_remove_unstake_ensures(0, subnet_id);
  });
}

#[test]
fn test_claim_stake_unbondings() {
  new_test_ext().execute_with(|| {
    let subnet_path: Vec<u8> = "petals-team/StableBeluga2".into();
    let deposit_amount: u128 = 1000000000000000000000000;
    let amount: u128 = 1000000000000000000000;

    let stake_amount: u128 = MinStakeBalance::<Test>::get();

    build_activated_subnet(subnet_path.clone(), 0, 0, deposit_amount, stake_amount);

    let subnet_id = SubnetPaths::<Test>::get(subnet_path.clone()).unwrap();
    let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);

    let _ = Balances::deposit_creating(&account(total_subnet_nodes+1), deposit_amount);

    let starting_balance = Balances::free_balance(&account(total_subnet_nodes+1));
    assert_eq!(starting_balance, deposit_amount);

    assert_ok!(
      Network::add_subnet_node(
        RuntimeOrigin::signed(account(total_subnet_nodes+1)),
        subnet_id,
        account(total_subnet_nodes+1),
        peer(total_subnet_nodes+1),
        peer(total_subnet_nodes+1),
        0,
        amount,
        None,
        None,
        None,
      ) 
    );

    let subnet_node_id = HotkeySubnetNodeId::<Test>::get(subnet_id, account(total_subnet_nodes+1)).unwrap();

    let stake_balance = AccountSubnetStake::<Test>::get(&account(total_subnet_nodes+1), subnet_id);
    assert_eq!(stake_balance, amount);

    let after_stake_balance = Balances::free_balance(&account(total_subnet_nodes+1));
    assert_eq!(after_stake_balance, starting_balance - amount);

    assert_ok!(
      Network::remove_subnet_node(
        RuntimeOrigin::signed(account(total_subnet_nodes+1)), 
        subnet_id,
        subnet_node_id,
      ) 
    );

    let stake_balance = AccountSubnetStake::<Test>::get(&account(total_subnet_nodes+1), subnet_id);

    // remove amount ontop
    assert_ok!(
      Network::remove_stake(
        RuntimeOrigin::signed(account(total_subnet_nodes+1)),
        subnet_id,
        account(total_subnet_nodes+1),
        stake_balance,
      )
    );

    assert_eq!(Network::account_subnet_stake(account(total_subnet_nodes+1), 1), 0);
    // assert_eq!(Network::total_account_stake(account(total_subnet_nodes+1)), 0);

    let epoch_length = EpochLength::get();
    let epoch = System::block_number() / epoch_length;

    let unbondings: BTreeMap<u32, u128> = StakeUnbondingLedger::<Test>::get(account(total_subnet_nodes+1));

    assert_eq!(unbondings.len(), 1);
    let (first_key, first_value) = unbondings.iter().next().unwrap();
    
    // assert_eq!(*first_key, &epoch + StakeCooldownEpochs::get());
    assert_eq!(*first_key, &epoch + RegisteredStakeCooldownEpochs::<Test>::get());
    assert!(*first_value <= stake_balance);
    
    // let stake_cooldown_epochs = StakeCooldownEpochs::get();
    let stake_cooldown_epochs = RegisteredStakeCooldownEpochs::<Test>::get();

    increase_epochs(stake_cooldown_epochs + 1);
    // System::set_block_number(System::block_number() + ((epoch_length  + 1) * stake_cooldown_epochs));

    assert_ok!(
      Network::claim_unbondings(
        RuntimeOrigin::signed(account(total_subnet_nodes+1)),
      )
    );

    let post_balance = Balances::free_balance(&account(total_subnet_nodes+1));

    assert_eq!(post_balance, starting_balance);

    let unbondings: BTreeMap<u32, u128> = StakeUnbondingLedger::<Test>::get(account(total_subnet_nodes+1));

    assert_eq!(unbondings.len(), 0);
  });
}

#[test]
fn test_remove_stake_twice_in_epoch() {
  new_test_ext().execute_with(|| {
    let subnet_path: Vec<u8> = "petals-team/StableBeluga2".into();
    let deposit_amount: u128 = 1000000000000000000000000;
    let amount: u128 = 1000000000000000000000;

    let stake_amount: u128 = MinStakeBalance::<Test>::get();

    build_activated_subnet(subnet_path.clone(), 0, 0, deposit_amount, stake_amount);

    let subnet_id = SubnetPaths::<Test>::get(subnet_path.clone()).unwrap();
    let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);

    let _ = Balances::deposit_creating(&account(total_subnet_nodes+1), deposit_amount);

    let starting_balance = Balances::free_balance(&account(total_subnet_nodes+1));
    assert_eq!(starting_balance, deposit_amount);

    assert_ok!(
      Network::add_subnet_node(
        RuntimeOrigin::signed(account(total_subnet_nodes+1)),
        subnet_id,
        account(total_subnet_nodes+1),
        peer(total_subnet_nodes+1),
        peer(total_subnet_nodes+1),
        0,
        amount,
        None,
        None,
        None,
      ) 
    );

    let subnet_node_id = HotkeySubnetNodeId::<Test>::get(subnet_id, account(total_subnet_nodes+1)).unwrap();

    let stake_balance = AccountSubnetStake::<Test>::get(&account(total_subnet_nodes+1), subnet_id);
    assert_eq!(stake_balance, amount);

    let after_stake_balance = Balances::free_balance(&account(total_subnet_nodes+1));
    assert_eq!(after_stake_balance, starting_balance - amount);

    let _ = Balances::deposit_creating(&account(1), amount*2);

    assert_ok!(
      Network::add_to_stake(
        RuntimeOrigin::signed(account(total_subnet_nodes+1)),
        subnet_id,
        subnet_node_id,
        account(total_subnet_nodes+1),
        amount*3,
      ) 
    );

    let stake_balance = AccountSubnetStake::<Test>::get(&account(total_subnet_nodes+1), subnet_id);
    assert_eq!(stake_balance, amount + amount*3);

    let epoch = System::block_number() / EpochLength::get();

    assert_ok!(
      Network::remove_stake(
        RuntimeOrigin::signed(account(total_subnet_nodes+1)),
        subnet_id,
        account(total_subnet_nodes+1),
        amount,
      )
    );

    let unbondings: BTreeMap<u32, u128> = StakeUnbondingLedger::<Test>::get(account(total_subnet_nodes+1));
    let ledger_balance: u128 = unbondings.values().copied().sum();
    assert_eq!(unbondings.len() as u32, 1);  
    assert_eq!(ledger_balance, amount);  

    let (ledger_epoch, ledger_balance) = unbondings.iter().next().unwrap();
    assert_eq!(*ledger_epoch, &epoch + StakeCooldownEpochs::get());

    assert_ok!(
      Network::remove_stake(
        RuntimeOrigin::signed(account(total_subnet_nodes+1)),
        subnet_id,
        account(total_subnet_nodes+1),
        amount,
      )
    );

    let unbondings: BTreeMap<u32, u128> = StakeUnbondingLedger::<Test>::get(account(total_subnet_nodes+1));
    let ledger_balance: u128 = unbondings.values().copied().sum();
    assert_eq!(unbondings.len() as u32, 1);  
    assert_eq!(ledger_balance, amount*2);

    let (ledger_epoch, ledger_balance) = unbondings.iter().next().unwrap();
    assert_eq!(*ledger_epoch, &epoch + StakeCooldownEpochs::get());

    increase_epochs(1);

    let epoch = System::block_number() / EpochLength::get();

    assert_ok!(
      Network::remove_stake(
        RuntimeOrigin::signed(account(total_subnet_nodes+1)),
        subnet_id,
        account(total_subnet_nodes+1),
        amount,
      )
    );

    let unbondings: BTreeMap<u32, u128> = StakeUnbondingLedger::<Test>::get(account(total_subnet_nodes+1));
    let total_ledger_balance: u128 = unbondings.values().copied().sum();
    assert_eq!(unbondings.len() as u32, 2);  
    assert_eq!(total_ledger_balance, amount*3);

    let (ledger_epoch, ledger_balance) = unbondings.iter().last().unwrap();
    assert_eq!(*ledger_epoch, &epoch + StakeCooldownEpochs::get());
    assert_eq!(*ledger_balance, amount);

    System::set_block_number(System::block_number() + ((EpochLength::get()  + 1) * StakeCooldownEpochs::get()));
    // increase_epochs(StakeCooldownEpochs::get() + 11);
    
    let starting_balance = Balances::free_balance(&account(total_subnet_nodes+1));

    assert_ok!(
      Network::claim_unbondings(
        RuntimeOrigin::signed(account(total_subnet_nodes+1)),
      )
    );

    let ending_balance = Balances::free_balance(&account(total_subnet_nodes+1));
    assert_eq!(starting_balance + total_ledger_balance, ending_balance);

  });
}


#[test]
fn test_claim_stake_unbondings_no_unbondings_err() {
  new_test_ext().execute_with(|| {
    let subnet_path: Vec<u8> = "petals-team/StableBeluga2".into();
    let deposit_amount: u128 = 1000000000000000000000000;
    let amount: u128 = 1000000000000000000000;

    let stake_amount: u128 = MinStakeBalance::<Test>::get();

    build_activated_subnet(subnet_path.clone(), 0, 0, deposit_amount, stake_amount);

    let subnet_id = SubnetPaths::<Test>::get(subnet_path.clone()).unwrap();
    let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);

    let _ = Balances::deposit_creating(&account(total_subnet_nodes+1), deposit_amount);

    let starting_balance = Balances::free_balance(&account(total_subnet_nodes+1));
    assert_eq!(starting_balance, deposit_amount);

    assert_ok!(
      Network::add_subnet_node(
        RuntimeOrigin::signed(account(total_subnet_nodes+1)),
        subnet_id,
        account(total_subnet_nodes+1),
        peer(total_subnet_nodes+1),
        peer(total_subnet_nodes+1),
        0,
        amount,
        None,
        None,
        None,
      ) 
    );

    let stake_balance = AccountSubnetStake::<Test>::get(&account(total_subnet_nodes+1), subnet_id);
    assert_eq!(stake_balance, amount);

    let after_stake_balance = Balances::free_balance(&account(total_subnet_nodes+1));
    assert_eq!(after_stake_balance, starting_balance - amount);

    assert_err!(
      Network::claim_unbondings(
        RuntimeOrigin::signed(account(total_subnet_nodes+1)),
      ),
      Error::<Test>::NoStakeUnbondingsOrCooldownNotMet
    );
  });
}

#[test]
fn test_remove_to_stake_max_unlockings_reached_err() {
  new_test_ext().execute_with(|| {
    let subnet_path: Vec<u8> = "petals-team/StableBeluga2".into();
    let deposit_amount: u128 = 1000000000000000000000000;
    let amount: u128 = 1000000000000000000000;

    let stake_amount: u128 = MinStakeBalance::<Test>::get();

    build_activated_subnet(subnet_path.clone(), 0, 0, deposit_amount, stake_amount);

    let subnet_id = SubnetPaths::<Test>::get(subnet_path.clone()).unwrap();
    let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);

    let _ = Balances::deposit_creating(&account(total_subnet_nodes+1), deposit_amount);

    let starting_balance = Balances::free_balance(&account(total_subnet_nodes+1));

    assert_ok!(
      Network::add_subnet_node(
        RuntimeOrigin::signed(account(total_subnet_nodes+1)),
        subnet_id,
        account(total_subnet_nodes+1),
        peer(total_subnet_nodes+1),
        peer(total_subnet_nodes+1),
        0,
        amount*2,
        None,
        None,
        None,
      ) 
    );

    let max_unlockings = MaxStakeUnlockings::get();
    for n in 1..max_unlockings+2 {
      // System::set_block_number(System::block_number() + EpochLength::get() + 1);
      increase_epochs(1);
      if n > max_unlockings {
        assert_err!(
          Network::remove_stake(
            RuntimeOrigin::signed(account(total_subnet_nodes+1)),
            subnet_id,
            account(total_subnet_nodes+1),
            1000,
          ),
          Error::<Test>::MaxUnlockingsReached
        );    
      } else {
        assert_ok!(
          Network::remove_stake(
            RuntimeOrigin::signed(account(total_subnet_nodes+1)),
            subnet_id,
            account(total_subnet_nodes+1),
            1000,
          )
        );

        let unbondings: BTreeMap<u32, u128> = StakeUnbondingLedger::<Test>::get(account(total_subnet_nodes+1));

        assert_eq!(unbondings.len() as u32, n);  
      }
    }
  });
}

#[test]
fn test_remove_subnet_node() {
  new_test_ext().execute_with(|| {
    let subnet_path: Vec<u8> = "petals-team/StableBeluga2".into();
    let deposit_amount: u128 = 1000000000000000000000000;
    let amount: u128 = 1000000000000000000000;

    let stake_amount: u128 = MinStakeBalance::<Test>::get();

    build_activated_subnet(subnet_path.clone(), 0, 0, deposit_amount, stake_amount);

    let subnet_id = SubnetPaths::<Test>::get(subnet_path.clone()).unwrap();
    let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);
    let amount_staked = TotalSubnetStake::<Test>::get(subnet_id);
    let remove_n_peers = total_subnet_nodes / 2;

    let block_number = System::block_number();
    let epoch_length = EpochLength::get();
    let epoch = block_number / epoch_length;

    for n in 1..remove_n_peers+1 {
      let subnet_node_id = HotkeySubnetNodeId::<Test>::get(subnet_id, account(n)).unwrap();
      assert_ok!(
        Network::remove_subnet_node(
          RuntimeOrigin::signed(account(n)),
          subnet_id,
          subnet_node_id,
        ) 
      );
      let subnet_node_data = SubnetNodesData::<Test>::try_get(subnet_id, subnet_node_id);
      assert_eq!(subnet_node_data, Err(()));
    }

    // let node_set = Network::get_classified_hotkeys(subnet_id, &SubnetNodeClass::Queue, epoch);
    let node_set: BTreeSet<<Test as frame_system::Config>::AccountId> = Network::get_classified_hotkeys(subnet_id, &SubnetNodeClass::Queue, epoch);

    assert_eq!(node_set.len(), (total_subnet_nodes - remove_n_peers) as usize);
    assert_eq!(Network::total_stake(), amount_staked);
    assert_eq!(Network::total_subnet_stake(subnet_id), amount_staked);
    assert_eq!(TotalSubnetNodes::<Test>::get(subnet_id), total_subnet_nodes - remove_n_peers);

    for n in 1..remove_n_peers+1 {
      let subnet_node_id = HotkeySubnetNodeId::<Test>::try_get(subnet_id, account(n));
      assert_eq!(subnet_node_id, Err(()));

      let subnet_node_account = PeerIdSubnetNode::<Test>::try_get(subnet_id, peer(n));
      assert_eq!(subnet_node_account, Err(()));
  
      let account_subnet_stake = AccountSubnetStake::<Test>::get(account(n), subnet_id);
      assert_eq!(account_subnet_stake, amount);
    }

    let total_subnet_stake = TotalSubnetStake::<Test>::get(subnet_id);
    assert_eq!(total_subnet_stake, amount_staked);

    let total_stake = TotalStake::<Test>::get();
    assert_eq!(total_subnet_stake, amount_staked);
  });
}

#[test]
fn test_deactivate_subnet_node_and_reactivate() {
  new_test_ext().execute_with(|| {
    let subnet_path: Vec<u8> = "petals-team/StableBeluga2".into();
    let deposit_amount: u128 = 10000000000000000000000;
    let amount: u128 = 1000000000000000000000;

    let n_peers = 8;
    let stake_amount: u128 = MinStakeBalance::<Test>::get();

    build_activated_subnet(subnet_path.clone(), 0, n_peers, deposit_amount, stake_amount);

    let subnet_id = SubnetPaths::<Test>::get(subnet_path.clone()).unwrap();
    let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);
    let subnet_node_id = HotkeySubnetNodeId::<Test>::get(subnet_id, account(1)).unwrap();

    assert_ok!(
      Network::deactivate_subnet_node(
        RuntimeOrigin::signed(account(1)),
        subnet_id,
        subnet_node_id,
      )
    );
  
    let subnet_node = SubnetNodesData::<Test>::get(subnet_id, subnet_node_id);
    assert_eq!(subnet_node.classification.class, SubnetNodeClass::Deactivated);
  });
}

#[test]
fn test_deactivate_subnet_node() {
  new_test_ext().execute_with(|| {
    let subnet_path: Vec<u8> = "petals-team/StableBeluga2".into();
    let deposit_amount: u128 = 10000000000000000000000;
    let amount: u128 = 1000000000000000000000;

    let n_peers = 8;
    let stake_amount: u128 = MinStakeBalance::<Test>::get();

    build_activated_subnet(subnet_path.clone(), 0, n_peers, deposit_amount, stake_amount);

    let subnet_id = SubnetPaths::<Test>::get(subnet_path.clone()).unwrap();
    let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);
    let subnet_node_id = HotkeySubnetNodeId::<Test>::get(subnet_id, account(1)).unwrap();

    assert_ok!(
      Network::deactivate_subnet_node(
        RuntimeOrigin::signed(account(1)),
        subnet_id,
        subnet_node_id,
      )
    );
  
    let subnet_node = SubnetNodesData::<Test>::get(subnet_id, subnet_node_id);
    assert_eq!(subnet_node.classification.class, SubnetNodeClass::Deactivated);
  });
}


#[test]
fn test_deactivation_ledger_as_attestor() {
  new_test_ext().execute_with(|| {
    let subnet_path: Vec<u8> = "petals-team/StableBeluga2".into();
    let deposit_amount: u128 = 10000000000000000000000;
    let amount: u128 = 1000000000000000000000;

    let n_peers = 8;
    let stake_amount: u128 = MinStakeBalance::<Test>::get();

    build_activated_subnet(subnet_path.clone(), 0, n_peers, deposit_amount, stake_amount);

    let subnet_id = SubnetPaths::<Test>::get(subnet_path.clone()).unwrap();
    let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);

    let epoch_length = EpochLength::get();
    let epoch = System::block_number() / epoch_length;

    // insert node as validator to place them into the ledger
    SubnetRewardsValidator::<Test>::insert(subnet_id, epoch, 1);
    let validator_id = SubnetRewardsValidator::<Test>::get(subnet_id, epoch);
    let mut validator = SubnetNodeIdHotkey::<Test>::get(subnet_id, validator_id.unwrap()).unwrap();

    let subnet_node_data_vec = subnet_node_data(0, total_subnet_nodes);

    assert_ok!(
      Network::validate(
        RuntimeOrigin::signed(account(1)), 
        subnet_id,
        subnet_node_data_vec.clone(),
        None,
      )
    );

    // Attest
    for n in 1..total_subnet_nodes+1 {
      if account(n) == validator.clone() {
        continue
      }
      assert_ok!(
        Network::attest(
          RuntimeOrigin::signed(account(n)), 
          subnet_id,
        )
      );
    }

    let subnet_node_id = HotkeySubnetNodeId::<Test>::get(subnet_id, account(1)).unwrap();

    assert_ok!(
      Network::deactivate_subnet_node(
        RuntimeOrigin::signed(account(1)),
        subnet_id,
        subnet_node_id,
      )
    );
  
    let subnet_node = SubnetNodesData::<Test>::get(subnet_id, subnet_node_id);
    assert_eq!(subnet_node.classification.class, SubnetNodeClass::Validator);

    let subnet_node_deactivation_validator = SubnetNodeDeactivation {
      subnet_id: subnet_id,
      subnet_node_id: subnet_node_id,
    };
    let deactivation_ledger = DeactivationLedger::<Test>::get();
    assert_ne!(deactivation_ledger.get(&subnet_node_deactivation_validator), None);

    Network::do_deactivation_ledger();
    
    let subnet_node = SubnetNodesData::<Test>::get(subnet_id, subnet_node_id);
    assert_eq!(subnet_node.classification.class, SubnetNodeClass::Deactivated);

    let subnet_node_deactivation_deactivated = SubnetNodeDeactivation {
      subnet_id: subnet_id,
      subnet_node_id: subnet_node_id,
    };

    let deactivation_ledger = DeactivationLedger::<Test>::get();
    assert_eq!(deactivation_ledger.get(&subnet_node_deactivation_validator), None);
    assert_eq!(deactivation_ledger.get(&subnet_node_deactivation_deactivated), None);
  });
}

#[test]
fn test_deactivation_ledger_as_chosen_validator() {
  new_test_ext().execute_with(|| {
    let subnet_path: Vec<u8> = "petals-team/StableBeluga2".into();
    let deposit_amount: u128 = 10000000000000000000000;
    let amount: u128 = 1000000000000000000000;

    let n_peers = 8;
    let stake_amount: u128 = MinStakeBalance::<Test>::get();

    build_activated_subnet(subnet_path.clone(), 0, n_peers, deposit_amount, stake_amount);

    let subnet_id = SubnetPaths::<Test>::get(subnet_path.clone()).unwrap();

    // let mut ledger = BTreeSet::new();

    // let subnet_node = SubnetNode {
    //   coldkey: account(1),
    //   hotkey: account(1),
    //   peer_id: peer(1),
    //   classification: SubnetNodeClassification {
    //     class: SubnetNodeClass::Validator,
    //     start_epoch: 1,
    //   },
    //   a: Vec::new(),
    //   b: Vec::new(),
    //   c: Vec::new(),
    // };

    let epoch_length = EpochLength::get();
    let epoch = System::block_number() / epoch_length;

    // insert node as validator to place them into the ledger
    SubnetRewardsValidator::<Test>::insert(subnet_id, epoch, 1);

    let subnet_node_id = HotkeySubnetNodeId::<Test>::get(subnet_id, account(1)).unwrap();

    assert_ok!(
      Network::deactivate_subnet_node(
        RuntimeOrigin::signed(account(1)),
        subnet_id,
        subnet_node_id,
      )
    );
  
    let subnet_node_id = HotkeySubnetNodeId::<Test>::get(subnet_id, account(1)).unwrap();
    let subnet_node = SubnetNodesData::<Test>::get(subnet_id, subnet_node_id);
    assert_eq!(subnet_node.classification.class, SubnetNodeClass::Validator);
    let subnet_node_deactivation_validator = SubnetNodeDeactivation {
      subnet_id: subnet_id,
      subnet_node_id: subnet_node_id,
    };
    let deactivation_ledger = DeactivationLedger::<Test>::get();
    assert_ne!(deactivation_ledger.get(&subnet_node_deactivation_validator), None);

    Network::do_deactivation_ledger();
    
    let subnet_node = SubnetNodesData::<Test>::get(subnet_id, subnet_node_id);
    assert_eq!(subnet_node.classification.class, SubnetNodeClass::Deactivated);

    let subnet_node_deactivation_deactivated = SubnetNodeDeactivation {
      subnet_id: subnet_id,
      subnet_node_id: subnet_node_id,
    };

    let deactivation_ledger = DeactivationLedger::<Test>::get();
    assert_eq!(deactivation_ledger.get(&subnet_node_deactivation_validator), None);
    assert_eq!(deactivation_ledger.get(&subnet_node_deactivation_deactivated), None);

    // ledger.insert(
    //   SubnetNodeDeactivation {
    //     subnet_id: 1,
    //     subnet_node_id: subnet_node_id,
    //   }
    // );

    // DeactivationLedger::<Test>::set(ledger);

  });
}

#[test]
fn test_update_delegate_reward_rate() {
  new_test_ext().execute_with(|| {
    let subnet_path: Vec<u8> = "petals-team/StableBeluga2".into();
    let deposit_amount: u128 = 10000000000000000000000;
    let amount: u128 = 1000000000000000000000;

    let n_peers = 8;
    let stake_amount: u128 = MinStakeBalance::<Test>::get();

    build_activated_subnet(subnet_path.clone(), 0, n_peers, deposit_amount, stake_amount);

    let subnet_id = SubnetPaths::<Test>::get(subnet_path.clone()).unwrap();
    let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);
    let subnet_node_id = HotkeySubnetNodeId::<Test>::get(subnet_id, account(1)).unwrap();

    let subnet_node = SubnetNodesData::<Test>::get(subnet_id, subnet_node_id);
    assert_eq!(subnet_node.delegate_reward_rate, 0);
    assert_eq!(subnet_node.last_delegate_reward_rate_update, 0);


    let max_reward_rate_decrease = MaxRewardRateDecrease::<Test>::get();
    let reward_rate_update_period = RewardRateUpdatePeriod::<Test>::get();
    let new_delegate_reward_rate = 50_000_000;

    System::set_block_number(System::block_number() + reward_rate_update_period);

    let block_number = System::block_number();

    // Increase reward rate to 5% then test decreasing
    assert_ok!(
      Network::update_delegate_reward_rate(
        RuntimeOrigin::signed(account(1)),
        subnet_id,
        subnet_node_id,
        new_delegate_reward_rate
      )
    );
  
    let subnet_node = SubnetNodesData::<Test>::get(subnet_id, subnet_node_id);
    assert_eq!(subnet_node.delegate_reward_rate, new_delegate_reward_rate);
    assert_eq!(subnet_node.last_delegate_reward_rate_update, block_number);

    System::set_block_number(System::block_number() + reward_rate_update_period);

    let new_delegate_reward_rate = new_delegate_reward_rate - max_reward_rate_decrease;

    // allow decreasing by 1%
    assert_ok!(
      Network::update_delegate_reward_rate(
        RuntimeOrigin::signed(account(1)),
        subnet_id,
        subnet_node_id,
        new_delegate_reward_rate
      )
    );

    assert_err!(
      Network::update_delegate_reward_rate(
        RuntimeOrigin::signed(account(1)),
        subnet_id,
        subnet_node_id,
        1000000001
      ),
      Error::<Test>::InvalidDelegateRewardRate
    );

    assert_err!(
      Network::update_delegate_reward_rate(
        RuntimeOrigin::signed(account(1)),
        subnet_id,
        subnet_node_id,
        new_delegate_reward_rate+1
      ),
      Error::<Test>::MaxRewardRateUpdates
    );

    System::set_block_number(System::block_number() + reward_rate_update_period);

    assert_err!(
      Network::update_delegate_reward_rate(
        RuntimeOrigin::signed(account(1)),
        subnet_id,
        subnet_node_id,
        new_delegate_reward_rate
      ),
      Error::<Test>::NoDelegateRewardRateChange
    );

  });
}
