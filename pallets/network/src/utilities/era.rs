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
use frame_system::pallet_prelude::BlockNumberFor;

impl<T: Config> Pallet<T> {
  pub fn do_epoch_preliminaries(block: u64, epoch: u32, epoch_length: u64) {
    let max_subnet_penalty_count = MaxSubnetPenaltyCount::<T>::get();
    let subnet_activation_enactment_period = SubnetActivationEnactmentPeriod::<T>::get();

    let subnets: Vec<_> = SubnetsData::<T>::iter().collect();
    let total_subnets: u32 = subnets.len() as u32;
    let excess_subnets: bool = total_subnets > MaxSubnets::<T>::get();
    let mut subnet_delegate_stake: Vec<(Vec<u8>, u128)> = Vec::new();

    for (subnet_id, data) in subnets {

      //
      //
      // TODO: Check Registration and Enactment period separately:
      //       Registration Period:
      //         - Can exist no matter what
      //       Enactment Period:
      //         - Once out of registration, if must have min nodes and min delegate stake.
      //       Out of Enactment Period:
      //         - Remove if not activated, althought should be automatically removed in Enactment if it didn't
      //           meet HT requirements.
      // let max_registration_block = data.initialized + data.registration_blocks;
      // let max_enactment_block = max_registration_block + subnet_activation_enactment_period;

      // --- Ensure subnet is active is able to submit consensus
      let max_registration_block = data.initialized + data.registration_blocks + subnet_activation_enactment_period;
      if data.activated == 0 && block <= max_registration_block {
        // We check if the subnet is still in registration phase and not yet out of the enactment phase
        continue
      } else if data.activated == 0 && block > max_registration_block {
        // --- Ensure subnet is in registration period and hasn't passed enactment period
        // If subnet hasn't been activated after the enacement period, then remove subnet
				Self::deactivate_subnet(
					data.path,
					SubnetRemovalReason::EnactmentPeriod,
				);
        continue
			}

      // --- All subnets are now activated and passed the registration period
      // Must have:
      //  - Minimum nodes (increases penalties if less than)
      //  - Minimum delegate stake balance (remove subnet if less than)

      let min_subnet_nodes = data.min_nodes;
			let subnet_delegate_stake_balance = TotalSubnetDelegateStakeBalance::<T>::get(subnet_id);
			let min_subnet_delegate_stake_balance = Self::get_min_subnet_delegate_stake_balance(min_subnet_nodes);

      // --- Ensure min delegate stake balance is met
      if subnet_delegate_stake_balance < min_subnet_delegate_stake_balance {
        Self::deactivate_subnet(
          data.path,
          SubnetRemovalReason::MinSubnetDelegateStake,
        );
        continue
      }

      // --- Get all possible validators
      let subnet_node_ids: Vec<u32> = Self::get_classified_subnet_node_ids(subnet_id, &SubnetNodeClass::Validator, epoch as u64);
      let subnet_nodes_count = subnet_node_ids.len();
      
      // --- Ensure min nodes are active
      // Only choose validator if min nodes are present
      // The ``SubnetPenaltyCount`` when surpassed doesn't penalize anyone, only removes the subnet from the chain
      if (subnet_nodes_count as u32) < min_subnet_nodes {
        SubnetPenaltyCount::<T>::mutate(subnet_id, |n: &mut u32| *n += 1);
      }

      // --- Check penalties and remove subnet is threshold is breached
      let penalties = SubnetPenaltyCount::<T>::get(subnet_id);
      if penalties > max_subnet_penalty_count {
        Self::deactivate_subnet(
          data.path,
          SubnetRemovalReason::MaxPenalties,
        );
        continue
      }

      if excess_subnets {
        subnet_delegate_stake.push((data.path, subnet_delegate_stake_balance));
      }

      Self::choose_validator(
        block,
        subnet_id,
        subnet_node_ids.clone(),
        min_subnet_nodes,
        epoch,
      );
    }

    if excess_subnets {
      subnet_delegate_stake.sort_by_key(|&(_, value)| value);
      Self::deactivate_subnet(
        subnet_delegate_stake[0].0.clone(),
        SubnetRemovalReason::MaxSubnets,
      );
    }
  }

  pub fn get_current_block_as_u64() -> u64 {
    TryInto::try_into(<frame_system::Pallet<T>>::block_number())
      .ok()
      .expect("blockchain will not exceed 2^64 blocks; QED.")
  }

  pub fn convert_block_as_u64(block: BlockNumberFor<T>) -> u64 {
    TryInto::try_into(block)
      .ok()
      .expect("blockchain will not exceed 2^64 blocks; QED.")
  }
  
  pub fn get_current_epoch_as_u32() -> u32 {
    let current_block = Self::get_current_block_as_u64();
    let epoch_length: u64 = T::EpochLength::get();
    (current_block / epoch_length) as u32
  }
}