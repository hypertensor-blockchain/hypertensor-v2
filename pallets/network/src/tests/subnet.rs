use super::mock::*;
use crate::tests::test_utils::*;
use crate::Event;
use frame_support::{
	assert_noop, assert_ok, assert_err
};
use log::info;
use frame_support::traits::{OnInitialize, Currency};
use sp_std::collections::btree_set::BTreeSet;
use crate::{
  Error,
  SubnetPaths, 
  MinSubnetNodes, 
  TotalSubnetNodes,
  SubnetsData,
  RegistrationSubnetData,
  SubnetRemovalReason,
  MinSubnetRegistrationBlocks, 
  MaxSubnetRegistrationBlocks, 
  SubnetActivationEnactmentBlocks,
  HotkeySubnetNodeId,
  SubnetRegistrationEpochs,
  SubnetState,
  MinStakeBalance,
};

//
//
//
//
//
//
//
// Subnets Add/Remove
//
//
//
//
//
//
//

#[test]
fn test_register_subnet() {
  new_test_ext().execute_with(|| {
    let subnet_path: Vec<u8> = "petals-team/StableBeluga2".into();

    let epoch_length = EpochLength::get();
    let block_number = System::block_number();
    let epoch = System::block_number().saturating_div(epoch_length);
  
    let cost = Network::registration_cost(epoch);
  
    let _ = Balances::deposit_creating(&account(0), cost+1000);
  
    let min_nodes = MinSubnetNodes::<Test>::get();
  
    let whitelist = get_coldkey_whitelist(0, min_nodes+1);
  
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
        RuntimeOrigin::signed(account(0)),
        add_subnet_data,
      )
    );
  
    let subnet_id = SubnetPaths::<Test>::get(subnet_path.clone()).unwrap();
    let subnet = SubnetsData::<Test>::get(subnet_id).unwrap();

    // Check treasury pot
    let minimum_balance = Balances::minimum_balance();
    let pot = Treasury::pot();
    assert_eq!(cost, pot + minimum_balance);
  })
}

#[test]
fn test_register_subnet_subnet_registration_cooldown() {
  new_test_ext().execute_with(|| {
    let subnet_path: Vec<u8> = "petals-team/StableBeluga2".into();

    increase_epochs(1);

    let epoch_length = EpochLength::get();
    let block_number = System::block_number();
    let epoch = System::block_number().saturating_div(epoch_length);
  
    let cost = Network::registration_cost(epoch);
  
    let _ = Balances::deposit_creating(&account(0), cost+1000);
  
    let min_nodes = MinSubnetNodes::<Test>::get();

    let whitelist = get_coldkey_whitelist(0, min_nodes+1);

    let add_subnet_data = RegistrationSubnetData {
      path: subnet_path.clone().into(),
      max_node_registration_epochs: 16,
      node_registration_interval: 0,
      node_activation_interval: 0,
      node_queue_period: 1,
      max_node_penalties: 3,
      coldkey_whitelist: whitelist.clone(),
      // coldkey_whitelist: None,
    };
  
    let epoch_length = EpochLength::get();
    let block_number = System::block_number();
    let epoch = System::block_number().saturating_div(epoch_length);
    let next_registration_epoch = Network::get_next_registration_epoch(epoch);
    // increase_epochs(next_registration_epoch - epoch);

    // --- Register subnet for activation
    assert_ok!(
      Network::register_subnet(
        RuntimeOrigin::signed(account(0)),
        add_subnet_data,
      )
    );
  
    let subnet_id = SubnetPaths::<Test>::get(subnet_path.clone()).unwrap();
    let subnet = SubnetsData::<Test>::get(subnet_id).unwrap();
  
    let subnet_path: Vec<u8> = "petals-team/StableBeluga3".into();

    let add_subnet_data = RegistrationSubnetData {
      path: subnet_path.clone().into(),
      max_node_registration_epochs: 16,
      node_registration_interval: 0,
      node_activation_interval: 0,
      node_queue_period: 1,
      max_node_penalties: 3,
      coldkey_whitelist: whitelist.clone(),
      // coldkey_whitelist: None,
    };

    assert_err!(
      Network::register_subnet(
        RuntimeOrigin::signed(account(0)),
        add_subnet_data.clone(),
      ),
      Error::<Test>::SubnetRegistrationCooldown
    );

    let epoch_length = EpochLength::get();
    let block_number = System::block_number();
    let epoch = System::block_number().saturating_div(epoch_length);
    let next_registration_epoch = Network::get_next_registration_epoch(epoch);
    increase_epochs(next_registration_epoch);

    let epoch_length = EpochLength::get();
    let block_number = System::block_number();
    let epoch = System::block_number().saturating_div(epoch_length);
  
    let cost = Network::registration_cost(epoch);
  
    let _ = Balances::deposit_creating(&account(0), cost+1000);

    // --- Register after cooldown
    assert_ok!(
      Network::register_subnet(
        RuntimeOrigin::signed(account(0)),
        add_subnet_data.clone(),
      )
    );


    let subnet_path: Vec<u8> = "petals-team/StableBeluga4".into();

    let add_subnet_data = RegistrationSubnetData {
      path: subnet_path.clone().into(),
      max_node_registration_epochs: 16,
      node_registration_interval: 0,
      node_activation_interval: 0,
      node_queue_period: 1,
      max_node_penalties: 3,
      coldkey_whitelist: whitelist.clone(),
      // coldkey_whitelist: None,
    };

    assert_err!(
      Network::register_subnet(
        RuntimeOrigin::signed(account(0)),
        add_subnet_data.clone(),
      ),
      Error::<Test>::SubnetRegistrationCooldown
    );
  })
}

