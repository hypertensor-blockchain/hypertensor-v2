use super::mock::*;
use super::test_utils::*;
use crate::Event;
use sp_core::OpaquePeerId as PeerId;
use frame_support::{
	assert_noop, assert_ok, assert_err
};
use sp_runtime::traits::Header;
use log::info;
use sp_core::{H256, U256};
use frame_support::traits::{OnInitialize, Currency};
use crate::{
  Error,   
  SubnetPaths, 
  SubnetNodeClass,
  VotingPeriod, 
  Proposals, 
  ProposalsCount, 
  ChallengePeriod, 
  VoteType,
  ProposalBidAmount, 
  SubnetNode, 
};
use frame_support::BoundedVec;
use strum::IntoEnumIterator;
use sp_io::crypto::sr25519_sign;
use sp_runtime::{MultiSigner, MultiSignature};
use sp_io::crypto::sr25519_generate;
use frame_support::pallet_prelude::Encode;
use sp_runtime::traits::IdentifyAccount;
use sp_core::Pair;
use sp_std::collections::{btree_map::BTreeMap, btree_set::BTreeSet};

// ///
// ///
// ///
// ///
// ///
// ///
// ///
// /// Proposals
// ///
// ///
// ///
// ///
// ///
// ///
// ///

// // #[test]
// // fn test_propose() {
// // 	new_test_ext().execute_with(|| {
// //     let subnet_path: Vec<u8> = "petals-team/StableBeluga2".into();

// //     build_subnet(subnet_path.clone());

// //     let subnet_id = SubnetPaths::<Test>::get(subnet_path.clone()).unwrap();

// //     let mut n_peers: u32 = Network::max_subnet_nodes();
// //     if n_peers > MAX_SUBNET_NODES {
// //       n_peers = MAX_SUBNET_NODES
// //     }
// //     let deposit_amount: u128 = 10000000000000000000000;
// //     let amount: u128 = 1000000000000000000000;
// //     let amount_staked = build_subnet_nodes(subnet_id, 0, n_peers, deposit_amount, amount);

// //     let epoch_length = EpochLength::get();
// //     let epochs = SubnetNodeClassEpochs::<Test>::get(SubnetNodeClass::Accountant);
// //     System::set_block_number(System::block_number() + epochs * epoch_length + 1);
// //     Network::shift_node_classes(System::block_number(), epoch_length);

// //     let proposal_bid_amount = ProposalBidAmount::<Test>::get();
// //     let plaintiff_starting_balance = Balances::free_balance(&account(0));

// //     let accountant_nodes = SubnetNodesClasses::<Test>::get(subnet_id, SubnetNodeClass::Accountant);

// //     let data = Vec::new();

// //     assert_ok!(
// //       Network::propose(
// //         RuntimeOrigin::signed(account(0)),
// //         subnet_id,
// //         peer(1),
// //         data.clone()
// //       ) 
// //     );
    

// //     // --- Ensure bonded
// //     let plaintiff_after_balance = Balances::free_balance(&account(0));
// //     assert_eq!(plaintiff_starting_balance - proposal_bid_amount, plaintiff_after_balance);


// //     let proposal = Proposals::<Test>::get(subnet_id, 0);
// //     assert_eq!(proposal.subnet_id, subnet_id);
// //     assert_eq!(proposal.plaintiff, account(0));
// //     assert_eq!(proposal.defendant, account(1));
// //     assert_eq!(proposal.plaintiff_bond, proposal_bid_amount);
// //     assert_eq!(proposal.defendant_bond, 0);
// //     assert_eq!(proposal.eligible_voters.len(), accountant_nodes.len());
// //     assert_eq!(proposal.start_block, System::block_number());
// //     assert_eq!(proposal.challenge_block, 0);
// //     assert_eq!(proposal.plaintiff_data, data);
// //     assert_eq!(proposal.defendant_data, data);
// //     assert_eq!(proposal.complete, false);
// //   })
// // }

// // #[test]
// // fn test_propose_subnet_not_exist() {
// // 	new_test_ext().execute_with(|| {
// //     let subnet_path: Vec<u8> = "petals-team/StableBeluga2".into();

// //     build_subnet(subnet_path.clone());

// //     let subnet_id = SubnetPaths::<Test>::get(subnet_path.clone()).unwrap();

// //     let mut n_peers: u32 = Network::max_subnet_nodes();
// //     if n_peers > MAX_SUBNET_NODES {
// //       n_peers = MAX_SUBNET_NODES
// //     }
// //     let deposit_amount: u128 = 10000000000000000000000;
// //     let amount: u128 = 1000000000000000000000;
// //     let amount_staked = build_subnet_nodes(subnet_id, 0, n_peers, deposit_amount, amount);

// //     let epoch_length = EpochLength::get();
// //     let epochs = SubnetNodeClassEpochs::<Test>::get(SubnetNodeClass::Accountant);
// //     System::set_block_number(System::block_number() + epochs * epoch_length + 1);
// //     Network::shift_node_classes(System::block_number(), epoch_length);

// //     assert_err!(
// //       Network::propose(
// //         RuntimeOrigin::signed(account(0)),
// //         2,
// //         peer(1),
// //         Vec::new()
// //       ),
// //       Error::<Test>::SubnetNotExist
// //     );
// //   })
// // }

// // #[test]
// // fn test_propose_subnet_node_not_exist() {
// // 	new_test_ext().execute_with(|| {
// //     let subnet_path: Vec<u8> = "petals-team/StableBeluga2".into();

// //     build_subnet(subnet_path.clone());

// //     let subnet_id = SubnetPaths::<Test>::get(subnet_path.clone()).unwrap();

// //     let mut n_peers: u32 = Network::max_subnet_nodes();
// //     if n_peers > MAX_SUBNET_NODES {
// //       n_peers = MAX_SUBNET_NODES
// //     }
// //     let deposit_amount: u128 = 10000000000000000000000;
// //     let amount: u128 = 1000000000000000000000;
// //     let amount_staked = build_subnet_nodes(subnet_id, 0, n_peers, deposit_amount, amount);

// //     let epoch_length = EpochLength::get();
// //     let epochs = SubnetNodeClassEpochs::<Test>::get(SubnetNodeClass::Accountant);
// //     System::set_block_number(System::block_number() + epochs * epoch_length + 1);
// //     Network::shift_node_classes(System::block_number(), epoch_length);

// //     assert_err!(
// //       Network::propose(
// //         RuntimeOrigin::signed(account(n_peers+1)),
// //         subnet_id,
// //         peer(1),
// //         Vec::new()
// //       ),
// //       Error::<Test>::SubnetNodeNotExist
// //     );
// //   })
// // }

// // #[test]
// // fn test_propose_not_accountant() {
// // 	new_test_ext().execute_with(|| {
// //     let subnet_path: Vec<u8> = "petals-team/StableBeluga2".into();

// //     build_subnet(subnet_path.clone());

// //     let subnet_id = SubnetPaths::<Test>::get(subnet_path.clone()).unwrap();

// //     let mut n_peers: u32 = Network::max_subnet_nodes() - 1;
// //     if n_peers > MAX_SUBNET_NODES {
// //       n_peers = MAX_SUBNET_NODES
// //     }
// //     let deposit_amount: u128 = 10000000000000000000000;
// //     let amount: u128 = 1000000000000000000000;
// //     let amount_staked = build_subnet_nodes(subnet_id, 0, n_peers, deposit_amount, amount);

// //     let epoch_length = EpochLength::get();
// //     let epochs = SubnetNodeClassEpochs::<Test>::get(SubnetNodeClass::Accountant);
// //     System::set_block_number(System::block_number() + epochs * epoch_length + 1);
// //     Network::shift_node_classes(System::block_number(), epoch_length);

// //     let _ = Balances::deposit_creating(&account(n_peers+1), deposit_amount);
// //     assert_ok!(
// //       Network::add_subnet_node(
// //         RuntimeOrigin::signed(account(n_peers+1)),
// //         subnet_id,
// //         peer(n_peers+1),
// //         amount,
// //       ) 
// //     );

// //     assert_err!(
// //       Network::propose(
// //         RuntimeOrigin::signed(account(n_peers+1)),
// //         subnet_id,
// //         peer(1),
// //         Vec::new()
// //       ),
// //       Error::<Test>::NodeAccountantEpochNotReached
// //     );
// //   })
// // }

// // #[test]
// // fn test_propose_peer_id_not_exist() {
// // 	new_test_ext().execute_with(|| {
// //     let subnet_path: Vec<u8> = "petals-team/StableBeluga2".into();

// //     build_subnet(subnet_path.clone());

// //     let subnet_id = SubnetPaths::<Test>::get(subnet_path.clone()).unwrap();

