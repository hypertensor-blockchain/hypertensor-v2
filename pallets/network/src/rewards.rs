// // Copyright (C) Hypertensor.
// // SPDX-License-Identifier: Apache-2.0

// // Licensed under the Apache License, Version 2.0 (the "License");
// // you may not use this file except in compliance with the License.
// // You may obtain a copy of the License at
// //
// // 	http://www.apache.org/licenses/LICENSE-2.0
// //
// // Unless required by applicable law or agreed to in writing, software
// // distributed under the License is distributed on an "AS IS" BASIS,
// // WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// // See the License for the specific language governing permissions and
// // limitations under the License.

// use super::*;
// use sp_runtime::Saturating;
// use frame_support::pallet_prelude::DispatchResultWithPostInfo;
// use frame_support::pallet_prelude::Pays;

// impl<T: Config> Pallet<T> {
//   pub fn reward_subnets(block: u64, epoch: u32) -> DispatchResultWithPostInfo {
//     // --- Get base rewards based on subnet memory requirements
//     let base_reward_per_mb: u128 = BaseRewardPerMB::<T>::get();
//     // --- Get required attestation percentage
//     let min_attestation_percentage = MinAttestationPercentage::<T>::get();
//     let min_vast_majority_attestation_percentage = MinVastMajorityAttestationPercentage::<T>::get();
//     // --- Get max epochs in a row a subnet node can be absent from consensus data
//     let max_subnet_node_penalties = MaxSubnetNodePenalties::<T>::get();
//     // --- Get the max penalties a subnet node can have before being removed from the network
//     let max_subnet_penalty_count = MaxSubnetPenaltyCount::<T>::get();
//     // --- Get the attestation percentage for a subnet node to be removed from the network
//     //     if they are not included in the validators consensus data
//     // --- If this attestation threshold is exceeded, the subnet node that is absent will have its
//     //     SubnetNodePenalties incrememented
//     let node_attestation_removal_threshold = NodeAttestationRemovalThreshold::<T>::get();
//     // --- Get the percentage of the subnet rewards that go to subnet delegate stakers
//     let delegate_stake_rewards_percentage: u128 = DelegateStakeRewardsPercentage::<T>::get();

//     let max_subnet_node_registration_epochs = MaxSubnetNodeRegistrationEpochs::<T>::get();
//     let subnet_owner_percentage = SubnetOwnerPercentage::<T>::get();

//     for (subnet_id, data) in SubnetsData::<T>::iter() {
//       let mut attestation_percentage: u128 = 0;

//       // --- We don't check for minimum nodes because nodes cannot validate or attest if they are not met
//       //     as they the validator will not be chosen in ``do_epoch_preliminaries`` if the 
//       //     min nodes are not met on that epoch.
//       if let Ok(mut submission) = SubnetRewardsSubmission::<T>::try_get(subnet_id, epoch) {
//         // --- Get memory of the subnet
//         let memory_mb = data.memory_mb;

//         // --- Get overall subnet rewards
//         let overall_subnet_reward: u128 = Self::percent_mul(base_reward_per_mb, memory_mb);

//         // --- Get delegators rewards
//         let delegate_stake_reward: u128 = Self::percent_mul(overall_subnet_reward, delegate_stake_rewards_percentage);

//         // --- Get subnet nodes rewards
//         let subnet_node_reward: u128 = overall_subnet_reward.saturating_sub(delegate_stake_reward);

//         // --- Redundant
//         if subnet_node_reward == 0 {
//           continue
//         }

//         let min_nodes = data.min_nodes;

//         // --- Get subnet nodes count to check against attestation count
//         // ``reward_subnuts`` is called before ``shift_node_classes`` so we can know how many nodes are validators
//         // while in this function that should have in the epoch the rewards are destined for
//         let subnet_nodes: Vec<T::AccountId> = Self::get_classified_hotkeys(subnet_id, &SubnetNodeClass::Validator, epoch as u64);
//         let subnet_node_count = subnet_nodes.len() as u128;

//         // --- Ensure nodes are at min requirement to continue rewards operations
//         if subnet_node_count < min_nodes as u128 {
//           continue
//         }

