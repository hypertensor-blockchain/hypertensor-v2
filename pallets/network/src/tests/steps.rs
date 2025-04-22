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
  HotkeySubnetNodeId,
  SubnetRewardsValidator,
  SubnetNodeIdHotkey,
  SubnetRewardsSubmission,
  SubnetNodesData,
  SubnetNodeClass,
};

// #[test]
// fn test_epoch_steps() {
//   new_test_ext().execute_with(|| {
//     let subnet_path: Vec<u8> = "petals-team/StableBeluga2".into();
//     let deposit_amount: u128 = 10000000000000000000000;
//     let amount: u128 = 1000000000000000000000;

//     let n_peers = 8;
//     build_activated_subnet(subnet_path.clone(), 0, n_peers, deposit_amount, amount);

//     let subnet_id = SubnetPaths::<Test>::get(subnet_path.clone()).unwrap();

//     let epoch_length = EpochLength::get();
//     let epoch = System::block_number() / epoch_length;

//     let included_nodes: BTreeSet<<Test as frame_system::Config>::AccountId> = Network::get_classified_hotkeys(subnet_id, &SubnetNodeClass::Included, epoch);
//     let initial_total_subnet_nodes = included_nodes.len() as u32;

//     for n in 1..6+1 {
//       let epoch = System::block_number() / epoch_length;
//       log::error!("test_epoch_steps epoch: {:?}", epoch);

//       Network::do_epoch_preliminaries(System::block_number(), epoch, epoch_length);
      
//       let included_nodes: BTreeSet<<Test as frame_system::Config>::AccountId> = Network::get_classified_hotkeys(subnet_id, &SubnetNodeClass::Included, epoch);
//       let included_nodes_count = included_nodes.len() as u128;

//       log::error!("included_nodes_count {:?}", included_nodes_count);
  
//       let submittable_nodes: BTreeSet<<Test as frame_system::Config>::AccountId> = Network::get_classified_hotkeys(subnet_id, &SubnetNodeClass::Validator, epoch);
//       let submittable_nodes_count = submittable_nodes.len() as u128;

//       // --- Get validator
//       let validator_id = SubnetRewardsValidator::<Test>::get(subnet_id, epoch).unwrap();
//       let mut validator = SubnetNodeIdHotkey::<Test>::get(subnet_id, validator_id).unwrap();

//       let subnet_node_data_vec = subnet_node_data(0, (included_nodes_count) as u32);
//       let subnet_node_data_vec_len = subnet_node_data_vec.len();
//       log::error!("subnet_node_data_vec_len {:?}", subnet_node_data_vec_len);

//       for node in &subnet_node_data_vec {

//         if node.peer_id == peer(n_peers) {
//           log::error!("node in subnet_node_data_vec");

//         }
//       }

//       assert_ok!(
//         Network::validate(
//           RuntimeOrigin::signed(validator.clone()), 
//           subnet_id,
//           subnet_node_data_vec.clone(),
//           None,
//         )
//       );
  
//       // Attest
//       for n in 1..(submittable_nodes_count as u32)+1 {
//         if account(n) == validator.clone() {
//           continue
//         }
//         assert_ok!(
//           Network::attest(
//             RuntimeOrigin::signed(account(n)), 
//             subnet_id,
//           )
//         );
//       }
      
//       let submission = SubnetRewardsSubmission::<Test>::get(subnet_id, epoch).unwrap();
  
//       assert_eq!(submission.validator_id, validator_id);
//       assert_eq!(submission.data.len(), subnet_node_data_vec.len());
//       assert_eq!(submission.attests.len(), submittable_nodes_count as usize);

//       Network::reward_subnets(System::block_number(), epoch);

//       // Add new subnet node and check if they're inclusion on next epoch      
//       if n == 0 {
//         let _ = Balances::deposit_creating(&account(n_peers), deposit_amount);
//         assert_ok!(
//           Network::add_subnet_node(
//             RuntimeOrigin::signed(account(n_peers)),
//             subnet_id,
//             account(n_peers),
//             peer(n_peers),
//             0,
//             amount,
//             None,
//             None,
//             None,
//           ) 
//         );
//         // activated as Queue
//         let subnet_node_id = HotkeySubnetNodeId::<Test>::get(subnet_id, account(n_peers)).unwrap();
//         let subnet_node = SubnetNodesData::<Test>::get(subnet_id, subnet_node_id);
//         assert_eq!(subnet_node.classification.class, SubnetNodeClass::Queue);    
//       } else if n == 1 {
//         let subnet_node_id = HotkeySubnetNodeId::<Test>::get(subnet_id, account(n_peers)).unwrap();
//         // automatically upgraded to Included after first next epoch
//         let subnet_node = SubnetNodesData::<Test>::get(subnet_id, subnet_node_id);
//         assert_eq!(subnet_node.classification.class, SubnetNodeClass::Included);    
//       } else if n == 2 {
//         let subnet_node_id = HotkeySubnetNodeId::<Test>::get(subnet_id, account(n_peers)).unwrap();
//         // automatically upgraded to Validator after first next epoch
//         let subnet_node = SubnetNodesData::<Test>::get(subnet_id, subnet_node_id);
//         assert_eq!(subnet_node.classification.class, SubnetNodeClass::Validator);    
//       } else {
//         let subnet_node_id = HotkeySubnetNodeId::<Test>::get(subnet_id, account(n_peers)).unwrap();
//         let subnet_node = SubnetNodesData::<Test>::get(subnet_id, subnet_node_id);
//         assert_eq!(subnet_node.classification.class, SubnetNodeClass::Validator);    
//       }
  
//       increase_epochs(1);
//     }
//   });
// }