// //     let mut n_peers: u32 = Network::max_subnet_nodes() - 1;
// //     if n_peers > MAX_SUBNET_NODES {
// //       n_peers = MAX_SUBNET_NODES
// //     }
// //     let deposit_amount: u128 = 10000000000000000000000;
// //     let amount: u128 = 1000000000000000000000;
// //     let amount_staked = build_subnet_nodes(subnet_id, 0, n_peers, deposit_amount, amount);

// //     let epoch_length = EpochLength::get();
// //     let epochs = SubnetNodeClassEpochs::<Test>::get(SubnetNodeClass::Accountant);
// //     System::set_block_number(System::block_number() + epochs * epoch_length + 1);
// //     Network::shift_node_classes(System::block_number(), epoch_length);

// //     assert_err!(
// //       Network::propose(
// //         RuntimeOrigin::signed(account(0)),
// //         subnet_id,
// //         peer(n_peers+1),
// //         Vec::new()
// //       ),
// //       Error::<Test>::PeerIdNotExist
// //     );
// //   })
// // }

// // #[test]
// // fn test_propose_min_subnet_nodes_accountants_error() {
// // 	new_test_ext().execute_with(|| {
// //     let subnet_path: Vec<u8> = "petals-team/StableBeluga2".into();

// //     build_subnet(subnet_path.clone());

// //     let subnet_id = SubnetPaths::<Test>::get(subnet_path.clone()).unwrap();

// //     let mut n_peers: u32 = Network::max_subnet_nodes() - 1;
// //     if n_peers > MAX_SUBNET_NODES {
// //       n_peers = MAX_SUBNET_NODES
// //     }
// //     let deposit_amount: u128 = 10000000000000000000000;
// //     let amount: u128 = 1000000000000000000000;

// //     let epoch_length = EpochLength::get();
// //     let epochs = SubnetNodeClassEpochs::<Test>::get(SubnetNodeClass::Accountant);

// //     let _ = Balances::deposit_creating(&account(n_peers+1), deposit_amount);
// //     assert_ok!(
// //       Network::add_subnet_node(
// //         RuntimeOrigin::signed(account(n_peers+1)),
// //         subnet_id,
// //         peer(n_peers+1),
// //         amount,
// //       ) 
// //     );

// //     // Shift node classes to accountant epoch for account(n_peers+1)
// //     System::set_block_number(System::block_number() + epochs * epoch_length + 1);
// //     Network::shift_node_classes(System::block_number(), epoch_length);

// //     // Add new subnet nodes that aren't accountants yet
// //     for n in 0..n_peers {
// //       let _ = Balances::deposit_creating(&account(n), deposit_amount);
// //       assert_ok!(
// //         Network::add_subnet_node(
// //           RuntimeOrigin::signed(account(n)),
// //           subnet_id,
// //           peer(n),
// //           amount,
// //         ) 
// //       );
// //     }
  
// //     assert_err!(
// //       Network::propose(
// //         RuntimeOrigin::signed(account(n_peers+1)),
// //         subnet_id,
// //         peer(1),
// //         Vec::new()
// //       ),
// //       Error::<Test>::SubnetNodesMin
// //     );
// //   })
// // }

// // #[test]
// // fn test_propose_peer_has_active_proposal() {
// // 	new_test_ext().execute_with(|| {
// //     let subnet_path: Vec<u8> = "petals-team/StableBeluga2".into();

// //     build_subnet(subnet_path.clone());

// //     let subnet_id = SubnetPaths::<Test>::get(subnet_path.clone()).unwrap();

// //     let mut n_peers: u32 = Network::max_subnet_nodes();
// //     if n_peers > MAX_SUBNET_NODES {
// //       n_peers = MAX_SUBNET_NODES
// //     }
// //     let deposit_amount: u128 = 10000000000000000000000;
// //     let amount: u128 = 1000000000000000000000;
// //     let amount_staked = build_subnet_nodes(subnet_id, 0, n_peers, deposit_amount, amount);

// //     let epoch_length = EpochLength::get();
// //     let epochs = SubnetNodeClassEpochs::<Test>::get(SubnetNodeClass::Accountant);
// //     System::set_block_number(System::block_number() + epochs * epoch_length + 1);
// //     Network::shift_node_classes(System::block_number(), epoch_length);

// //     assert_ok!(
// //       Network::propose(
// //         RuntimeOrigin::signed(account(0)),
// //         subnet_id,
// //         peer(1),
// //         Vec::new()
// //       ) 
// //     );

// //     assert_err!(
// //       Network::propose(
// //         RuntimeOrigin::signed(account(2)),
// //         subnet_id,
// //         peer(1),
// //         Vec::new()
// //       ),
// //       Error::<Test>::NodeHasActiveProposal
// //     );

// //     assert_err!(
// //       Network::propose(
// //         RuntimeOrigin::signed(account(3)),
// //         subnet_id,
// //         peer(1),
// //         Vec::new()
// //       ),
// //       Error::<Test>::NodeHasActiveProposal
// //     );
// //   })
// // }

// // #[test]
// // fn test_propose_not_enough_balance_to_bid() {
// // 	new_test_ext().execute_with(|| {
// //     let subnet_path: Vec<u8> = "petals-team/StableBeluga2".into();

// //     build_subnet(subnet_path.clone());

// //     let subnet_id = SubnetPaths::<Test>::get(subnet_path.clone()).unwrap();

// //     let mut n_peers: u32 = Network::max_subnet_nodes();
// //     if n_peers > MAX_SUBNET_NODES {
// //       n_peers = MAX_SUBNET_NODES
// //     }
// //     let deposit_amount: u128 = 10000000000000000000000;
// //     let amount: u128 = 1000000000000000000000;
// //     let amount_staked = build_subnet_nodes(subnet_id, 0, n_peers, deposit_amount, amount);

// //     let epoch_length = EpochLength::get();
// //     let epochs = SubnetNodeClassEpochs::<Test>::get(SubnetNodeClass::Accountant);
// //     System::set_block_number(System::block_number() + epochs * epoch_length + 1);
// //     Network::shift_node_classes(System::block_number(), epoch_length);

// //     let proposal_bid_amount = ProposalBidAmount::<Test>::get();
// //     let free_balance = Balances::free_balance(&account(0));

// //     assert_ok!(
// //       Balances::transfer_keep_alive(
// //         RuntimeOrigin::signed(account(0)),
// //         sp_runtime::MultiAddress::Id(account(1)),
// //         free_balance-500,
// //       )  
// //     );

// //     assert_err!(
// //       Network::propose(
// //         RuntimeOrigin::signed(account(0)),
// //         subnet_id,
// //         peer(1),
// //         Vec::new()
// //       ),
// //       Error::<Test>::NotEnoughBalanceToBid
// //     );
// //   })
// // }

// // #[test]
// // fn test_cancel_proposal() {
// // 	new_test_ext().execute_with(|| {
// //     let subnet_path: Vec<u8> = "petals-team/StableBeluga2".into();

// //     build_subnet(subnet_path.clone());

// //     let subnet_id = SubnetPaths::<Test>::get(subnet_path.clone()).unwrap();

// //     let mut n_peers: u32 = Network::max_subnet_nodes();
// //     if n_peers > MAX_SUBNET_NODES {
// //       n_peers = MAX_SUBNET_NODES
// //     }
// //     let deposit_amount: u128 = 10000000000000000000000;
// //     let amount: u128 = 1000000000000000000000;
// //     let amount_staked = build_subnet_nodes(subnet_id, 0, n_peers, deposit_amount, amount);

// //     let epoch_length = EpochLength::get();
// //     let epochs = SubnetNodeClassEpochs::<Test>::get(SubnetNodeClass::Accountant);
// //     System::set_block_number(System::block_number() + epochs * epoch_length + 1);
// //     Network::shift_node_classes(System::block_number(), epoch_length);

// //     assert_ok!(
// //       Network::propose(
// //         RuntimeOrigin::signed(account(0)),
// //         subnet_id,
// //         peer(1),
// //         Vec::new()
// //       ) 
// //     );

// //     let proposal_index = ProposalsCount::<Test>::get() - 1;
// //     let proposal = Proposals::<Test>::get(subnet_id, proposal_index);
// //     let plaintiff_bond = proposal.plaintiff_bond;

// //     let proposer_balance = Balances::free_balance(&account(0));

// //     assert_ok!(
// //       Network::cancel_proposal(
// //         RuntimeOrigin::signed(account(0)),
// //         subnet_id,
// //         proposal_index,
// //       )
// //     );

// //     // --- Ensure proposer gets bond back
// //     let after_cancel_proposer_balance = Balances::free_balance(&account(0));
// //     assert_eq!(proposer_balance + plaintiff_bond, after_cancel_proposer_balance);

