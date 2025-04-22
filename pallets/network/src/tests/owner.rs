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
    
    let deposit_amount: u128 = 10000000000000000000000;
    let amount: u128 = 1000000000000000000000;

    let stake_amount: u128 = MinStakeBalance::<Test>::get();

    build_activated_subnet(subnet_path.clone(), 0, 0, deposit_amount, stake_amount);

    let subnet_id = SubnetPaths::<Test>::get(subnet_path.clone()).unwrap();
    let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);

    let n_account = total_subnet_nodes + 1;

  })
}