#[test]
fn test_register_subnet_exists_error() {
  new_test_ext().execute_with(|| {
    let subnet_path: Vec<u8> = "petals-team/StableBeluga2".into();

    let epoch_length = EpochLength::get();
    let block_number = System::block_number();
    let epoch = System::block_number().saturating_div(epoch_length);
  
    let cost = Network::registration_cost(epoch);
  
    let _ = Balances::deposit_creating(&account(0), cost+1000);
  
    let min_nodes = MinSubnetNodes::<Test>::get();

    let whitelist = get_coldkey_whitelist(0, min_nodes+1);

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
        RuntimeOrigin::signed(account(0)),
        add_subnet_data.clone(),
      )
    );
  
    assert_err!(
      Network::register_subnet(
        RuntimeOrigin::signed(account(0)),
        add_subnet_data.clone(),
      ),
      Error::<Test>::SubnetExist
    );

  })
}

// #[test]
// fn test_register_subnet_registration_blocks_err() {
//   new_test_ext().execute_with(|| {
//     let subnet_path: Vec<u8> = "petals-team/StableBeluga2".into();

//     let epoch_length = EpochLength::get();
//     let block_number = System::block_number();
//     let epoch = System::block_number().saturating_div(epoch_length);
  
//     let cost = Network::registration_cost(epoch);
  
//     let _ = Balances::deposit_creating(&account(0), cost+1000);
  
//     let add_subnet_data = RegistrationSubnetData {
//       path: subnet_path.clone().into(),
//       max_node_registration_epochs: 16,
//       node_registration_interval: 0,
//       // coldkey_whitelist: Some(BTreeSet::new()),
//       coldkey_whitelist: None,
//     };
    
//     let epoch_length = EpochLength::get();
//     let block_number = System::block_number();
//     let epoch = System::block_number().saturating_div(epoch_length);
//     let next_registration_epoch = Network::get_next_registration_epoch(epoch);
//     increase_epochs(next_registration_epoch - epoch);

//     assert_err!(
//       Network::register_subnet(
//         RuntimeOrigin::signed(account(0)),
//         add_subnet_data,
//       ),
//       Error::<Test>::InvalidSubnetRegistrationBlocks
//     );

//     let add_subnet_data = RegistrationSubnetData {
//       path: subnet_path.clone().into(),
//       max_node_registration_epochs: 16,
//       node_registration_interval: 0,
//       // coldkey_whitelist: Some(BTreeSet::new()),
//       coldkey_whitelist: None,
//     };

//     assert_err!(
//       Network::register_subnet(
//         RuntimeOrigin::signed(account(0)),
//         add_subnet_data,
//       ),
//       Error::<Test>::InvalidSubnetRegistrationBlocks
//     );
//   })
// }