// //     let proposal = Proposals::<Test>::try_get(subnet_id, 0);
// //     assert_eq!(proposal, Err(()));
// //   })
// // }

// // #[test]
// // fn test_cancel_proposal_not_plaintiff() {
// // 	new_test_ext().execute_with(|| {
// //     let subnet_path: Vec<u8> = "petals-team/StableBeluga2".into();

// //     build_subnet(subnet_path.clone());

// //     let subnet_id = SubnetPaths::<Test>::get(subnet_path.clone()).unwrap();

// //     let mut n_peers: u32 = Network::max_subnet_nodes();
// //     if n_peers > MAX_SUBNET_NODES {
// //       n_peers = MAX_SUBNET_NODES
// //     }
// //     let deposit_amount: u128 = 10000000000000000000000;
// //     let amount: u128 = 1000000000000000000000;
// //     let amount_staked = build_subnet_nodes(subnet_id, 0, n_peers, deposit_amount, amount);

// //     let epoch_length = EpochLength::get();
// //     let epochs = SubnetNodeClassEpochs::<Test>::get(SubnetNodeClass::Accountant);
// //     System::set_block_number(System::block_number() + epochs * epoch_length + 1);
// //     Network::shift_node_classes(System::block_number(), epoch_length);

// //     assert_ok!(
// //       Network::propose(
// //         RuntimeOrigin::signed(account(0)),
// //         subnet_id,
// //         peer(1),
// //         Vec::new()
// //       ) 
// //     );

// //     let proposal_index = ProposalsCount::<Test>::get() - 1;

// //     assert_err!(
// //       Network::cancel_proposal(
// //         RuntimeOrigin::signed(account(1)),
// //         subnet_id,
// //         proposal_index,
// //       ),
// //       Error::<Test>::NotPlaintiff
// //     );
// //   })
// // }

// // #[test]
// // fn test_cancel_proposal_already_challenged() {
// // 	new_test_ext().execute_with(|| {
// //     let subnet_path: Vec<u8> = "petals-team/StableBeluga2".into();

// //     build_subnet(subnet_path.clone());

// //     let subnet_id = SubnetPaths::<Test>::get(subnet_path.clone()).unwrap();

// //     let mut n_peers: u32 = Network::max_subnet_nodes();
// //     if n_peers > MAX_SUBNET_NODES {
// //       n_peers = MAX_SUBNET_NODES
// //     }
// //     let deposit_amount: u128 = 10000000000000000000000;
// //     let amount: u128 = 1000000000000000000000;
// //     let amount_staked = build_subnet_nodes(subnet_id, 0, n_peers, deposit_amount, amount);

// //     let epoch_length = EpochLength::get();
// //     let epochs = SubnetNodeClassEpochs::<Test>::get(SubnetNodeClass::Accountant);
// //     System::set_block_number(System::block_number() + epochs * epoch_length + 1);
// //     Network::shift_node_classes(System::block_number(), epoch_length);

// //     assert_ok!(
// //       Network::propose(
// //         RuntimeOrigin::signed(account(0)),
// //         subnet_id,
// //         peer(1),
// //         Vec::new()
// //       ) 
// //     );

// //     let proposal_index = ProposalsCount::<Test>::get() - 1;

// //     assert_ok!(
// //       Network::challenge_proposal(
// //         RuntimeOrigin::signed(account(1)),
// //         subnet_id,
// //         proposal_index,
// //         Vec::new()
// //       ) 
// //     );

// //     assert_err!(
// //       Network::cancel_proposal(
// //         RuntimeOrigin::signed(account(0)),
// //         subnet_id,
// //         proposal_index,
// //       ),
// //       Error::<Test>::ProposalChallenged
// //     );
// //   })
// // }

// // #[test]
// // fn test_cancel_proposal_already_complete() {
// // 	new_test_ext().execute_with(|| {
// //     let subnet_path: Vec<u8> = "petals-team/StableBeluga2".into();

// //     build_subnet(subnet_path.clone());

// //     let subnet_id = SubnetPaths::<Test>::get(subnet_path.clone()).unwrap();

// //     let mut n_peers: u32 = Network::max_subnet_nodes();
// //     if n_peers > MAX_SUBNET_NODES {
// //       n_peers = MAX_SUBNET_NODES
// //     }
// //     let deposit_amount: u128 = 10000000000000000000000;
// //     let amount: u128 = 1000000000000000000000;
// //     let amount_staked = build_subnet_nodes(subnet_id, 0, n_peers, deposit_amount, amount);

// //     let epoch_length = EpochLength::get();
// //     let epochs = SubnetNodeClassEpochs::<Test>::get(SubnetNodeClass::Accountant);
// //     System::set_block_number(System::block_number() + epochs * epoch_length + 1);
// //     Network::shift_node_classes(System::block_number(), epoch_length);

// //     assert_ok!(
// //       Network::propose(
// //         RuntimeOrigin::signed(account(0)),
// //         subnet_id,
// //         peer(1),
// //         Vec::new()
// //       ) 
// //     );

// //     let proposal_index = ProposalsCount::<Test>::get() - 1;

// //     assert_ok!(
// //       Network::cancel_proposal(
// //         RuntimeOrigin::signed(account(0)),
// //         subnet_id,
// //         proposal_index,
// //       )
// //     );

// //     // assert_err!(
// //     //   Network::cancel_proposal(
// //     //     RuntimeOrigin::signed(account(0)),
// //     //     subnet_id,
// //     //     proposal_index,
// //     //   ),
// //     //   Error::<Test>::ProposalComplete
// //     // );
// //     assert_err!(
// //       Network::cancel_proposal(
// //         RuntimeOrigin::signed(account(0)),
// //         subnet_id,
// //         proposal_index,
// //       ),
// //       Error::<Test>::ProposalInvalid
// //     );

// //   })
// // }

// // #[test]
// // fn test_challenge_proposal() {
// // 	new_test_ext().execute_with(|| {
// //     let subnet_path: Vec<u8> = "petals-team/StableBeluga2".into();

// //     build_subnet(subnet_path.clone());

// //     let subnet_id = SubnetPaths::<Test>::get(subnet_path.clone()).unwrap();

// //     let mut n_peers: u32 = Network::max_subnet_nodes();
// //     if n_peers > MAX_SUBNET_NODES {
// //       n_peers = MAX_SUBNET_NODES
// //     }
// //     let deposit_amount: u128 = 10000000000000000000000;
// //     let amount: u128 = 1000000000000000000000;
// //     let amount_staked = build_subnet_nodes(subnet_id, 0, n_peers, deposit_amount, amount);

// //     let epoch_length = EpochLength::get();
// //     let epochs = SubnetNodeClassEpochs::<Test>::get(SubnetNodeClass::Accountant);
// //     System::set_block_number(System::block_number() + epochs * epoch_length + 1);
// //     Network::shift_node_classes(System::block_number(), epoch_length);

// //     let proposal_bid_amount = ProposalBidAmount::<Test>::get();

// //     assert_ok!(
// //       Network::propose(
// //         RuntimeOrigin::signed(account(0)),
// //         subnet_id,
// //         peer(1),
// //         Vec::new()
// //       ) 
// //     );

// //     let proposal_index = ProposalsCount::<Test>::get() - 1;
// //     let defendant_starting_balance = Balances::free_balance(&account(1));

// //     assert_ok!(
// //       Network::challenge_proposal(
// //         RuntimeOrigin::signed(account(1)),
// //         subnet_id,
// //         proposal_index,
// //         Vec::new()
// //       ) 
// //     );

// //     // --- Ensure bonded
// //     let defendant_after_balance = Balances::free_balance(&account(1));
// //     assert_eq!(defendant_starting_balance - proposal_bid_amount, defendant_after_balance);
// //   })
// // }

// // #[test]
// // fn test_challenge_proposal_invalid_proposal_id() {
// // 	new_test_ext().execute_with(|| {
// //     let subnet_path: Vec<u8> = "petals-team/StableBeluga2".into();

// //     build_subnet(subnet_path.clone());

// //     let subnet_id = SubnetPaths::<Test>::get(subnet_path.clone()).unwrap();

// //     let mut n_peers: u32 = Network::max_subnet_nodes();
// //     if n_peers > MAX_SUBNET_NODES {
// //       n_peers = MAX_SUBNET_NODES
// //     }
// //     let deposit_amount: u128 = 10000000000000000000000;
// //     let amount: u128 = 1000000000000000000000;
// //     let amount_staked = build_subnet_nodes(subnet_id, 0, n_peers, deposit_amount, amount);

