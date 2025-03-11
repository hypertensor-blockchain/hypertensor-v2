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
use frame_support::pallet_prelude::DispatchResultWithPostInfo;
use frame_support::pallet_prelude::Pays;

impl<T: Config> Pallet<T> {
  /// Submit subnet scores per subnet node
  /// Validator of the epoch receives rewards when attestation passes consensus
  pub fn do_validate(
    subnet_id: u32, 
    hotkey: T::AccountId,
    block: u64, 
    epoch_length: u64,
    epoch: u32,
    mut data: Vec<SubnetNodeData>,
    args: Option<BoundedVec<u8, DefaultValidatorArgsLimit>>,
  ) -> DispatchResultWithPostInfo {
    // TODO: Add max sum to avoid overflow

    // --- Ensure current subnet validator by its hotkey
    let validator_id = SubnetRewardsValidator::<T>::get(subnet_id, epoch).ok_or(Error::<T>::InvalidValidator)?;

    // --- If hotkey is hotkey, ensure it matches validator, otherwise if coldkey -> get hotkey
    ensure!(
      SubnetNodeIdHotkey::<T>::get(subnet_id, validator_id) == Some(hotkey.clone()),
      Error::<T>::InvalidValidator
    );

    // --- Ensure not submitted already
    ensure!(
      !SubnetRewardsSubmission::<T>::contains_key(subnet_id, epoch),
      Error::<T>::SubnetRewardsAlreadySubmitted
    );

    // Remove duplicates based on peer_id
    data.dedup_by(|a, b| a.peer_id == b.peer_id);

    // Remove idle classified entries
    // Each peer must have an inclusion classification at minimum
    data.retain(|x| {
      match SubnetNodesData::<T>::try_get(
        subnet_id, 
        SubnetNodeAccount::<T>::get(subnet_id, &x.peer_id)
      ) {
        Ok(subnet_node) => subnet_node.has_classification(&SubnetNodeClass::Included, epoch as u64),
        Err(()) => false,
      }
    });

    //
    // --- Qualify the data
    //

    // --- Get count of eligible nodes that can be submitted for consensus rewards
    // This is the maximum amount of nodes that can be entered
    let included_nodes: Vec<u32> = Self::get_classified_subnet_node_ids(subnet_id, &SubnetNodeClass::Included, epoch as u64);
    let included_nodes_count = included_nodes.len();

    // --- Ensure data isn't greater than current registered subnet peers
    // Redundant because of ``retain``
    ensure!(
      data.len() as u32 <= included_nodes_count as u32,
      Error::<T>::InvalidRewardsDataLength
    );
    
    // --- Validator auto-attests the epoch
    let mut attests: BTreeMap<u32, u64> = BTreeMap::new();
    attests.insert(validator_id, block);

    let rewards_data: RewardsData = RewardsData {
      validator_id: validator_id,
      attests: attests,
      data: data,
      args: args,
    };

    SubnetRewardsSubmission::<T>::insert(subnet_id, epoch, rewards_data);
  
    Self::deposit_event(
      Event::ValidatorSubmission { 
        subnet_id: subnet_id, 
        account_id: hotkey, 
        epoch: epoch,
      }
    );

    Ok(Pays::No.into())
  }

    /// Attest validator subnet rewards data
  // Nodes must attest data to receive rewards
  pub fn do_attest(
    subnet_id: u32, 
    hotkey: T::AccountId,
    block: u64, 
    epoch_length: u64,
    epoch: u32,
  ) -> DispatchResultWithPostInfo {
    // --- Ensure subnet node exists under hotkey
    let subnet_node_id = match HotkeySubnetNodeId::<T>::try_get(
      subnet_id, 
      hotkey.clone()
    ) {
      Ok(subnet_node_id) => subnet_node_id,
      Err(()) => return Err(Error::<T>::SubnetNodeNotExist.into()),
    };

    // --- Ensure node classified to attest
    match SubnetNodesData::<T>::try_get(
      subnet_id, 
      subnet_node_id
    ) {
      Ok(subnet_node) => subnet_node.has_classification(&SubnetNodeClass::Validator, epoch as u64),
      Err(()) => return Err(Error::<T>::SubnetNodeNotExist.into()),
    };

    SubnetRewardsSubmission::<T>::try_mutate_exists(
      subnet_id,
      epoch.clone(),
      |maybe_params| -> DispatchResult {
        let params = maybe_params.as_mut().ok_or(Error::<T>::InvalidSubnetRewardsSubmission)?;
        let mut attests = &mut params.attests;

        ensure!(attests.insert(subnet_node_id, block) == None, Error::<T>::AlreadyAttested);

        params.attests = attests.clone();
        Ok(())
      }
    )?;

    Self::deposit_event(
      Event::Attestation { 
        subnet_id: subnet_id, 
        account_id: hotkey, 
        epoch: epoch,
      }
    );

    Ok(Pays::No.into())
  }