//         let attestations: u128 = submission.attests.len() as u128;
//         attestation_percentage = Self::percent_div(attestations, subnet_node_count);

//         // Redundant
//         // When subnet nodes exit, the consensus data is updated to remove them from it
//         if attestation_percentage > Self::PERCENTAGE_FACTOR {
//           attestation_percentage = Self::PERCENTAGE_FACTOR;
//         }
        
//         let validator_subnet_node_id: u32 = submission.validator_id;

//         let data_len = submission.data.len();

//         // --- If validator submitted no data, or less than the minimum required subnet nodes 
//         //     we assume the subnet is broken
//         // There is no slashing if subnet is broken, only risk of subnet being removed
//         // The subnet is deemed broken is there is no consensus or not enough nodes
//         //
//         // The subnet has up to the MaxSubnetPenaltyCount to solve the issue before the subnet and all subnet nodes are removed
//         if (data_len as u32) < min_nodes  {
//           // --- Increase the penalty count for the subnet because its deemed in a broken state
//           SubnetPenaltyCount::<T>::mutate(subnet_id, |n: &mut u32| *n += 1);

//           // If the subnet is broken, the validator can avoid slashing by submitting consensus with null data

//           // --- If subnet nodes aren't in consensus this is true
//           // Since we can assume the subnet is in a broken state, we don't slash the validator
//           // even if others do not attest to this state???

//           // --- If the subnet nodes are not in agreement with the validator that the model is broken, we
//           //     increase the penalty score for the validator
//           //     and slash
//           // --- We only do this if the vast majority of nodes are not in agreement with the validator
//           //     Otherwise we assume the issue just started or is being resolved.
//           //     i.e. if a validator sends in no data but by the time some other nodes check, it's resolved,
//           //          the validator only gets slashed if the vast majority of nodes disagree at ~87.5%
//           //     Or vice versa if validator submits data in a healthy state and the subnet breaks
//           //
//           // This is an unlikely scenario because all nodes should be checking the subnets state within a few seconds
//           // of each other.
//           if attestation_percentage < min_vast_majority_attestation_percentage {
//             // --- Slash validator and increase penalty score
//             Self::slash_validator(subnet_id, validator_subnet_node_id, attestation_percentage, block);
//           }

//           // --- If the subnet was deemed in a broken stake by the validator, rewards are bypassed
//           continue;
//         }

//         // --- If the minimum required attestation not reached, assume validator is dishonest, slash, and continue
//         // We don't increase subnet penalty count here because this is likely the validators fault
//         if attestation_percentage < min_attestation_percentage {
//           // --- Slash validator and increase penalty score
//           Self::slash_validator(subnet_id, validator_subnet_node_id, attestation_percentage, block);
          
//           // --- Attestation not successful, move on to next subnet
//           continue
//         }

//         // --- Get sum of subnet total scores for use of divvying rewards
//         let sum = submission.data.iter().fold(0, |acc, x| acc + x.score);
    
//         // --- Reward validators
//         for (subnet_node_id, subnet_node) in SubnetNodesData::<T>::iter_prefix(subnet_id) {
//           let hotkey: T::AccountId = match SubnetNodeIdHotkey::<T>::try_get(subnet_id, subnet_node_id) {
//             Ok(hotkey) => hotkey,
//             Err(()) => continue,
//           };
      
//           // --- (if) Check if subnet node is past the max registration epochs to activate (if registered or deactivated)
//           // --- (else if) Check if past Queue and can be included in validation data
//           // Always continue if any of these are true
//           // Note: Only ``included`` or above nodes can get emissions
//           if subnet_node.classification.class <= SubnetNodeClass::Registered {
//             if epoch as u64 > subnet_node.classification.start_epoch.saturating_add(max_subnet_node_registration_epochs) {
//               Self::perform_remove_subnet_node(block, subnet_id, subnet_node_id);
//             }
//             continue
//           } else if subnet_node.classification.class == SubnetNodeClass::Queue {
//             // If not, upgrade classification and continue
//             // --- Upgrade to included
//             SubnetNodesData::<T>::mutate(
//               subnet_id,
//               subnet_node_id,
//               |params: &mut SubnetNode<T::AccountId>| {
//                 params.classification = SubnetNodeClassification {
//                   class: SubnetNodeClass::Included,
//                   start_epoch: (epoch) as u64,
//                 };
//               },
//             );
//             continue
//           }