// //     let epoch_length = EpochLength::get();
// //     let epochs = SubnetNodeClassEpochs::<Test>::get(SubnetNodeClass::Accountant);
// //     System::set_block_number(System::block_number() + epochs * epoch_length + 1);
// //     Network::shift_node_classes(System::block_number(), epoch_length);

// //     assert_ok!(
// //       Network::propose(
// //         RuntimeOrigin::signed(account(0)),
// //         subnet_id,
// //         peer(1),
// //         Vec::new()
// //       ) 
// //     );

// //     assert_err!(
// //       Network::challenge_proposal(
// //         RuntimeOrigin::signed(account(0)),
// //         subnet_id,
// //         15,
// //         Vec::new()
// //       ),
// //       Error::<Test>::ProposalInvalid
// //     );
// //   })
// // }

// // #[test]
// // fn test_challenge_proposal_not_defendant() {
// // 	new_test_ext().execute_with(|| {
// //     let subnet_path: Vec<u8> = "petals-team/StableBeluga2".into();

// //     build_subnet(subnet_path.clone());

// //     let subnet_id = SubnetPaths::<Test>::get(subnet_path.clone()).unwrap();

// //     let mut n_peers: u32 = Network::max_subnet_nodes();
// //     if n_peers > MAX_SUBNET_NODES {
// //       n_peers = MAX_SUBNET_NODES
// //     }
// //     let deposit_amount: u128 = 10000000000000000000000;
// //     let amount: u128 = 1000000000000000000000;
// //     let amount_staked = build_subnet_nodes(subnet_id, 0, n_peers, deposit_amount, amount);

// //     let epoch_length = EpochLength::get();
// //     let epochs = SubnetNodeClassEpochs::<Test>::get(SubnetNodeClass::Accountant);
// //     System::set_block_number(System::block_number() + epochs * epoch_length + 1);
// //     Network::shift_node_classes(System::block_number(), epoch_length);

// //     assert_ok!(
// //       Network::propose(
// //         RuntimeOrigin::signed(account(0)),
// //         subnet_id,
// //         peer(1),
// //         Vec::new()
// //       ) 
// //     );

// //     let proposal_index = ProposalsCount::<Test>::get() - 1;

// //     assert_err!(
// //       Network::challenge_proposal(
// //         RuntimeOrigin::signed(account(2)),
// //         subnet_id,
// //         proposal_index,
// //         Vec::new()
// //       ),
// //       Error::<Test>::NotDefendant
// //     );
// //   })
// // }

// // #[test]
// // fn test_challenge_proposal_complete() {
// // 	new_test_ext().execute_with(|| {
// //     let subnet_path: Vec<u8> = "petals-team/StableBeluga2".into();

// //     build_subnet(subnet_path.clone());

// //     let subnet_id = SubnetPaths::<Test>::get(subnet_path.clone()).unwrap();

// //     let mut n_peers: u32 = Network::max_subnet_nodes();
// //     if n_peers > MAX_SUBNET_NODES {
// //       n_peers = MAX_SUBNET_NODES
// //     }
// //     let deposit_amount: u128 = 10000000000000000000000;
// //     let amount: u128 = 1000000000000000000000;
// //     let amount_staked = build_subnet_nodes(subnet_id, 0, n_peers, deposit_amount, amount);

// //     let epoch_length = EpochLength::get();
// //     let epochs = SubnetNodeClassEpochs::<Test>::get(SubnetNodeClass::Accountant);
// //     System::set_block_number(System::block_number() + epochs * epoch_length + 1);
// //     Network::shift_node_classes(System::block_number(), epoch_length);

// //     assert_ok!(
// //       Network::propose(
// //         RuntimeOrigin::signed(account(0)),
// //         subnet_id,
// //         peer(1),
// //         Vec::new()
// //       ) 
// //     );

// //     let proposal_index = ProposalsCount::<Test>::get() - 1;

// //     assert_ok!(
// //       Network::cancel_proposal(
// //         RuntimeOrigin::signed(account(0)),
// //         subnet_id,
// //         proposal_index,
// //       )
// //     );

// //     // assert_err!(
// //     //   Network::challenge_proposal(
// //     //     RuntimeOrigin::signed(account(1)),
// //     //     subnet_id,
// //     //     proposal_index,
// //     //     Vec::new()
// //     //   ),
// //     //   Error::<Test>::ProposalComplete
// //     // );

// //     assert_err!(
// //       Network::challenge_proposal(
// //         RuntimeOrigin::signed(account(1)),
// //         subnet_id,
// //         proposal_index,
// //         Vec::new()
// //       ),
// //       Error::<Test>::ProposalInvalid
// //     );
// //   })
// // }

// // #[test]
// // fn test_challenge_proposal_challenge_period_passed() {
// // 	new_test_ext().execute_with(|| {
// //     let subnet_path: Vec<u8> = "petals-team/StableBeluga2".into();

// //     build_subnet(subnet_path.clone());

// //     let subnet_id = SubnetPaths::<Test>::get(subnet_path.clone()).unwrap();

// //     let mut n_peers: u32 = Network::max_subnet_nodes();
// //     if n_peers > MAX_SUBNET_NODES {
// //       n_peers = MAX_SUBNET_NODES
// //     }
// //     let deposit_amount: u128 = 10000000000000000000000;
// //     let amount: u128 = 1000000000000000000000;
// //     let amount_staked = build_subnet_nodes(subnet_id, 0, n_peers, deposit_amount, amount);

// //     let epoch_length = EpochLength::get();
// //     let epochs = SubnetNodeClassEpochs::<Test>::get(SubnetNodeClass::Accountant);
// //     System::set_block_number(System::block_number() + epochs * epoch_length + 1);
// //     Network::shift_node_classes(System::block_number(), epoch_length);

// //     assert_ok!(
// //       Network::propose(
// //         RuntimeOrigin::signed(account(0)),
// //         subnet_id,
// //         peer(1),
// //         Vec::new()
// //       ) 
// //     );

// //     let proposal_index = ProposalsCount::<Test>::get() - 1;

// //     let challenge_period = ChallengePeriod::<Test>::get();
// //     System::set_block_number(System::block_number() + challenge_period + 1);

// //     assert_err!(
// //       Network::challenge_proposal(
// //         RuntimeOrigin::signed(account(1)),
// //         subnet_id,
// //         proposal_index,
// //         Vec::new()
// //       ),
// //       Error::<Test>::ProposalChallengePeriodPassed
// //     );
// //   })
// // }

// // #[test]
// // fn test_challenge_proposal_already_challenged() {
// // 	new_test_ext().execute_with(|| {
// //     let subnet_path: Vec<u8> = "petals-team/StableBeluga2".into();

// //     build_subnet(subnet_path.clone());

// //     let subnet_id = SubnetPaths::<Test>::get(subnet_path.clone()).unwrap();

// //     let mut n_peers: u32 = Network::max_subnet_nodes();
// //     if n_peers > MAX_SUBNET_NODES {
// //       n_peers = MAX_SUBNET_NODES
// //     }
// //     let deposit_amount: u128 = 10000000000000000000000;
// //     let amount: u128 = 1000000000000000000000;
// //     let amount_staked = build_subnet_nodes(subnet_id, 0, n_peers, deposit_amount, amount);

// //     let epoch_length = EpochLength::get();
// //     let epochs = SubnetNodeClassEpochs::<Test>::get(SubnetNodeClass::Accountant);
// //     System::set_block_number(System::block_number() + epochs * epoch_length + 1);
// //     Network::shift_node_classes(System::block_number(), epoch_length);

// //     assert_ok!(
// //       Network::propose(
// //         RuntimeOrigin::signed(account(0)),
// //         subnet_id,
// //         peer(1),
// //         Vec::new()
// //       ) 
// //     );

// //     let proposal_index = ProposalsCount::<Test>::get() - 1;

// //     assert_ok!(
// //       Network::challenge_proposal(
// //         RuntimeOrigin::signed(account(1)),
// //         subnet_id,
// //         proposal_index,
// //         Vec::new()
// //       ) 
// //     );

// //     assert_err!(
// //       Network::challenge_proposal(
// //         RuntimeOrigin::signed(account(1)),
// //         subnet_id,
// //         proposal_index,
// //         Vec::new()
// //       ),
// //       Error::<Test>::ProposalChallenged
// //     );

// //   })
// // }

// // #[test]
// // fn test_challenge_proposal_not_enough_balance_to_bid() {
// // 	new_test_ext().execute_with(|| {
// //     let subnet_path: Vec<u8> = "petals-team/StableBeluga2".into();

// //     build_subnet(subnet_path.clone());

// //     let subnet_id = SubnetPaths::<Test>::get(subnet_path.clone()).unwrap();

