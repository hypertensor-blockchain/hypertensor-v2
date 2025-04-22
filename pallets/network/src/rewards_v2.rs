// Copyright (C) Hypertensor.
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use super::*;
use sp_runtime::Saturating;
use frame_support::pallet_prelude::DispatchResultWithPostInfo;
use frame_support::pallet_prelude::Pays;
use libm::sqrt;

impl<T: Config> Pallet<T> {
  pub fn reward_subnets(block: u32, epoch: u32) -> DispatchResultWithPostInfo {
    // --- Get required attestation percentage
    let min_attestation_percentage = MinAttestationPercentage::<T>::get();
    let min_vast_majority_attestation_percentage = MinVastMajorityAttestationPercentage::<T>::get();
    // --- Get max epochs in a row a subnet node can be absent from consensus data
    // --- Get the max penalties a subnet node can have before being removed from the network
    let max_subnet_penalty_count = MaxSubnetPenaltyCount::<T>::get();
    // --- Get the attestation percentage for a subnet node to be removed from the network
    //     if they are not included in the validators consensus data
    // --- If this attestation threshold is exceeded, the subnet node that is absent will have its
    //     SubnetNodePenalties incrememented
    let node_attestation_removal_threshold = NodeAttestationRemovalThreshold::<T>::get();
    // --- Get the percentage of the subnet rewards that go to subnet delegate stakers
    let delegate_stake_rewards_percentage: u128 = DelegateStakeRewardsPercentage::<T>::get();

    // let max_subnet_node_registration_epochs = SubnetNodeRegistrationEpochs::<T>::get();
    let subnet_owner_percentage = SubnetOwnerPercentage::<T>::get();

    let total_delegate_stake = TotalDelegateStake::<T>::get();
    let min_subnet_nodes = MinSubnetNodes::<T>::get();

    // --- Get total rewards for this epoch
    let rewards: u128 = Self::get_epoch_emissions(epoch);

    for (subnet_id, data) in SubnetsData::<T>::iter() {
      let mut attestation_percentage: u128 = 0;

      // --- We don't check for minimum nodes because nodes cannot validate or attest if they are not met
      //     as they the validator will not be chosen in ``do_epoch_preliminaries`` if the 
      //     min nodes are not met on that epoch.
      if let Ok(mut submission) = SubnetRewardsSubmission::<T>::try_get(subnet_id, epoch) {
        // --- Get total subnet delegate stake balance
        let total_subnet_delegate_stake = TotalSubnetDelegateStakeBalance::<T>::get(subnet_id);

        // --- Get subnet delegate stake weight
        let subnet_delegate_stake_weight = Self::percent_div(total_subnet_delegate_stake, total_delegate_stake);

        // --- Get overall subnet rewards
        let overall_subnet_reward: u128 = Self::percent_mul(rewards, subnet_delegate_stake_weight);

        // --- Get owner rewards
        let subnet_owner_reward: u128 = Self::percent_mul(overall_subnet_reward, subnet_owner_percentage);

        // --- Get subnet rewards minus owner cut
        let subnet_reward: u128 = overall_subnet_reward.saturating_sub(subnet_owner_reward);

        // --- Get delegators rewards
        let delegate_stake_reward: u128 = Self::percent_mul(subnet_reward, delegate_stake_rewards_percentage);

        // --- Get subnet nodes rewards
        let subnet_node_reward: u128 = subnet_reward.saturating_sub(delegate_stake_reward);
        
        // --- Redundant
        if subnet_node_reward == 0 {
          continue
        }

        // --- Get subnet nodes count to check against attestation count
        // while in this function that should have in the epoch the rewards are destined for
        let subnet_nodes: Vec<T::AccountId> = Self::get_classified_hotkeys(subnet_id, &SubnetNodeClass::Validator, epoch);
        let subnet_node_count = subnet_nodes.len() as u128;

        // --- Ensure nodes are at min requirement to continue rewards operations
        if subnet_node_count < min_subnet_nodes as u128 {
          // We don't give penalties here because they will be given in the next step operation when selecting a new
          // validator
          continue
        }

        let attestations: u128 = submission.attests.len() as u128;
        attestation_percentage = Self::percent_div(attestations, subnet_node_count);

        // Redundant
        // When subnet nodes exit, the consensus data is updated to remove them from it
        if attestation_percentage > Self::PERCENTAGE_FACTOR {
          attestation_percentage = Self::PERCENTAGE_FACTOR;
        }
        
        let validator_subnet_node_id: u32 = submission.validator_id;

        let data_len = submission.data.len();

        // --- If validator submitted no data, or less than the minimum required subnet nodes 
        //     we assume the subnet is broken
        // There is no slashing if subnet is broken, only risk of subnet being removed
        // The subnet is deemed broken is there is no consensus or not enough nodes
        //
        // The subnet has up to the MaxSubnetPenaltyCount to solve the issue before the subnet and all subnet nodes are removed
        if (data_len as u32) < min_subnet_nodes  {
          // --- Increase the penalty count for the subnet because its deemed in a broken state
          SubnetPenaltyCount::<T>::mutate(subnet_id, |n: &mut u32| *n += 1);

          // If the subnet is broken, the validator can avoid slashing by submitting consensus with null data

          // --- If subnet nodes aren't in consensus this is true
          // Since we can assume the subnet is in a broken state, we don't slash the validator
          // even if others do not attest to this state???

          // --- If the subnet nodes are not in agreement with the validator that the model is broken, we
          //     increase the penalty score for the validator
          //     and slash
          // --- We only do this if the vast majority of nodes are not in agreement with the validator
          //     Otherwise we assume the issue just started or is being resolved.
          //     i.e. if a validator sends in no data but by the time some other nodes check, it's resolved,
          //          the validator only gets slashed if the vast majority of nodes disagree at ~87.5%
          //     Or vice versa if validator submits data in a healthy state and the subnet breaks
          //
          // This is an unlikely scenario because all nodes should be checking the subnets state within a few seconds
          // of each other.
          if attestation_percentage < min_vast_majority_attestation_percentage {
            // --- Slash validator and increase penalty score
            Self::slash_validator(subnet_id, validator_subnet_node_id, attestation_percentage, block);
          }

          // --- If the subnet was deemed in a broken stake by the validator, rewards are bypassed
          continue;
        }

        // --- If the minimum required attestation not reached, assume validator is dishonest, slash, and continue
        // We don't increase subnet penalty count here because this is likely the validators fault
        if attestation_percentage < min_attestation_percentage {
          // --- Slash validator and increase penalty score
          Self::slash_validator(subnet_id, validator_subnet_node_id, attestation_percentage, block);
          
          // --- Attestation not successful, move on to next subnet
          continue
        }

        // --- Deposit owners rewards
        match SubnetOwner::<T>::try_get(subnet_id) {
          Ok(coldkey) => {
            let subnet_owner_reward_as_currency = Self::u128_to_balance(subnet_owner_reward);
            if subnet_owner_reward_as_currency.is_some() {
              Self::add_balance_to_coldkey_account(
                &coldkey,
                subnet_owner_reward_as_currency.unwrap()
              );    
            }
          },
          Err(()) => (),
        };

        // --- Get sum of subnet total scores for use of divvying rewards
        let sum = submission.data.iter().fold(0, |acc, x| acc + x.score);

        let max_subnet_node_registration_epochs = SubnetNodeRegistrationEpochs::<T>::get(subnet_id);
        let max_subnet_node_penalties = MaxSubnetNodePenalties::<T>::get(subnet_id);
    
        // --- Reward validators
        for (subnet_node_id, subnet_node) in SubnetNodesData::<T>::iter_prefix(subnet_id) {
          let hotkey: T::AccountId = match SubnetNodeIdHotkey::<T>::try_get(subnet_id, subnet_node_id) {
            Ok(hotkey) => hotkey,
            Err(()) => continue,
          };
      
          // --- (if) Check if subnet node is past the max registration epochs to activate (if registered or deactivated)
          // --- (else if) Check if past Queue and can be included in validation data
          // Always continue if any of these are true
          // Note: Only ``included`` or above nodes can get emissions
          if subnet_node.classification.class <= SubnetNodeClass::Registered {
            if epoch > subnet_node.classification.start_epoch.saturating_add(max_subnet_node_registration_epochs) {
              Self::perform_remove_subnet_node(block, subnet_id, subnet_node_id);
            }
            continue
          } else if subnet_node.classification.class == SubnetNodeClass::Queue {
            // If not, upgrade classification and continue
            // --- Upgrade to included
            Self::increase_class(
              subnet_id, 
              subnet_node_id, 
              epoch,
            );
            continue
          }

          // --- At this point, all nodes should be included in consensus data

          let peer_id: PeerId = subnet_node.peer_id;

          let mut subnet_node_data: SubnetNodeData = SubnetNodeData::default();

          // --- Confirm if ``peer_id`` is present in validator data
          let subnet_node_data_find: Option<(usize, &SubnetNodeData)> = submission.data.iter().enumerate().find(
            |&x| x.1.peer_id == peer_id
          );
          
          // --- If subnet_node_id is present in validator data
          let validated: bool = subnet_node_data_find.is_some();

          if validated {
            subnet_node_data = subnet_node_data_find.unwrap().1.clone();
            submission.data.remove(subnet_node_data_find.unwrap().0);
          }

          let penalties = SubnetNodePenalties::<T>::get(subnet_id, subnet_node_id);

          // --- If node not validated and consensus reached:
          //      otherwise, increment penalty score only
          //      remove them if max penalties threshold is reached
          if !validated {
            // --- Mutate nodes penalties count if not in consensus
            SubnetNodePenalties::<T>::insert(subnet_id, subnet_node_id, penalties + 1);

            // --- To be removed or increase absent count, the consensus threshold must be reached
            if attestation_percentage > node_attestation_removal_threshold {
              // We don't slash nodes for not being in consensus
              // A node can be removed for any reason and may not be due to dishonesty
              // If subnet validators want to remove and slash a node, they can use the proposals mechanism

              // --- Ensure maximum sequential removal consensus threshold is reached
              // We make sure the super majority are in agreeance to remove someone
              // TODO: Check the size of subnet and scale it from there
              if penalties + 1 > max_subnet_node_penalties {
                // --- Increase account penalty count
                Self::perform_remove_subnet_node(block, subnet_id, subnet_node_id);
              }
            }
            // Even if there is a n-1 100% consensus on the node being out of consensus, we don't remove them.
            // In the case where a subnet wants to remove a node, they should initiate a proposal to have them removed
            // using ``propose``method
            continue
          }

          // --- At this point, a subnet node is in the consensus data

          // --- Check if can be included in validation data
          // By this point, node is validated, update to submittable if they have no penalties
          let is_included = subnet_node.classification.class == SubnetNodeClass::Included;
          if is_included && penalties == 0 {
            // --- Upgrade to Validator
            Self::increase_class(
              subnet_id, 
              subnet_node_id, 
              epoch,
            );
            continue
          } else if is_included && penalties != 0 {
            // --- Decrease subnet node penalty count by one if in consensus and attested consensus
            SubnetNodePenalties::<T>::mutate(subnet_id, subnet_node_id, |n: &mut u32| n.saturating_dec());
            continue
          }

          // --- At this point, the subnet node is submittable and included in consensus data

          //
          // TODO: Test removing this ``!submission.attests.contains(&hotkey)`` to allow those that do not attest to gain rewards
          //

          // --- If not attested, do not receive rewards
          // We only penalize accounts on vast majority attestations for not attesting data in case data is corrupted
          // It is up to subnet nodes to remove them via consensus
          // But since consensus was formed at the least, we assume they're against the consensus, therefor likely dishonest
          if !submission.attests.contains_key(&subnet_node_id) {
            if attestation_percentage > min_vast_majority_attestation_percentage {
              // --- Penalize on vast majority only
              SubnetNodePenalties::<T>::insert(subnet_id, subnet_node_id, penalties + 1);
            }  
            continue
          }

          let score = subnet_node_data.score;

          // The subnet node has passed the gauntlet and is about to receive rewards
          
          // --- Decrease subnet node penalty count by one if in consensus and attested consensus
          // Don't hit the storage unless we have to
          if penalties != 0 {
            SubnetNodePenalties::<T>::mutate(subnet_id, subnet_node_id, |n: &mut u32| n.saturating_dec());
          }

          // --- Calculate score percentage of peer versus sum
          let score_percentage: u128 = Self::percent_div(subnet_node_data.score, sum as u128);
          // --- Calculate score percentage of total subnet generated epoch rewards
          let mut account_reward: u128 = Self::percent_mul(score_percentage, subnet_node_reward);

          // --- Increase reward if validator
          if subnet_node_id == validator_subnet_node_id {
            account_reward += Self::get_validator_reward(attestation_percentage);    
          }

          // --- Skip if no rewards to give
          // Unlikely to happen
          if account_reward == 0 {
            continue
          }

          let mut node_delegate_reward = 0;
          if subnet_node.delegate_reward_rate != 0 {
            let total_node_delegated_stake_shares = TotalNodeDelegateStakeShares::<T>::get(subnet_id, subnet_node_id);
            if total_node_delegated_stake_shares != 0 {
              node_delegate_reward = Self::percent_mul(account_reward, subnet_node.delegate_reward_rate);
              account_reward = account_reward - node_delegate_reward;
              Self::do_increase_node_delegate_stake(
                subnet_id,
                subnet_node_id,
                node_delegate_reward,
              );  
            }
          }

          // --- Increase account stake and emit event
          Self::increase_account_stake(
            &hotkey,
            subnet_id, 
            account_reward,
          ); 
        }

        // --- Portion of rewards to delegate stakers
        Self::do_increase_delegate_stake(
          subnet_id,
          delegate_stake_reward,
        );

        // --- Increment down subnet penalty score on successful epochs
        SubnetPenaltyCount::<T>::mutate(subnet_id, |n: &mut u32| n.saturating_dec());
      } else if let Ok(validator_id) = SubnetRewardsValidator::<T>::try_get(subnet_id, epoch) {
        // --- If a validator has been chosen that means they are supposed to be submitting consensus data
        // --- If there is no submission but validator chosen, increase penalty on subnet and validator
        // --- Increase the penalty count for the subnet
        // The next validator on the next epoch can increment the penalty score down
        SubnetPenaltyCount::<T>::mutate(subnet_id, |n: &mut u32| *n += 1);

        // NOTE:
        //  Each subnet increases the penalty score if they don't have the minimum subnet nodes required by the time
        //  the subnet is enabled for emissions. This happens by the blockchain validator before choosing the subnet validator

        // If validator didn't submit anything, then slash
        // Even if a subnet is in a broken state, the chosen validator must submit blank data
        Self::slash_validator(subnet_id, validator_id, 0, block);
      }

      // TODO: Get benchmark for removing max subnets in one epoch to ensure does not surpass max weights

      Self::deposit_event(
        Event::RewardResult { 
          subnet_id: subnet_id, 
          attestation_percentage: attestation_percentage, 
        }
      );
  
      // --- If subnet is past its max penalty count, remove
      let subnet_penalty_count = SubnetPenaltyCount::<T>::get(subnet_id);
      if subnet_penalty_count > max_subnet_penalty_count {
        Self::do_remove_subnet(
          data.path,
          SubnetRemovalReason::MaxPenalties,
        );
      }
    }

    Ok(None.into())
  }