// #[test]
// fn test_register_subnet_max_total_subnet_mem_err() {
//   new_test_ext().execute_with(|| {
//     let epoch_length = EpochLength::get();
//     let block_number = System::block_number();
//     let epoch = System::block_number().saturating_div(epoch_length);
  
//     let cost = Network::registration_cost(epoch);
    
//     let total_subnet_memory_mb = TotalSubnetMemoryMB::<Test>::get();

//     // Limit while loop to 10 ierations
//     let iterations = 11;
//     let epoch_length = EpochLength::get();

//     let mut current_total_subnet_memory_mb = total_subnet_memory_mb;

//     for n in 0..iterations {
//       let epoch_length = EpochLength::get();
//       let block_number = System::block_number();
//       let epoch = System::block_number().saturating_div(epoch_length);
//       let next_registration_epoch = Network::get_next_registration_epoch(epoch);
//       increase_epochs(next_registration_epoch - epoch);
  
//       let _ = Balances::deposit_creating(&account(0), cost+1000);

//       let path: Vec<u8> = format!("model-name-{n}").into(); 

//       let add_subnet_data = RegistrationSubnetData {
//         path: path,
//         max_node_registration_epochs: 16,
//         node_registration_interval: 0,
      // coldkey_whitelist: Some(BTreeSet::new()),
      // coldkey_whitelist: None,
//       };

//       let next_subnet_total_memory_mb = TotalSubnetMemoryMB::<Test>::get() + subnet_mem_mb;

//       if next_subnet_total_memory_mb <= max_total_subnet_memory_mb {
//         assert_ok!(
//           Network::register_subnet(
//             RuntimeOrigin::signed(account(0)),
//             add_subnet_data,
//           )
//         );
//       } else {
//         assert_err!(
//           Network::register_subnet(
//             RuntimeOrigin::signed(account(0)),
//             add_subnet_data,
//           ),
//           Error::<Test>::MaxTotalSubnetMemory
//         );
//       }
//     }
//   })
// }

#[test]
fn test_register_subnet_not_enough_balance_err() {
  new_test_ext().execute_with(|| {
    // let _ = Balances::deposit_creating(&account(0), cost+1000);  
    let subnet_path: Vec<u8> = "petals-team/StableBeluga2".into();

    let min_nodes = MinSubnetNodes::<Test>::get();

    let whitelist = get_coldkey_whitelist(0, min_nodes+1);

    let add_subnet_data = RegistrationSubnetData {
      path: subnet_path.into(),
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

    assert_err!(
      Network::register_subnet(
        RuntimeOrigin::signed(account(0)),
        add_subnet_data,
      ),
      Error::<Test>::NotEnoughBalanceToStake
    );
  })
}

#[test]
fn test_activate_subnet() {
  new_test_ext().execute_with(|| {
    let subnet_path: Vec<u8> = "petals-team/StableBeluga2".into();

    let epoch_length = EpochLength::get();
    let block_number = System::block_number();
    let epoch = System::block_number().saturating_div(epoch_length);
  
    let cost = Network::registration_cost(epoch);
  
    let _ = Balances::deposit_creating(&account(0), cost+1000);
  
    let min_nodes = MinSubnetNodes::<Test>::get();

    let whitelist = get_coldkey_whitelist(0, min_nodes+1);

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
        RuntimeOrigin::signed(account(0)),
        add_subnet_data,
      )
    );
  
    let subnet_id = SubnetPaths::<Test>::get(subnet_path.clone()).unwrap();
    let subnet = SubnetsData::<Test>::get(subnet_id).unwrap();
  
    let id = subnet.id;
		let path = subnet.path;
		let min_nodes = MinSubnetNodes::<Test>::get();

    // --- Add subnet nodes
    let deposit_amount: u128 = 10000000000000000000000;
    let amount: u128 = 1000000000000000000000;
    for n in 1..min_nodes+1 {
      let _ = Balances::deposit_creating(&account(n), deposit_amount);
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
    }
  
    let min_subnet_delegate_stake = Network::get_min_subnet_delegate_stake_balance();
    // --- Add the minimum required delegate stake balance to activate the subnet
    assert_ok!(
      Network::add_to_delegate_stake(
        RuntimeOrigin::signed(account(1)),
        subnet_id,
        min_subnet_delegate_stake,
      ) 
    );

    // --- Increase blocks to max registration block
    let epochs = SubnetRegistrationEpochs::<Test>::get();
    increase_epochs(epochs + 1);
    let current_epoch = get_epoch();

    assert_ok!(
      Network::activate_subnet(
        RuntimeOrigin::signed(account(0)),
        subnet_id,
      )
    );

    let subnet_id = SubnetPaths::<Test>::get(subnet_path.clone()).unwrap();
    let subnet = SubnetsData::<Test>::get(subnet_id).unwrap();
    assert_eq!(subnet.id, subnet_id);

    // ensure subnet exists and nothing changed but the activation block
    assert_eq!(subnet.id, id);
    assert_eq!(subnet.path, path);
    assert_eq!(subnet.state, SubnetState::Active);
  })
}