// //     let mut n_peers: u32 = Network::max_subnet_nodes();
// //     if n_peers > MAX_SUBNET_NODES {
// //       n_peers = MAX_SUBNET_NODES
// //     }
// //     let deposit_amount: u128 = 10000000000000000000000;
// //     let amount: u128 = 1000000000000000000000;
// //     let amount_staked = build_subnet_nodes(subnet_id, 0, n_peers, deposit_amount, amount);

// //     let epoch_length = EpochLength::get();
// //     let epochs = SubnetNodeClassEpochs::<Test>::get(SubnetNodeClass::Accountant);
// //     System::set_block_number(System::block_number() + epochs * epoch_length + 1);
// //     Network::shift_node_classes(System::block_number(), epoch_length);

// //     assert_ok!(
// //       Network::propose(
// //         RuntimeOrigin::signed(account(0)),
// //         subnet_id,
// //         peer(1),
// //         Vec::new()
// //       ) 
// //     );

// //     let proposal_index = ProposalsCount::<Test>::get() - 1;
// //     let proposal_bid_amount = ProposalBidAmount::<Test>::get();
// //     let free_balance = Balances::free_balance(&account(1));

// //     assert_ok!(
// //       Balances::transfer_keep_alive(
// //         RuntimeOrigin::signed(account(1)),
// //         sp_runtime::MultiAddress::Id(account(2)),
// //         free_balance-500,
// //       )  
// //     );

// //     assert_err!(
// //       Network::challenge_proposal(
// //         RuntimeOrigin::signed(account(1)),
// //         subnet_id,
// //         proposal_index,
// //         Vec::new()
// //       ),
// //       Error::<Test>::NotEnoughBalanceToBid
// //     );

// //   })
// // }

// // #[test]
// // fn test_proposal_voting() {
// // 	new_test_ext().execute_with(|| {
// //     let subnet_path: Vec<u8> = "petals-team/StableBeluga2".into();

// //     build_subnet(subnet_path.clone());

// //     let subnet_id = SubnetPaths::<Test>::get(subnet_path.clone()).unwrap();

// //     let mut n_peers: u32 = Network::max_subnet_nodes();
// //     if n_peers > MAX_SUBNET_NODES {
// //       n_peers = MAX_SUBNET_NODES
// //     }
// //     let deposit_amount: u128 = 10000000000000000000000;
// //     let amount: u128 = 1000000000000000000000;
// //     let amount_staked = build_subnet_nodes(subnet_id, 0, n_peers, deposit_amount, amount);

// //     let epoch_length = EpochLength::get();
// //     let epochs = SubnetNodeClassEpochs::<Test>::get(SubnetNodeClass::Accountant);
// //     System::set_block_number(System::block_number() + epochs * epoch_length + 1);
// //     Network::shift_node_classes(System::block_number(), epoch_length);

// //     assert_ok!(
// //       Network::propose(
// //         RuntimeOrigin::signed(account(0)),
// //         subnet_id,
// //         peer(1),
// //         Vec::new()
// //       ) 
// //     );

// //     let proposal_index = ProposalsCount::<Test>::get() - 1;

// //     assert_ok!(
// //       Network::challenge_proposal(
// //         RuntimeOrigin::signed(account(1)),
// //         subnet_id,
// //         proposal_index,
// //         Vec::new()
// //       ) 
// //     );

// //     assert_ok!(
// //       Network::vote(
// //         RuntimeOrigin::signed(account(2)),
// //         subnet_id,
// //         proposal_index,
// //         VoteType::Yay
// //       ) 
// //     );

// //     let proposal = Proposals::<Test>::get(subnet_id, 0);
// //     assert_eq!(proposal.votes.yay.get(&account(2)), Some(&account(2)));
// //     assert_ne!(proposal.votes.yay.get(&account(2)), None);

// //   })
// // }

// // #[test]
// // fn test_proposal_voting_invalid_proposal_id() {
// // 	new_test_ext().execute_with(|| {
// //     let subnet_path: Vec<u8> = "petals-team/StableBeluga2".into();

// //     build_subnet(subnet_path.clone());

// //     let subnet_id = SubnetPaths::<Test>::get(subnet_path.clone()).unwrap();

// //     let mut n_peers: u32 = Network::max_subnet_nodes();
// //     if n_peers > MAX_SUBNET_NODES {
// //       n_peers = MAX_SUBNET_NODES
// //     }
// //     let deposit_amount: u128 = 10000000000000000000000;
// //     let amount: u128 = 1000000000000000000000;
// //     let amount_staked = build_subnet_nodes(subnet_id, 0, n_peers, deposit_amount, amount);

// //     let epoch_length = EpochLength::get();
// //     let epochs = SubnetNodeClassEpochs::<Test>::get(SubnetNodeClass::Accountant);
// //     System::set_block_number(System::block_number() + epochs * epoch_length + 1);
// //     Network::shift_node_classes(System::block_number(), epoch_length);

// //     assert_ok!(
// //       Network::propose(
// //         RuntimeOrigin::signed(account(0)),
// //         subnet_id,
// //         peer(1),
// //         Vec::new()
// //       ) 
// //     );

// //     let proposal_index = ProposalsCount::<Test>::get() - 1;

// //     assert_ok!(
// //       Network::challenge_proposal(
// //         RuntimeOrigin::signed(account(1)),
// //         subnet_id,
// //         proposal_index,
// //         Vec::new()
// //       ) 
// //     );

// //     assert_err!(
// //       Network::vote(
// //         RuntimeOrigin::signed(account(2)),
// //         subnet_id,
// //         1,
// //         VoteType::Yay
// //       ),
// //       Error::<Test>::ProposalInvalid
// //     );
// //   })
// // }

// // #[test]
// // fn test_proposal_voting_subnet_node_not_exist() {
// // 	new_test_ext().execute_with(|| {
// //     let subnet_path: Vec<u8> = "petals-team/StableBeluga2".into();

// //     build_subnet(subnet_path.clone());

// //     let subnet_id = SubnetPaths::<Test>::get(subnet_path.clone()).unwrap();

// //     let mut n_peers: u32 = Network::max_subnet_nodes();
// //     if n_peers > MAX_SUBNET_NODES {
// //       n_peers = MAX_SUBNET_NODES
// //     }
// //     let deposit_amount: u128 = 10000000000000000000000;
// //     let amount: u128 = 1000000000000000000000;
// //     let amount_staked = build_subnet_nodes(subnet_id, 0, n_peers, deposit_amount, amount);

// //     let epoch_length = EpochLength::get();
// //     let epochs = SubnetNodeClassEpochs::<Test>::get(SubnetNodeClass::Accountant);
// //     System::set_block_number(System::block_number() + epochs * epoch_length + 1);
// //     Network::shift_node_classes(System::block_number(), epoch_length);

// //     assert_ok!(
// //       Network::propose(
// //         RuntimeOrigin::signed(account(0)),
// //         subnet_id,
// //         peer(1),
// //         Vec::new()
// //       ) 
// //     );

// //     let proposal_index = ProposalsCount::<Test>::get() - 1;

// //     assert_ok!(
// //       Network::challenge_proposal(
// //         RuntimeOrigin::signed(account(1)),
// //         subnet_id,
// //         proposal_index,
// //         Vec::new()
// //       ) 
// //     );

// //     assert_err!(
// //       Network::vote(
// //         RuntimeOrigin::signed(account(n_peers+1)),
// //         subnet_id,
// //         proposal_index,
// //         VoteType::Yay
// //       ),
// //       Error::<Test>::SubnetNodeNotExist
// //     );
// //   })
// // }

// // #[test]
// // fn test_proposal_voting_proposal_unchallenged() {
// // 	new_test_ext().execute_with(|| {
// //     let subnet_path: Vec<u8> = "petals-team/StableBeluga2".into();

// //     build_subnet(subnet_path.clone());

// //     let subnet_id = SubnetPaths::<Test>::get(subnet_path.clone()).unwrap();

// //     let mut n_peers: u32 = Network::max_subnet_nodes();
// //     if n_peers > MAX_SUBNET_NODES {
// //       n_peers = MAX_SUBNET_NODES
// //     }
// //     let deposit_amount: u128 = 10000000000000000000000;
// //     let amount: u128 = 1000000000000000000000;
// //     let amount_staked = build_subnet_nodes(subnet_id, 0, n_peers, deposit_amount, amount);

// //     let epoch_length = EpochLength::get();
// //     let epochs = SubnetNodeClassEpochs::<Test>::get(SubnetNodeClass::Accountant);
// //     System::set_block_number(System::block_number() + epochs * epoch_length + 1);
// //     Network::shift_node_classes(System::block_number(), epoch_length);