//           // --- At this point, all nodes should be included in consensus data

//           let peer_id: PeerId = subnet_node.peer_id;

//           let mut subnet_node_data: SubnetNodeData = SubnetNodeData::default();

//           // --- Confirm if ``peer_id`` is present in validator data
//           let subnet_node_data_find: Option<(usize, &SubnetNodeData)> = submission.data.iter().enumerate().find(
//             |&x| x.1.peer_id == peer_id
//           );
          
//           // --- If subnet_node_id is present in validator data
//           let validated: bool = subnet_node_data_find.is_some();

//           if validated {
//             subnet_node_data = subnet_node_data_find.unwrap().1.clone();
//             submission.data.remove(subnet_node_data_find.unwrap().0);
//           }

//           let penalties = SubnetNodePenalties::<T>::get(subnet_id, subnet_node_id);

//           // --- If node not validated and consensus reached:
//           //      otherwise, increment penalty score only
//           //      remove them if max penalties threshold is reached
//           if !validated {
//             // --- Mutate nodes penalties count if not in consensus
//             SubnetNodePenalties::<T>::insert(subnet_id, subnet_node_id, penalties + 1);

//             // --- To be removed or increase absent count, the consensus threshold must be reached
//             if attestation_percentage > node_attestation_removal_threshold {
//               // We don't slash nodes for not being in consensus
//               // A node can be removed for any reason and may not be due to dishonesty
//               // If subnet validators want to remove and slash a node, they can use the proposals mechanism

//               // --- Ensure maximum sequential removal consensus threshold is reached
//               // We make sure the super majority are in agreeance to remove someone
//               // TODO: Check the size of subnet and scale it from there
//               if penalties + 1 > max_subnet_node_penalties {
//                 // --- Increase account penalty count
//                 Self::perform_remove_subnet_node(block, subnet_id, subnet_node_id);
//               }
//             }
//             // Even if there is a n-1 100% consensus on the node being out of consensus, we don't remove them.
//             // In the case where a subnet wants to remove a node, they should initiate a proposal to have them removed
//             // using ``propose``method
//             continue
//           }

//           // --- At this point, a subnet node is in the consensus data

//           // --- Check if can be included in validation data
//           // By this point, node is validated, update to submittable if they have no penalties
//           let is_included = subnet_node.classification.class == SubnetNodeClass::Included;
//           if is_included && penalties == 0 {
//             // --- Upgrade to Validator
//             SubnetNodesData::<T>::mutate(
//               subnet_id,
//               subnet_node_id,
//               |params: &mut SubnetNode<T::AccountId>| {
//                 params.classification = SubnetNodeClassification {
//                   class: SubnetNodeClass::Validator,
//                   start_epoch: (epoch) as u64, // in case rewards are called late, we add them to the next epoch, 2 from the consensus data
//                 };
//               },
//             );
//             continue
//           } else if is_included && penalties != 0 {
//             // --- Decrease subnet node penalty count by one if in consensus and attested consensus
//             SubnetNodePenalties::<T>::mutate(subnet_id, subnet_node_id, |n: &mut u32| n.saturating_dec());
//             continue
//           }

//           // --- At this point, the subnet node is submittable and included in consensus data

//           //
//           // TODO: Test removing this ``!submission.attests.contains(&hotkey)`` to allow those that do not attest to gain rewards
//           //

//           // --- If not attested, do not receive rewards
//           // We only penalize accounts on vast majority attestations for not attesting data in case data is corrupted
//           // It is up to subnet nodes to remove them via consensus
//           // But since consensus was formed at the least, we assume they're against the consensus, therefor likely dishonest
//           if !submission.attests.contains_key(&subnet_node_id) {
//             if attestation_percentage > min_vast_majority_attestation_percentage {
//               // --- Penalize on vast majority only
//               SubnetNodePenalties::<T>::insert(subnet_id, subnet_node_id, penalties + 1);
//             }  
//             continue
//           }

