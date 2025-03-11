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
  BaseSubnetNodeMemoryMB,
  MaxSubnetMemoryMB,
  LastSubnetRegistrationEpoch,
  MinSubnetRegistrationFee,
  MaxSubnetRegistrationFee,
  SubnetRegistrationInterval,
};

#[test]
fn test_get_min_subnet_nodes_scaled() {
  new_test_ext().execute_with(|| {
    let base_node_memory: u128 = BaseSubnetNodeMemoryMB::<Test>::get();
    let max_subnet_memory: u128 = MaxSubnetMemoryMB::<Test>::get();

    let step = max_subnet_memory / 100;
    let mut i = step;

    let mut last_min_subnet_nodes = 0;

    while i < max_subnet_memory {
      let min_subnet_nodes = Network::get_min_subnet_nodes(base_node_memory, i);
      log::error!(
        "Min: {:?} Last Min: {:?} step: {:?}", 
        min_subnet_nodes, 
        last_min_subnet_nodes,
        step
      );

      assert!(
        min_subnet_nodes >= last_min_subnet_nodes, 
        "Min: {:?} Last Min: {:?} step: {:?}", 
        min_subnet_nodes, 
        last_min_subnet_nodes,
        step
      );
      last_min_subnet_nodes = min_subnet_nodes;
      i += step;
    }
  });
}

#[test]
fn test_registration_cost() {
  new_test_ext().execute_with(|| {    
    let last_registration_epoch = LastSubnetRegistrationEpoch::<Test>::get();
    let fee_min: u128 = MinSubnetRegistrationFee::<Test>::get();
    let fee_max: u128 = MaxSubnetRegistrationFee::<Test>::get();
    let period: u32 = SubnetRegistrationInterval::<Test>::get();
    increase_epochs(period);

    let epoch_length = EpochLength::get();
    let block_number = System::block_number();
    let epoch = System::block_number().saturating_div(epoch_length);
  
    let cost = Network::registration_cost(0);
    assert_eq!(cost, fee_max);
    log::error!("cost is: {:?}", cost);

    let cost = Network::registration_cost(epoch as u32 + period);
    assert_eq!(cost, fee_min);

    let cost = Network::registration_cost(period*100);
    assert_eq!(cost, fee_min);

    // division is not perfect due to epoch values being so little
    let cycle_epoch = (epoch as u32 + period/2) as u32 % period;
    let decrease_per_epoch = (fee_max.saturating_sub(fee_min)) / period as u128;
    
    let cost = Network::registration_cost(epoch as u32 + period/2);
    assert_eq!(cost, fee_max.saturating_sub(decrease_per_epoch * cycle_epoch as u128));
  })
}

#[test]
fn test_get_next_registration_epoch() {
  new_test_ext().execute_with(|| {
    let last_registration_epoch: u32 = LastSubnetRegistrationEpoch::<Test>::get();
    let subnet_registration_fee_period: u32 = SubnetRegistrationInterval::<Test>::get();

    let next_registration_epoch = Network::get_next_registration_epoch(0);
    assert_eq!(next_registration_epoch, subnet_registration_fee_period);

    let next_registration_epoch = Network::get_next_registration_epoch(subnet_registration_fee_period-1);
    assert_eq!(next_registration_epoch, subnet_registration_fee_period);

  })
}

#[test]
fn test_get_target_subnet_nodes() {
  new_test_ext().execute_with(|| {
    let target_nodes = Network::get_target_subnet_nodes(10);
    log::error!("target_nodes: {:?}", target_nodes);
    assert!(target_nodes < 100);
  });
}