// //     assert_ok!(
// //       Network::propose(
// //         RuntimeOrigin::signed(account(0)),
// //         subnet_id,
// //         peer(1),
// //         Vec::new()
// //       ) 
// //     );

// //     let proposal_index = ProposalsCount::<Test>::get() - 1;

// //     assert_err!(
// //       Network::vote(
// //         RuntimeOrigin::signed(account(2)),
// //         subnet_id,
// //         proposal_index,
// //         VoteType::Yay
// //       ),
// //       Error::<Test>::ProposalUnchallenged
// //     );
// //   })
// // }

// // // TODO: Need to finalize and then attempt to vote the proposal for failure
// // #[test]
// // fn test_proposal_voting_proposal_complete() {
// // 	new_test_ext().execute_with(|| {
// //     let subnet_path: Vec<u8> = "petals-team/StableBeluga2".into();

// //     build_subnet(subnet_path.clone());

// //     let subnet_id = SubnetPaths::<Test>::get(subnet_path.clone()).unwrap();

// //     let mut n_peers: u32 = Network::max_subnet_nodes();
// //     if n_peers > MAX_SUBNET_NODES {
// //       n_peers = MAX_SUBNET_NODES
// //     }
// //     let deposit_amount: u128 = 10000000000000000000000;
// //     let amount: u128 = 1000000000000000000000;
// //     let amount_staked = build_subnet_nodes(subnet_id, 0, n_peers, deposit_amount, amount);

// //     let epoch_length = EpochLength::get();
// //     let epochs = SubnetNodeClassEpochs::<Test>::get(SubnetNodeClass::Accountant);
// //     System::set_block_number(System::block_number() + epochs * epoch_length + 1);
// //     Network::shift_node_classes(System::block_number(), epoch_length);

// //     assert_ok!(
// //       Network::propose(
// //         RuntimeOrigin::signed(account(0)),
// //         subnet_id,
// //         peer(1),
// //         Vec::new()
// //       ) 
// //     );

// //     let proposal_index = ProposalsCount::<Test>::get() - 1;

// //     assert_err!(
// //       Network::vote(
// //         RuntimeOrigin::signed(account(2)),
// //         subnet_id,
// //         proposal_index,
// //         VoteType::Yay
// //       ),
// //       Error::<Test>::ProposalUnchallenged
// //     );
// //   })
// // }

// // #[test]
// // fn test_proposal_voting_invalid_voting_period() {
// // 	new_test_ext().execute_with(|| {
// //     let subnet_path: Vec<u8> = "petals-team/StableBeluga2".into();

// //     build_subnet(subnet_path.clone());

// //     let subnet_id = SubnetPaths::<Test>::get(subnet_path.clone()).unwrap();

// //     let mut n_peers: u32 = Network::max_subnet_nodes();
// //     if n_peers > MAX_SUBNET_NODES {
// //       n_peers = MAX_SUBNET_NODES
// //     }
// //     let deposit_amount: u128 = 10000000000000000000000;
// //     let amount: u128 = 1000000000000000000000;
// //     let amount_staked = build_subnet_nodes(subnet_id, 0, n_peers, deposit_amount, amount);

// //     let epoch_length = EpochLength::get();
// //     let epochs = SubnetNodeClassEpochs::<Test>::get(SubnetNodeClass::Accountant);
// //     System::set_block_number(System::block_number() + epochs * epoch_length + 1);
// //     Network::shift_node_classes(System::block_number(), epoch_length);

// //     assert_ok!(
// //       Network::propose(
// //         RuntimeOrigin::signed(account(0)),
// //         subnet_id,
// //         peer(1),
// //         Vec::new()
// //       ) 
// //     );


// //     let proposal_index = ProposalsCount::<Test>::get() - 1;

// //     assert_ok!(
// //       Network::challenge_proposal(
// //         RuntimeOrigin::signed(account(1)),
// //         subnet_id,
// //         proposal_index,
// //         Vec::new()
// //       ) 
// //     );

// //     let voting_period = VotingPeriod::<Test>::get();
// //     System::set_block_number(System::block_number() + voting_period + 1);

// //     assert_err!(
// //       Network::vote(
// //         RuntimeOrigin::signed(account(2)),
// //         subnet_id,
// //         proposal_index,
// //         VoteType::Yay
// //       ),
// //       Error::<Test>::VotingPeriodInvalid
// //     );
// //   })
// // }

// // #[test]
// // fn test_proposal_voting_not_eligible() {
// // 	new_test_ext().execute_with(|| {
// //     let subnet_path: Vec<u8> = "petals-team/StableBeluga2".into();

// //     build_subnet(subnet_path.clone());

// //     let subnet_id = SubnetPaths::<Test>::get(subnet_path.clone()).unwrap();

// //     let n_peers: u32 = 12;
// //     let deposit_amount: u128 = 10000000000000000000000;
// //     let amount: u128 = 1000000000000000000000;
// //     let amount_staked = build_subnet_nodes(subnet_id, 0, n_peers, deposit_amount, amount);

// //     let epoch_length = EpochLength::get();
// //     let epochs = SubnetNodeClassEpochs::<Test>::get(SubnetNodeClass::Accountant);
// //     System::set_block_number(System::block_number() + epochs * epoch_length + 1);
// //     Network::shift_node_classes(System::block_number(), epoch_length);

// //     assert_ok!(
// //       Network::propose(
// //         RuntimeOrigin::signed(account(0)),
// //         subnet_id,
// //         peer(1),
// //         Vec::new()
// //       ) 
// //     );

// //     let proposal_index = ProposalsCount::<Test>::get() - 1;

// //     assert_ok!(
// //       Network::challenge_proposal(
// //         RuntimeOrigin::signed(account(1)),
// //         subnet_id,
// //         proposal_index,
// //         Vec::new()
// //       ) 
// //     );

// //     let _ = Balances::deposit_creating(&account(n_peers+1), deposit_amount);
// //     assert_ok!(
// //       Network::add_subnet_node(
// //         RuntimeOrigin::signed(account(n_peers+1)),
// //         subnet_id,
// //         peer(n_peers+1),
// //         amount,
// //       ) 
// //     );

// //     assert_err!(
// //       Network::vote(
// //         RuntimeOrigin::signed(account(n_peers+1)),
// //         subnet_id,
// //         proposal_index,
// //         VoteType::Yay
// //       ),
// //       Error::<Test>::NotEligible
// //     );
// //   })
// // }

// // #[test]
// // fn test_proposal_voting_already_voted() {
// // 	new_test_ext().execute_with(|| {
// //     let subnet_path: Vec<u8> = "petals-team/StableBeluga2".into();

// //     build_subnet(subnet_path.clone());

// //     let subnet_id = SubnetPaths::<Test>::get(subnet_path.clone()).unwrap();

// //     let mut n_peers: u32 = Network::max_subnet_nodes();
// //     if n_peers > MAX_SUBNET_NODES {
// //       n_peers = MAX_SUBNET_NODES
// //     }
// //     let deposit_amount: u128 = 10000000000000000000000;
// //     let amount: u128 = 1000000000000000000000;
// //     let amount_staked = build_subnet_nodes(subnet_id, 0, n_peers, deposit_amount, amount);

// //     let epoch_length = EpochLength::get();
// //     let epochs = SubnetNodeClassEpochs::<Test>::get(SubnetNodeClass::Accountant);
// //     System::set_block_number(System::block_number() + epochs * epoch_length + 1);
// //     Network::shift_node_classes(System::block_number(), epoch_length);

// //     assert_ok!(
// //       Network::propose(
// //         RuntimeOrigin::signed(account(0)),
// //         subnet_id,
// //         peer(1),
// //         Vec::new()
// //       ) 
// //     );

// //     let proposal_index = ProposalsCount::<Test>::get() - 1;

// //     assert_ok!(
// //       Network::challenge_proposal(
// //         RuntimeOrigin::signed(account(1)),
// //         subnet_id,
// //         proposal_index,
// //         Vec::new()
// //       ) 
// //     );

// //     assert_ok!(
// //       Network::vote(
// //         RuntimeOrigin::signed(account(2)),
// //         subnet_id,
// //         proposal_index,
// //         VoteType::Yay
// //       ) 
// //     );

// //     assert_ok!(
// //       Network::vote(
// //         RuntimeOrigin::signed(account(3)),
// //         subnet_id,
// //         proposal_index,
// //         VoteType::Yay
// //       ) 
// //     );

// //     assert_err!(
// //       Network::vote(
// //         RuntimeOrigin::signed(account(3)),
// //         subnet_id,
// //         proposal_index,
// //         VoteType::Yay
// //       ),
// //       Error::<Test>::AlreadyVoted
// //     );

// //   })
// // }