#[test]
fn test_activate_subnet_invalid_subnet_id_error() {
  new_test_ext().execute_with(|| {
    let subnet_path: Vec<u8> = "petals-team/StableBeluga2".into();

    let epoch_length = EpochLength::get();
    let block_number = System::block_number();
    let epoch = System::block_number().saturating_div(epoch_length);
  
    let cost = Network::registration_cost(epoch);
  
    let _ = Balances::deposit_creating(&account(0), cost+1000);
  
    let min_nodes = MinSubnetNodes::<Test>::get();

    let whitelist = get_coldkey_whitelist(0, min_nodes+1);

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
        RuntimeOrigin::signed(account(0)),
        add_subnet_data,
      )
    );
  
    let subnet_id = SubnetPaths::<Test>::get(subnet_path.clone()).unwrap();
    let subnet = SubnetsData::<Test>::get(subnet_id).unwrap();
  
    let id = subnet.id;
		let path = subnet.path;
		let min_nodes = MinSubnetNodes::<Test>::get();

    // --- Add subnet nodes
    let deposit_amount: u128 = 10000000000000000000000;
    let amount: u128 = 1000000000000000000000;
    for n in 1..min_nodes+1 {
      let _ = Balances::deposit_creating(&account(n), deposit_amount);
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
    }
  
    assert_err!(
      Network::activate_subnet(
        RuntimeOrigin::signed(account(0)),
        subnet_id+1,
      ),
      Error::<Test>::InvalidSubnetId
    );
  })
}

#[test]
fn test_activate_subnet_already_activated_err() {
  new_test_ext().execute_with(|| {
    let subnet_path: Vec<u8> = "petals-team/StableBeluga2".into();

    let epoch_length = EpochLength::get();
    let block_number = System::block_number();
    let epoch = System::block_number().saturating_div(epoch_length);
  
    let cost = Network::registration_cost(epoch);
  
    let _ = Balances::deposit_creating(&account(0), cost+1000);
  
    let min_nodes = MinSubnetNodes::<Test>::get();

    let whitelist = get_coldkey_whitelist(0, min_nodes+1);

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
        RuntimeOrigin::signed(account(0)),
        add_subnet_data,
      )
    );
  
    let subnet_id = SubnetPaths::<Test>::get(subnet_path.clone()).unwrap();
    let subnet = SubnetsData::<Test>::get(subnet_id).unwrap();
  
    let id = subnet.id;
		let path = subnet.path;
		let min_nodes = MinSubnetNodes::<Test>::get();

    // --- Add subnet nodes
    let deposit_amount: u128 = 10000000000000000000000;
    let amount: u128 = 1000000000000000000000;
    for n in 1..min_nodes+1 {
      let _ = Balances::deposit_creating(&account(n), deposit_amount);
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
    }
  
    let min_subnet_delegate_stake = Network::get_min_subnet_delegate_stake_balance();
    // --- Add the minimum required delegate stake balance to activate the subnet
    assert_ok!(
      Network::add_to_delegate_stake(
        RuntimeOrigin::signed(account(1)),
        subnet_id,
        min_subnet_delegate_stake,
      ) 
    );

    // --- Increase blocks to max registration block
    let epochs = SubnetRegistrationEpochs::<Test>::get();
    increase_epochs(epochs + 1);
    let current_epoch = get_epoch();

    assert_ok!(
      Network::activate_subnet(
        RuntimeOrigin::signed(account(0)),
        subnet_id,
      )
    );

    assert_err!(
      Network::activate_subnet(
        RuntimeOrigin::signed(account(0)),
        subnet_id,
      ),
      Error::<Test>::SubnetActivatedAlready
    );
  })
}