  pub fn reward_subnets_v2(block: u32, epoch: u32) -> DispatchResultWithPostInfo {
    // --- Get total rewards for this epoch
    let rewards: u128 = Self::get_epoch_emissions(epoch);
    log::error!("v2 rewards              {:?}", rewards);

    let subnets: Vec<_> = SubnetsData::<T>::iter()
      .filter(|(_, subnet)| subnet.state == SubnetState::Active)
      .collect();

    let total_subnets: u32 = subnets.len() as u32;
    let total_delegate_stake = TotalDelegateStake::<T>::get();

    let mut stake_weights: BTreeMap<&u32, f64> = BTreeMap::new();
    let mut stake_weight_sum: f64 = 0.0;

    for (subnet_id, _) in &subnets {
      let total_subnet_delegate_stake = TotalSubnetDelegateStakeBalance::<T>::get(subnet_id);
      // 1. Get all weights in f64
      // *We later use sqrt that uses floats

      let weight: f64 = total_subnet_delegate_stake as f64 / total_delegate_stake as f64;
      let weight_sqrt: f64 = sqrt(weight);

      stake_weights.insert(subnet_id, weight_sqrt);
      stake_weight_sum += weight_sqrt;
    }

    let mut stake_weights_normalized: BTreeMap<&u32, u128> = BTreeMap::new();

    // --- Normalize delegate stake weights from `sqrt`
    for (subnet_id, weight) in stake_weights {
      let weight_normalized: u128 = (weight / stake_weight_sum * Self::PERCENTAGE_FACTOR as f64) as u128;
      stake_weights_normalized.insert(subnet_id, weight_normalized);
    }

    let subnet_owner_percentage = SubnetOwnerPercentage::<T>::get();
    let delegate_stake_rewards_percentage: u128 = DelegateStakeRewardsPercentage::<T>::get();
    let min_attestation_percentage = MinAttestationPercentage::<T>::get();
    let min_vast_majority_attestation_percentage = MinVastMajorityAttestationPercentage::<T>::get();
    let min_subnet_nodes = MinSubnetNodes::<T>::get();
    let node_attestation_removal_threshold = NodeAttestationRemovalThreshold::<T>::get();
    let max_subnet_penalty_count = MaxSubnetPenaltyCount::<T>::get();

    for (subnet_id, data) in &subnets {
      let mut attestation_percentage: u128 = 0;

      // --- Get subnet validator submission
      // --- - Run rewards logic
      // --- Otherwise, check if validator exists since they didn't submit incentives consensus
      // --- - Penalize and slash validator if existed
      if let Ok(mut submission) = SubnetRewardsSubmission::<T>::try_get(subnet_id, epoch) {
        // --- Get overall subnet rewards
        let weight: u128 = match stake_weights_normalized.get(&subnet_id) {
          Some(weight) => {
            if weight == &0 {
              continue
            }
            *weight
          },
          None => continue,
        };
        log::error!("v2 weight                {:?}", weight);

        let overall_subnet_reward: u128 = Self::percent_mul(rewards, weight);
        log::error!("v2 overall_subnet_reward {:?}", overall_subnet_reward);

        // --- Get owner rewards
        let subnet_owner_reward: u128 = Self::percent_mul(overall_subnet_reward, subnet_owner_percentage);
        log::error!("v2 subnet_owner_reward   {:?}", subnet_owner_reward);

        // --- Get subnet rewards minus owner cut
        let subnet_reward: u128 = overall_subnet_reward.saturating_sub(subnet_owner_reward);
        log::error!("v2 subnet_reward         {:?}", subnet_reward);

        // --- Get delegators rewards
        let delegate_stake_reward: u128 = Self::percent_mul(subnet_reward, delegate_stake_rewards_percentage);
        log::error!("v2 delegate_stake_reward {:?}", delegate_stake_reward);

        // --- Get subnet nodes rewards total
        let subnet_node_reward: u128 = subnet_reward.saturating_sub(delegate_stake_reward);
        log::error!("v2 subnet_node_reward    {:?}", subnet_node_reward);

        // --- Get subnet nodes count to check against attestation count and make sure min nodes are present during time of rewards
        let subnet_nodes: Vec<T::AccountId> = Self::get_classified_hotkeys(*subnet_id, &SubnetNodeClass::Validator, epoch);
        let subnet_node_count = subnet_nodes.len() as u128;

        // --- Ensure nodes are at min requirement to continue rewards operations
        if subnet_node_count < min_subnet_nodes as u128 {
          // We don't give penalties here because they will be given in the next step operation when selecting a new
          // validator
          continue
        }

        let attestations: u128 = submission.attests.len() as u128;
        attestation_percentage = Self::percent_div(attestations, subnet_node_count);

        // Redundant
        // When subnet nodes exit, the consensus data is updated to remove them from it
        if attestation_percentage > Self::PERCENTAGE_FACTOR {
          attestation_percentage = Self::PERCENTAGE_FACTOR;
        }
        
        let validator_subnet_node_id: u32 = submission.validator_id;

        let data_len = submission.data.len();
        log::error!("data_len {:?}", data_len);

        /* 
          - Ensures the subnet has enough nodes.
            * If validator submits under the minimum nodes we assume the subnet is in an unusable state
          - If the subnet agrees in the validators logic we don't skip rewards
            * This is to not incentivize subnets from falsely attesting any epochs that have under the required nodes.
          - Slashes the validator if attestation is below the required minimum.
        */
        // If the number of data points (data_len) is less than the required minimum subnet nodes
        if (data_len as u32) < min_subnet_nodes {
          // --- Subnet no longer submitting consensus
          //     Increase the penalty count
          SubnetPenaltyCount::<T>::mutate(subnet_id, |n: &mut u32| *n += 1);
          
          // Check if the attestation percentage is below the "vast majority" threshold
          if attestation_percentage < min_vast_majority_attestation_percentage {
            // If the attestation percentage is also below the minimum required threshold, slash the validator
            if attestation_percentage < min_attestation_percentage {
              Self::slash_validator(*subnet_id, validator_subnet_node_id, attestation_percentage, block);
            }
            // Skip further execution and continue to the next iteration
            continue;
          }
          // Subnet agrees with validators submission, continue unless results are None
          if data_len == 0 {
            continue
          }
        }

        // --- If the minimum required attestation not reached, assume validator is dishonest, slash, and continue
        // We don't increase subnet penalty count here because this is likely the validators fault
        if attestation_percentage < min_attestation_percentage {
          // --- Slash validator and increase penalty score
          Self::slash_validator(*subnet_id, validator_subnet_node_id, attestation_percentage, block);
          
          // --- Attestation not successful, move on to next subnet
          continue
        }

        // --- Deposit owners rewards
        match SubnetOwner::<T>::try_get(subnet_id) {
          Ok(coldkey) => {
            let subnet_owner_reward_as_currency = Self::u128_to_balance(subnet_owner_reward);
            if subnet_owner_reward_as_currency.is_some() {
              Self::add_balance_to_coldkey_account(
                &coldkey,
                subnet_owner_reward_as_currency.unwrap()
              );    
            }
          },
          Err(()) => (),
        };

        // --- Get sum of subnet total scores for use of divvying rewards
        let sum = submission.data.iter().fold(0, |acc, x| acc.saturating_add(x.score));

        let max_subnet_node_registration_epochs = SubnetNodeRegistrationEpochs::<T>::get(subnet_id);
        let max_subnet_node_penalties = MaxSubnetNodePenalties::<T>::get(subnet_id);

        for (subnet_node_id, subnet_node) in SubnetNodesData::<T>::iter_prefix(subnet_id) {
          let hotkey: T::AccountId = match SubnetNodeIdHotkey::<T>::try_get(subnet_id, subnet_node_id) {
            Ok(hotkey) => hotkey,
            Err(()) => continue,
          };

          // --- (if) Check if subnet node is past the max registration epochs to activate (if registered or deactivated)
          // --- (else if) Check if past Queue and can be included in validation data
          //
          // Note: Only ``included`` or above nodes can get emissions
          if subnet_node.classification.class <= SubnetNodeClass::Registered {
            if epoch > subnet_node.classification.start_epoch.saturating_add(max_subnet_node_registration_epochs) {
              Self::perform_remove_subnet_node(block, *subnet_id, subnet_node_id);
            }
            continue
          } else if subnet_node.classification.class == SubnetNodeClass::Queue {
            // --- Automatically upgrade to Inclusion if activated into Queue class
            Self::increase_class(*subnet_id, subnet_node_id, epoch);
            continue
          }

          // --- At this point, all nodes can be included in consensus data and receive rewards

          let peer_id: PeerId = subnet_node.peer_id;

          let subnet_node_data_find = submission.data
            .iter()
            .find(|data| data.peer_id == peer_id);
    
          let penalties = SubnetNodePenalties::<T>::get(subnet_id, subnet_node_id);

          if subnet_node_data_find.is_none() {
            // --- Mutate nodes penalties count if not in consensus
            SubnetNodePenalties::<T>::insert(subnet_id, subnet_node_id, penalties + 1);

            // --- To be removed or increase penalty count, the consensus threshold must be reached
            if attestation_percentage > node_attestation_removal_threshold {
              // We don't slash nodes for not being in consensus
              // A node can be removed for any reason such as shutting their node down and may not be due to dishonesty
              // If subnet validators want to remove and slash a node, they can use the proposals mechanism

              // --- Ensure maximum sequential removal consensus threshold is reached
              // We make sure the super majority are in agreeance to remove someone
              // TODO: Check the size of subnet and scale it from there
              if penalties + 1 > max_subnet_node_penalties {
                // --- Increase account penalty count
                Self::perform_remove_subnet_node(block, *subnet_id, subnet_node_id);
              }
            }

            continue
          }
          
          // --- At this point, the subnet node is in the consensus data

          // --- Check if can be included in validation data
          // By this point, node is validated, update to submittable if they have no penalties
          let is_included = subnet_node.classification.class == SubnetNodeClass::Included;
          if is_included && penalties == 0 {
            // --- Upgrade to Validator
            Self::increase_class(*subnet_id, subnet_node_id, epoch);
            continue
          } else if is_included && penalties != 0 {
            // --- Decrease subnet node penalty count by one if in consensus and attested consensus
            SubnetNodePenalties::<T>::mutate(subnet_id, subnet_node_id, |n: &mut u32| n.saturating_dec());
            continue
          }

          // --- At this point, the subnet node is submittable and included in consensus data

          // --- If subnet node does not attest a super majority attested era, we penalize and skip them
          if !submission.attests.contains_key(&subnet_node_id) {
            if attestation_percentage > min_vast_majority_attestation_percentage {
              // --- Penalize on vast majority only
              SubnetNodePenalties::<T>::insert(subnet_id, subnet_node_id, penalties + 1);
              continue
            }  
          }

          let subnet_node_data: SubnetNodeData = subnet_node_data_find.unwrap().clone();

          let score = subnet_node_data.score;

          // --- Validators are allowed to submit scores of 0
          // This is useful if a subnet wants to keep a node around but not give them rewards
          // This can be used in scenarios when the max subnet nodes are reached and they don't
          // want to kick them out as a way to have a waitlist.
          if score == 0 {
            continue
          }

          // --- Decrease subnet node penalty count by one if in consensus and attested consensus
          // Don't hit the db unless we have to
          if penalties != 0 {
            SubnetNodePenalties::<T>::mutate(subnet_id, subnet_node_id, |n: &mut u32| n.saturating_dec());
          }

          // --- Calculate score percentage of peer versus sum
          let score_percentage: u128 = Self::percent_div(subnet_node_data.score, sum as u128);
          log::error!("v2 score_percentage:      {:?}", score_percentage);

          // --- Calculate score percentage of total subnet generated epoch rewards
          let mut account_reward: u128 = Self::percent_mul(score_percentage, subnet_node_reward);
          log::error!("v2 account_reward:             {:?}", account_reward);
          log::error!("v2 subnet_node_reward:         {:?}", subnet_node_reward);

          // --- Skip if no rewards to give
          // Unlikely to happen
          if account_reward == 0 {
            continue
          }

          if subnet_node.delegate_reward_rate != 0 {
            // --- Ensure users are staked to subnet node
            let total_node_delegated_stake_shares = TotalNodeDelegateStakeShares::<T>::get(subnet_id, subnet_node_id);
            if total_node_delegated_stake_shares != 0 {
              log::error!("v2 subnet_node.delegate_reward_rate: {:?}", subnet_node.delegate_reward_rate);

              let node_delegate_reward = Self::percent_mul(account_reward, subnet_node.delegate_reward_rate);
              log::error!("v2 node_delegate_reward:    {:?}", node_delegate_reward);
              log::error!("v2 b4 account_reward:       {:?}", account_reward);

              account_reward = account_reward - node_delegate_reward;
              log::error!("v2 a4 account_reward:       {:?}", account_reward);

              Self::do_increase_node_delegate_stake(
                *subnet_id,
                subnet_node_id,
                node_delegate_reward,
              );  
            }
          }

          // --- Increase reward if validator
          if subnet_node_id == validator_subnet_node_id {
            log::error!("attestation_percentage: {:?}", attestation_percentage);

            account_reward += Self::get_validator_reward(attestation_percentage);    
            log::error!("validator reward here:  {:?}", account_reward);
          }
          
          // --- Increase account stake and emit event
          Self::increase_account_stake(
            &hotkey,
            *subnet_id, 
            account_reward,
          );
        }
        // --- Portion of rewards to delegate stakers
        Self::do_increase_delegate_stake(
          *subnet_id,
          delegate_stake_reward,
        );

        // --- Increment down subnet penalty score on successful epochs if result were greater than or equal to the min required nodes
        if data_len as u32 >= min_subnet_nodes {
          SubnetPenaltyCount::<T>::mutate(subnet_id, |n: &mut u32| n.saturating_dec());
        }
      } else if let Ok(validator_id) = SubnetRewardsValidator::<T>::try_get(subnet_id, epoch) {
        // --- If a validator has been chosen that means they are supposed to be submitting consensus data
        // --- If there is no submission but validator chosen, increase penalty on subnet and validator
        // --- Increase the penalty count for the subnet
        // The next validator on the next epoch can increment the penalty score down
        SubnetPenaltyCount::<T>::mutate(subnet_id, |n: &mut u32| *n += 1);

        // NOTE:
        //  Each subnet increases the penalty score if they don't have the minimum subnet nodes required by the time
        //  the subnet is enabled for emissions. This happens by the blockchain validator before choosing the subnet validator

        // If validator didn't submit anything, then slash
        // Even if a subnet is in a broken state, the chosen validator must submit blank data
        Self::slash_validator(*subnet_id, validator_id, 0, block);
      }
      // TODO: Get benchmark for removing max subnets in one epoch to ensure does not surpass max weights

      Self::deposit_event(
        Event::RewardResult { 
          subnet_id: *subnet_id, 
          attestation_percentage: attestation_percentage, 
        }
      );

      // --- If subnet is past its max penalty count, remove
      let subnet_penalty_count = SubnetPenaltyCount::<T>::get(subnet_id);
      if subnet_penalty_count > max_subnet_penalty_count {
        Self::do_remove_subnet(
          data.path.clone(),
          SubnetRemovalReason::MaxPenalties,
        );
      }
    }

    Ok(None.into())
  }
}