// // #[test]
// // fn test_proposal_finalize_proposal_plaintiff_winner() {
// // 	new_test_ext().execute_with(|| {
// //     let subnet_path: Vec<u8> = "petals-team/StableBeluga2".into();

// //     build_subnet(subnet_path.clone());

// //     let subnet_id = SubnetPaths::<Test>::get(subnet_path.clone()).unwrap();

// //     let mut n_peers: u32 = Network::max_subnet_nodes();
// //     if n_peers > MAX_SUBNET_NODES {
// //       n_peers = MAX_SUBNET_NODES
// //     }
// //     let deposit_amount: u128 = 10000000000000000000000;
// //     let amount: u128 = 1000000000000000000000;
// //     let amount_staked = build_subnet_nodes(subnet_id, 0, n_peers, deposit_amount, amount);

// //     let epoch_length = EpochLength::get();
// //     let epochs = SubnetNodeClassEpochs::<Test>::get(SubnetNodeClass::Accountant);
// //     System::set_block_number(System::block_number() + epochs * epoch_length + 1);
// //     Network::shift_node_classes(System::block_number(), epoch_length);

// //     let proposal_bid_amount = ProposalBidAmount::<Test>::get();

// //     let plaintiff_starting_balance = Balances::free_balance(&account(0));
// //     let defendant_starting_balance = Balances::free_balance(&account(1));

// //     assert_ok!(
// //       Network::propose(
// //         RuntimeOrigin::signed(account(0)),
// //         subnet_id,
// //         peer(1),
// //         Vec::new()
// //       ) 
// //     );

// //     let plaintiff_after_balance = Balances::free_balance(&account(0));
// //     assert_eq!(plaintiff_starting_balance - proposal_bid_amount, plaintiff_after_balance);

// //     let proposal_index = ProposalsCount::<Test>::get() - 1;

// //     assert_ok!(
// //       Network::challenge_proposal(
// //         RuntimeOrigin::signed(account(1)),
// //         subnet_id,
// //         proposal_index,
// //         Vec::new()
// //       ) 
// //     );

// //     for n in 0..n_peers {
// //       if n == 0 || n == 1 {
// //         continue
// //       }
// //       assert_ok!(
// //         Network::vote(
// //           RuntimeOrigin::signed(account(n)),
// //           subnet_id,
// //           proposal_index,
// //           VoteType::Yay
// //         ) 
// //       );  
// //     }

// //     let plaintiff_after_balance = Balances::free_balance(&account(0));
// //     assert_eq!(plaintiff_starting_balance - proposal_bid_amount, plaintiff_after_balance);

// //     let voting_period = VotingPeriod::<Test>::get();
// //     System::set_block_number(System::block_number() + voting_period + 1);

// //     let voter_starting_balance = Balances::free_balance(&account(3));

// //     assert_ok!(
// //       Network::finalize_proposal(
// //         RuntimeOrigin::signed(account(0)),
// //         subnet_id,
// //         proposal_index,
// //       ) 
// //     );

// //     let mut proposal = Proposals::<Test>::get(subnet_id, proposal_index);
// //     let winner_voters_len = proposal.votes.yay.len();
// //     assert_eq!(winner_voters_len, (n_peers - 2) as usize);

// //     let mut distributees = proposal.votes.yay;
// //     // Insert winner to the distributees
// //     distributees.insert(account(0));

// //     let distribution_amount = proposal_bid_amount.saturating_div(distributees.len() as u128);

// //     for n in 0..n_peers {
// //       if n == 0 || n == 1 {
// //         continue
// //       }
// //       let voter_balance = Balances::free_balance(&account(n));
// //       assert_eq!(voter_balance, voter_starting_balance + distribution_amount);
// //     }

// //     let distribution_dust = proposal_bid_amount - (distribution_amount * (distributees.len() as u128));

// //     // --- Plaintiff after finalization should be bond amount + distribution + dust
// //     let plaintiff_after_balance = Balances::free_balance(&account(0));

// //     assert_eq!(plaintiff_after_balance, plaintiff_starting_balance + distribution_amount + distribution_dust);

// //     // --- Defendant after finalization should be same since they lost
// //     let defendant_after_balance = Balances::free_balance(&account(1));
// //     assert_eq!(defendant_starting_balance - proposal_bid_amount, defendant_after_balance);
// //   })
// // }

// // #[test]
// // fn test_proposal_finalize_proposal_defendant_winner() {
// // 	new_test_ext().execute_with(|| {
// //     let subnet_path: Vec<u8> = "petals-team/StableBeluga2".into();

// //     build_subnet(subnet_path.clone());

// //     let subnet_id = SubnetPaths::<Test>::get(subnet_path.clone()).unwrap();

// //     let mut n_peers: u32 = Network::max_subnet_nodes();
// //     if n_peers > MAX_SUBNET_NODES {
// //       n_peers = MAX_SUBNET_NODES
// //     }
// //     let deposit_amount: u128 = 10000000000000000000000;
// //     let amount: u128 = 1000000000000000000000;
// //     let amount_staked = build_subnet_nodes(subnet_id, 0, n_peers, deposit_amount, amount);

// //     let epoch_length = EpochLength::get();
// //     let epochs = SubnetNodeClassEpochs::<Test>::get(SubnetNodeClass::Accountant);
// //     System::set_block_number(System::block_number() + epochs * epoch_length + 1);
// //     Network::shift_node_classes(System::block_number(), epoch_length);

// //     let proposal_bid_amount = ProposalBidAmount::<Test>::get();

// //     let plaintiff_starting_balance = Balances::free_balance(&account(0));
// //     let defendant_starting_balance = Balances::free_balance(&account(1));

// //     assert_ok!(
// //       Network::propose(
// //         RuntimeOrigin::signed(account(0)),
// //         subnet_id,
// //         peer(1),
// //         Vec::new()
// //       ) 
// //     );

// //     let proposal_index = ProposalsCount::<Test>::get() - 1;

// //     assert_ok!(
// //       Network::challenge_proposal(
// //         RuntimeOrigin::signed(account(1)),
// //         subnet_id,
// //         proposal_index,
// //         Vec::new()
// //       ) 
// //     );

// //     for n in 0..n_peers {
// //       if n == 0 || n == 1 {
// //         continue
// //       }
// //       assert_ok!(
// //         Network::vote(
// //           RuntimeOrigin::signed(account(n)),
// //           subnet_id,
// //           proposal_index,
// //           VoteType::Nay
// //         ) 
// //       );  
// //     }

// //     let plaintiff_after_balance = Balances::free_balance(&account(0));
// //     assert_eq!(plaintiff_starting_balance - proposal_bid_amount, plaintiff_after_balance);

// //     let voting_period = VotingPeriod::<Test>::get();
// //     System::set_block_number(System::block_number() + voting_period + 1);

// //     let voter_starting_balance = Balances::free_balance(&account(3));

// //     assert_ok!(
// //       Network::finalize_proposal(
// //         RuntimeOrigin::signed(account(0)),
// //         subnet_id,
// //         proposal_index,
// //       ) 
// //     );

// //     let mut proposal = Proposals::<Test>::get(subnet_id, proposal_index);
// //     let winner_voters_len = proposal.votes.nay.len();
// //     assert_eq!(winner_voters_len, (n_peers - 2) as usize);

// //     let mut distributees = proposal.votes.nay;
// //     // Insert winner to the distributees
// //     distributees.insert(account(0));

// //     let distribution_amount = proposal_bid_amount.saturating_div(distributees.len() as u128);

// //     for n in 0..n_peers {
// //       if n == 0 || n == 1 {
// //         continue
// //       }
// //       let voter_balance = Balances::free_balance(&account(n));
// //       assert_eq!(voter_balance, voter_starting_balance + distribution_amount);
// //     }

// //     let distribution_dust = proposal_bid_amount - (distribution_amount * (distributees.len() as u128));

// //     // --- Plaintiff after finalization should be bond amount + distribution + dust
// //     let defendant_after_balance = Balances::free_balance(&account(1));
    
// //     assert_eq!(defendant_after_balance, defendant_starting_balance + distribution_amount + distribution_dust);

// //     // --- Defendant after finalization should be same since they lost
// //     let plaintiff_after_balance = Balances::free_balance(&account(0));
// //     assert_eq!(plaintiff_starting_balance - proposal_bid_amount, plaintiff_after_balance);
// //   })
// // }

// // #[test]
// // fn test_proposal_finalize_proposal_unchallenged() {
// // 	new_test_ext().execute_with(|| {
// //     let subnet_path: Vec<u8> = "petals-team/StableBeluga2".into();

// //     build_subnet(subnet_path.clone());

// //     let subnet_id = SubnetPaths::<Test>::get(subnet_path.clone()).unwrap();