#[test]
fn test_activate_subnet_enactment_period_remove_subnet() {
  new_test_ext().execute_with(|| {
    let subnet_path: Vec<u8> = "petals-team/StableBeluga2".into();

    let epoch_length = EpochLength::get();
    let block_number = System::block_number();
    let epoch = System::block_number().saturating_div(epoch_length);
  
    let cost = Network::registration_cost(epoch);
  
    let _ = Balances::deposit_creating(&account(0), cost+1000);
  
    let min_nodes = MinSubnetNodes::<Test>::get();

    let whitelist = get_coldkey_whitelist(0, min_nodes+1);

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
        RuntimeOrigin::signed(account(0)),
        add_subnet_data,
      )
    );
  
    let subnet_id = SubnetPaths::<Test>::get(subnet_path.clone()).unwrap();
    let subnet = SubnetsData::<Test>::get(subnet_id).unwrap();

    let id = subnet.id;
		let path = subnet.path;
		let min_nodes = MinSubnetNodes::<Test>::get();

    // --- Add subnet nodes
    let deposit_amount: u128 = 10000000000000000000000;
    let amount: u128 = 1000000000000000000000;
    for n in 1..min_nodes+1 {
      let _ = Balances::deposit_creating(&account(n), deposit_amount);
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
    }
  
    let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);

    let min_subnet_delegate_stake = Network::get_min_subnet_delegate_stake_balance();
    // --- Add the minimum required delegate stake balance to activate the subnet
    assert_ok!(
      Network::add_to_delegate_stake(
        RuntimeOrigin::signed(account(1)),
        subnet_id,
        min_subnet_delegate_stake,
      ) 
    );

    // --- Increase blocks to max registration block
    let registration_epochs = SubnetRegistrationEpochs::<Test>::get();
    let enactment_epochs = SubnetActivationEnactmentBlocks::<Test>::get();
    increase_epochs(registration_epochs + enactment_epochs + 1);

    assert_ok!(
      Network::activate_subnet(
        RuntimeOrigin::signed(account(0)),
        subnet_id,
      )
    );

    assert_eq!(
			*network_events().last().unwrap(),
			Event::SubnetDeactivated {
        subnet_id: subnet_id, 
        reason: SubnetRemovalReason::EnactmentPeriod
      }
		);

    let removed_subnet_id = SubnetPaths::<Test>::try_get(subnet_path.clone());
    assert_eq!(removed_subnet_id, Err(()));
    let subnet = SubnetsData::<Test>::try_get(subnet_id);
    assert_eq!(subnet, Err(()));

    // --- Ensure nodes can be removed and unstake
    post_subnet_removal_ensures(subnet_id, subnet_path, 0, total_subnet_nodes);
  })
}


#[test]
fn test_activate_subnet_initializing_error() {
  new_test_ext().execute_with(|| {
    let subnet_path: Vec<u8> = "petals-team/StableBeluga2".into();

    let epoch_length = EpochLength::get();
    let block_number = System::block_number();
    let epoch = System::block_number().saturating_div(epoch_length);
  
    let cost = Network::registration_cost(epoch);
  
    let _ = Balances::deposit_creating(&account(0), cost+1000);
  
    let min_nodes = MinSubnetNodes::<Test>::get();

    let whitelist = get_coldkey_whitelist(0, min_nodes+1);

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
        RuntimeOrigin::signed(account(0)),
        add_subnet_data,
      )
    );
  
    let subnet_id = SubnetPaths::<Test>::get(subnet_path.clone()).unwrap();
    let subnet = SubnetsData::<Test>::get(subnet_id).unwrap();
  
    let id = subnet.id;
		let path = subnet.path;
		let min_nodes = MinSubnetNodes::<Test>::get();

    // --- Add subnet nodes
    let deposit_amount: u128 = 10000000000000000000000;
    let amount: u128 = 1000000000000000000000;
    for n in 1..min_nodes+1 {
      let _ = Balances::deposit_creating(&account(n), deposit_amount);
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
    }
  
    let min_subnet_delegate_stake = Network::get_min_subnet_delegate_stake_balance();
    // --- Add the minimum required delegate stake balance to activate the subnet
    assert_ok!(
      Network::add_to_delegate_stake(
        RuntimeOrigin::signed(account(1)),
        subnet_id,
        min_subnet_delegate_stake,
      ) 
    );

    assert_err!(
      Network::activate_subnet(
        RuntimeOrigin::signed(account(0)),
        subnet_id,
      ),
      Error::<Test>::SubnetInitializing
    );
  })
}