  pub fn choose_validator(
    block: u64,
    subnet_id: u32,
    subnet_node_ids: Vec<u32>,
    min_subnet_nodes: u32,
    epoch: u32,
  ) {
    // TODO: Make sure this is only called if subnet is activated and on the following epoch
    
    // Redundant
    // If validator already chosen, then return
    if let Ok(validator_id) = SubnetRewardsValidator::<T>::try_get(subnet_id, epoch) {
      return
    }

    let subnet_nodes_len = subnet_node_ids.len();
    
    // --- Ensure min subnet peers that are submittable are at least the minimum required
    // --- Consensus cannot begin until this minimum is reached
    // --- If not min subnet peers count then accountant isn't needed
    if (subnet_nodes_len as u32) < min_subnet_nodes {
      return
    }

    // --- n-1 to get 0 index in the randomization
    let rand_index = Self::get_random_number(subnet_nodes_len as u32, block as u32);

    // --- Choose random accountant from eligible accounts
    let validator: &u32 = &subnet_node_ids[rand_index as usize];

    // --- Insert validator for next epoch
    SubnetRewardsValidator::<T>::insert(subnet_id, epoch, validator);
  }

  /// Return the validators reward that submitted data on the previous epoch
  // The attestation percentage must be greater than the MinAttestationPercentage
  pub fn get_validator_reward(
    attestation_percentage: u128,
  ) -> u128 {
    if MinAttestationPercentage::<T>::get() > attestation_percentage {
      return 0
    }
    Self::percent_mul(BaseValidatorReward::<T>::get(), attestation_percentage)
  }

  pub fn slash_validator(
    subnet_id: u32, 
    subnet_node_id: u32,
    attestation_percentage: u128, 
    block: u64
  ) {
    // We never ensure balance is above 0 because any hotkey chosen must have the target stake
    // balance at a minimum
    let hotkey = SubnetNodeIdHotkey::<T>::get(subnet_id, subnet_node_id).unwrap();

    // --- Get stake balance
    // This could be greater than the target stake balance
    let account_subnet_stake: u128 = AccountSubnetStake::<T>::get(hotkey.clone(), subnet_id);

    // --- Get slash amount up to max slash
    //
    let mut slash_amount: u128 = Self::percent_mul(account_subnet_stake, SlashPercentage::<T>::get());
    // --- Update slash amount up to attestation percent
    slash_amount = Self::percent_mul(slash_amount, Self::PERCENTAGE_FACTOR - attestation_percentage);
    // --- Update slash amount up to max slash
    let max_slash: u128 = MaxSlashAmount::<T>::get();
    if slash_amount > max_slash {
      slash_amount = max_slash
    }
    
    // --- Decrease account stake
    Self::decrease_account_stake(
      &hotkey.clone(),
      subnet_id, 
      slash_amount,
    );

    // --- Increase validator penalty count
    let penalties = SubnetNodePenalties::<T>::get(subnet_id, subnet_node_id);
    SubnetNodePenalties::<T>::insert(subnet_id, subnet_node_id, penalties + 1);

    // --- Ensure maximum sequential removal consensus threshold is reached
    if penalties + 1 > MaxSubnetNodePenalties::<T>::get() {
      // --- Increase account penalty count
      Self::perform_remove_subnet_node(block, subnet_id, subnet_node_id);
    } else {
      
    }

    Self::deposit_event(
      Event::Slashing { 
        subnet_id: subnet_id, 
        account_id: hotkey, 
        amount: slash_amount,
      }
    );

  }
}