//           let score = subnet_node_data.score;

//           // The subnet node has passed the gauntlet and is about to receive rewards
          
//           // --- Decrease subnet node penalty count by one if in consensus and attested consensus
//           // Don't hit the storage unless we have to
//           if penalties != 0 {
//             SubnetNodePenalties::<T>::mutate(subnet_id, subnet_node_id, |n: &mut u32| n.saturating_dec());
//           }

//           // --- Calculate score percentage of peer versus sum
//           let score_percentage: u128 = Self::percent_div(subnet_node_data.score, sum as u128);
//           // --- Calculate score percentage of total subnet generated epoch rewards
//           let mut account_reward: u128 = Self::percent_mul(score_percentage, subnet_node_reward);

//           // --- Increase reward if validator
//           if subnet_node_id == validator_subnet_node_id {
//             account_reward += Self::get_validator_reward(attestation_percentage);    
//           }

//           // --- Skip if no rewards to give
//           // Unlikely to happen
//           if account_reward == 0 {
//             continue
//           }

//           let mut node_delegate_reward = 0;
//           if subnet_node.delegate_reward_rate != 0 {
//             let total_node_delegated_stake_shares = TotalNodeDelegateStakeShares::<T>::get(subnet_id, subnet_node_id);
//             if total_node_delegated_stake_shares != 0 {
//               node_delegate_reward = Self::percent_mul(account_reward, subnet_node.delegate_reward_rate);
//               account_reward = account_reward - node_delegate_reward;
//               Self::do_increase_node_delegate_stake(
//                 subnet_id,
//                 subnet_node_id,
//                 node_delegate_reward,
//               );  
//             }
//           }

//           // --- Increase account stake and emit event
//           Self::increase_account_stake(
//             &hotkey,
//             subnet_id, 
//             account_reward,
//           ); 
//         }

//         // --- Portion of rewards to delegate stakers
//         Self::do_increase_delegate_stake(
//           subnet_id,
//           delegate_stake_reward,
//         );

//         // --- Increment down subnet penalty score on successful epochs
//         SubnetPenaltyCount::<T>::mutate(subnet_id, |n: &mut u32| n.saturating_dec());
//       } else if let Ok(validator_id) = SubnetRewardsValidator::<T>::try_get(subnet_id, epoch) {
//         // --- If a validator has been chosen that means they are supposed to be submitting consensus data
//         // --- If there is no submission but validator chosen, increase penalty on subnet and validator
//         // --- Increase the penalty count for the subnet
//         // The next validator on the next epoch can increment the penalty score down
//         SubnetPenaltyCount::<T>::mutate(subnet_id, |n: &mut u32| *n += 1);

//         // NOTE:
//         //  Each subnet increases the penalty score if they don't have the minimum subnet nodes required by the time
//         //  the subnet is enabled for emissions. This happens by the blockchain validator before choosing the subnet validator

//         // If validator didn't submit anything, then slash
//         // Even if a subnet is in a broken state, the chosen validator must submit blank data
//         Self::slash_validator(subnet_id, validator_id, 0, block);
//       }

//       // TODO: Automatically remove subnet if greater than max penalties count
//       // TODO: Get benchmark for removing max subnets in one epoch to ensure does not surpass max weights

//       Self::deposit_event(
//         Event::RewardResult { 
//           subnet_id: subnet_id, 
//           attestation_percentage: attestation_percentage, 
//         }
//       );
  
//       // --- If subnet is past its max penalty count, remove
//       let subnet_penalty_count = SubnetPenaltyCount::<T>::get(subnet_id);
//       if subnet_penalty_count > max_subnet_penalty_count {
//         Self::do_remove_subnet(
//           data.path,
//           SubnetRemovalReason::MaxPenalties,
//         );
//       }
//     }

//     Ok(None.into())
//   }
// }