// //     let mut n_peers: u32 = Network::max_subnet_nodes();
// //     if n_peers > MAX_SUBNET_NODES {
// //       n_peers = MAX_SUBNET_NODES
// //     }
// //     let deposit_amount: u128 = 10000000000000000000000;
// //     let amount: u128 = 1000000000000000000000;
// //     let amount_staked = build_subnet_nodes(subnet_id, 0, n_peers, deposit_amount, amount);

// //     let epoch_length = EpochLength::get();
// //     let epochs = SubnetNodeClassEpochs::<Test>::get(SubnetNodeClass::Accountant);
// //     System::set_block_number(System::block_number() + epochs * epoch_length + 1);
// //     Network::shift_node_classes(System::block_number(), epoch_length);

// //     let proposal_bid_amount = ProposalBidAmount::<Test>::get();

// //     let plaintiff_starting_balance = Balances::free_balance(&account(0));
// //     let defendant_starting_balance = Balances::free_balance(&account(1));

// //     assert_ok!(
// //       Network::propose(
// //         RuntimeOrigin::signed(account(0)),
// //         subnet_id,
// //         peer(1),
// //         Vec::new()
// //       ) 
// //     );

// //     let proposal_index = ProposalsCount::<Test>::get() - 1;

// //     assert_err!(
// //       Network::finalize_proposal(
// //         RuntimeOrigin::signed(account(0)),
// //         subnet_id,
// //         proposal_index,
// //       ),
// //       Error::<Test>::ProposalUnchallenged
// //     );

// //   })
// // }

// // #[test]
// // fn test_proposal_finalize_proposal_complete() {
// // 	new_test_ext().execute_with(|| {
// //     let subnet_path: Vec<u8> = "petals-team/StableBeluga2".into();

// //     build_subnet(subnet_path.clone());

// //     let subnet_id = SubnetPaths::<Test>::get(subnet_path.clone()).unwrap();

// //     let mut n_peers: u32 = Network::max_subnet_nodes();
// //     if n_peers > MAX_SUBNET_NODES {
// //       n_peers = MAX_SUBNET_NODES
// //     }
// //     let deposit_amount: u128 = 10000000000000000000000;
// //     let amount: u128 = 1000000000000000000000;
// //     let amount_staked = build_subnet_nodes(subnet_id, 0, n_peers, deposit_amount, amount);

// //     let epoch_length = EpochLength::get();
// //     let epochs = SubnetNodeClassEpochs::<Test>::get(SubnetNodeClass::Accountant);
// //     System::set_block_number(System::block_number() + epochs * epoch_length + 1);
// //     Network::shift_node_classes(System::block_number(), epoch_length);

// //     let proposal_bid_amount = ProposalBidAmount::<Test>::get();

// //     let plaintiff_starting_balance = Balances::free_balance(&account(0));
// //     let defendant_starting_balance = Balances::free_balance(&account(1));

// //     assert_ok!(
// //       Network::propose(
// //         RuntimeOrigin::signed(account(0)),
// //         subnet_id,
// //         peer(1),
// //         Vec::new()
// //       ) 
// //     );

// //     let plaintiff_after_balance = Balances::free_balance(&account(0));
// //     assert_eq!(plaintiff_starting_balance - proposal_bid_amount, plaintiff_after_balance);

// //     let proposal_index = ProposalsCount::<Test>::get() - 1;

// //     assert_ok!(
// //       Network::challenge_proposal(
// //         RuntimeOrigin::signed(account(1)),
// //         subnet_id,
// //         proposal_index,
// //         Vec::new()
// //       ) 
// //     );

// //     for n in 0..n_peers {
// //       if n == 0 || n == 1 {
// //         continue
// //       }
// //       assert_ok!(
// //         Network::vote(
// //           RuntimeOrigin::signed(account(n)),
// //           subnet_id,
// //           proposal_index,
// //           VoteType::Yay
// //         ) 
// //       );  
// //     }

// //     let plaintiff_after_balance = Balances::free_balance(&account(0));
// //     assert_eq!(plaintiff_starting_balance - proposal_bid_amount, plaintiff_after_balance);

// //     let voting_period = VotingPeriod::<Test>::get();
// //     System::set_block_number(System::block_number() + voting_period + 1);

// //     let voter_starting_balance = Balances::free_balance(&account(3));

// //     assert_ok!(
// //       Network::finalize_proposal(
// //         RuntimeOrigin::signed(account(0)),
// //         subnet_id,
// //         proposal_index,
// //       ) 
// //     );

// //     let mut proposal = Proposals::<Test>::get(subnet_id, proposal_index);
// //     let winner_voters_len = proposal.votes.yay.len();
// //     assert_eq!(winner_voters_len, (n_peers - 2) as usize);

// //     let mut distributees = proposal.votes.yay;
// //     // Insert winner to the distributees
// //     distributees.insert(account(0));

// //     let distribution_amount = proposal_bid_amount.saturating_div(distributees.len() as u128);

// //     for n in 0..n_peers {
// //       if n == 0 || n == 1 {
// //         continue
// //       }
// //       let voter_balance = Balances::free_balance(&account(n));
// //       assert_eq!(voter_balance, voter_starting_balance + distribution_amount);
// //     }

// //     let distribution_dust = proposal_bid_amount - (distribution_amount * (distributees.len() as u128));

// //     // --- Plaintiff after finalization should be bond amount + distribution + dust
// //     let plaintiff_after_balance = Balances::free_balance(&account(0));

// //     assert_eq!(plaintiff_after_balance, plaintiff_starting_balance + distribution_amount + distribution_dust);

// //     // --- Defendant after finalization should be same since they lost
// //     let defendant_after_balance = Balances::free_balance(&account(1));
// //     assert_eq!(defendant_starting_balance - proposal_bid_amount, defendant_after_balance);

// //     assert_err!(
// //       Network::finalize_proposal(
// //         RuntimeOrigin::signed(account(0)),
// //         subnet_id,
// //         proposal_index,
// //       ),
// //       Error::<Test>::ProposalComplete
// //     );
// //   })
// // }

// // #[test]
// // fn test_proposal_finalize_proposal_voting_period_invalid() {
// // 	new_test_ext().execute_with(|| {
// //     let subnet_path: Vec<u8> = "petals-team/StableBeluga2".into();

// //     build_subnet(subnet_path.clone());

// //     let subnet_id = SubnetPaths::<Test>::get(subnet_path.clone()).unwrap();

// //     let mut n_peers: u32 = Network::max_subnet_nodes();
// //     if n_peers > MAX_SUBNET_NODES {
// //       n_peers = MAX_SUBNET_NODES
// //     }
// //     let deposit_amount: u128 = 10000000000000000000000;
// //     let amount: u128 = 1000000000000000000000;
// //     let amount_staked = build_subnet_nodes(subnet_id, 0, n_peers, deposit_amount, amount);

// //     let epoch_length = EpochLength::get();
// //     let epochs = SubnetNodeClassEpochs::<Test>::get(SubnetNodeClass::Accountant);
// //     System::set_block_number(System::block_number() + epochs * epoch_length + 1);
// //     Network::shift_node_classes(System::block_number(), epoch_length);

// //     let proposal_bid_amount = ProposalBidAmount::<Test>::get();

// //     let plaintiff_starting_balance = Balances::free_balance(&account(0));
// //     let defendant_starting_balance = Balances::free_balance(&account(1));

// //     assert_ok!(
// //       Network::propose(
// //         RuntimeOrigin::signed(account(0)),
// //         subnet_id,
// //         peer(1),
// //         Vec::new()
// //       ) 
// //     );

// //     let plaintiff_after_balance = Balances::free_balance(&account(0));
// //     assert_eq!(plaintiff_starting_balance - proposal_bid_amount, plaintiff_after_balance);

// //     let proposal_index = ProposalsCount::<Test>::get() - 1;

// //     assert_ok!(
// //       Network::challenge_proposal(
// //         RuntimeOrigin::signed(account(1)),
// //         subnet_id,
// //         proposal_index,
// //         Vec::new()
// //       ) 
// //     );

// //     for n in 0..n_peers {
// //       if n == 0 || n == 1 {
// //         continue
// //       }
// //       assert_ok!(
// //         Network::vote(
// //           RuntimeOrigin::signed(account(n)),
// //           subnet_id,
// //           proposal_index,
// //           VoteType::Yay
// //         ) 
// //       );  
// //     }

// //     assert_err!(
// //       Network::finalize_proposal(
// //         RuntimeOrigin::signed(account(0)),
// //         subnet_id,
// //         proposal_index,
// //       ),
// //       Error::<Test>::VotingPeriodInvalid
// //     );
// //   })
// // }