#[test]
fn test_not_subnet_node_owner() {
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
    
    assert_err!(
      Network::add_to_stake(
        RuntimeOrigin::signed(account(total_subnet_nodes+1)),
        subnet_id,
        subnet_node_id,
        account(1),
        amount,
      ),
      Error::<Test>::NotKeyOwner,
    );

    assert_err!(
      Network::add_to_stake(
        RuntimeOrigin::signed(account(total_subnet_nodes+1)),
        subnet_id,
        1,
        account(total_subnet_nodes+1),
        amount,
      ),
      Error::<Test>::NotSubnetNodeOwner,
    );


  });
}

#[test]
fn test_activate_subnet_min_subnet_nodes_remove_subnet() {
  new_test_ext().execute_with(|| {
    let subnet_path: Vec<u8> = "petals-team/StableBeluga2".into();

    let epoch_length = EpochLength::get();
    let block_number = System::block_number();
    let epoch = System::block_number().saturating_div(epoch_length);
  
    let cost = Network::registration_cost(epoch);
  
    let _ = Balances::deposit_creating(&account(0), cost+1000);
  
    let min_nodes = MinSubnetNodes::<Test>::get();

    let whitelist = get_coldkey_whitelist(0, min_nodes+1);

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
        RuntimeOrigin::signed(account(0)),
        add_subnet_data,
      )
    );
  
    let subnet_id = SubnetPaths::<Test>::get(subnet_path.clone()).unwrap();
    let subnet = SubnetsData::<Test>::get(subnet_id).unwrap();
  
    let id = subnet.id;
		let path = subnet.path;
		let min_nodes = MinSubnetNodes::<Test>::get();

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
			Event::SubnetDeactivated {
        subnet_id: subnet_id, 
        reason: SubnetRemovalReason::MinSubnetNodes
      }
		);

    let removed_subnet_id = SubnetPaths::<Test>::try_get(subnet_path.clone());
    assert_eq!(removed_subnet_id, Err(()));
    let subnet = SubnetsData::<Test>::try_get(subnet_id);
    assert_eq!(subnet, Err(()));
  })
}

#[test]
fn test_activate_subnet_min_delegate_balance_remove_subnet() {
  new_test_ext().execute_with(|| {
    let subnet_path: Vec<u8> = "petals-team/StableBeluga2".into();

    let epoch_length = EpochLength::get();
    let block_number = System::block_number();
    let epoch = System::block_number().saturating_div(epoch_length);
  
    let cost = Network::registration_cost(epoch);
  
    let _ = Balances::deposit_creating(&account(0), cost+1000);
  
    let min_nodes = MinSubnetNodes::<Test>::get();

    let whitelist = get_coldkey_whitelist(0, min_nodes+1);

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
        RuntimeOrigin::signed(account(0)),
        add_subnet_data,
      )
    );
  
    let subnet_id = SubnetPaths::<Test>::get(subnet_path.clone()).unwrap();
    let subnet = SubnetsData::<Test>::get(subnet_id).unwrap();
  
    let id = subnet.id;
		let path = subnet.path;
		let min_nodes = MinSubnetNodes::<Test>::get();

    // --- Add subnet nodes
    let deposit_amount: u128 = 10000000000000000000000;
    let amount: u128 = 1000000000000000000000;
    for n in 1..min_nodes+1 {
      let _ = Balances::deposit_creating(&account(n), deposit_amount);
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
    }
  
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
			Event::SubnetDeactivated {
        subnet_id: subnet_id, 
        reason: SubnetRemovalReason::MinSubnetDelegateStake
      }
		);

    let removed_subnet_id = SubnetPaths::<Test>::try_get(subnet_path.clone());
    assert_eq!(removed_subnet_id, Err(()));
    let subnet = SubnetsData::<Test>::try_get(subnet_id);
    assert_eq!(subnet, Err(()));
  })
}