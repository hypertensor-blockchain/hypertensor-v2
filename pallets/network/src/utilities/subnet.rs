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

impl<T: Config> Pallet<T> {
  pub fn get_min_subnet_nodes(base_node_memory: u128, memory_mb: u128) -> u32 {
    0
  }

  pub fn get_target_subnet_nodes(min_subnet_nodes: u32) -> u32 {
    min_subnet_nodes
  }

  pub fn registration_cost(epoch: u32) -> u128 {
    let last_registration_epoch = LastSubnetRegistrationEpoch::<T>::get();
    let fee_min: u128 = MinSubnetRegistrationFee::<T>::get();
    let fee_max: u128 = MaxSubnetRegistrationFee::<T>::get();
    let period: u32 = SubnetRegistrationInterval::<T>::get();

    // Calculate the start of the next registration period
    let next_registration_lower_bound_epoch = Self::get_next_registration_epoch(last_registration_epoch);
    let next_registration_upper_bound_epoch = next_registration_lower_bound_epoch + period;

    // If the current epoch is beyond the next registration period, return min fee
    if epoch >= next_registration_upper_bound_epoch {
        return fee_min;
    }

    // Calculate the current position within the registration period
    let cycle_epoch = epoch.saturating_sub(next_registration_lower_bound_epoch);

    // Calculate the fee decrease per epoch
    let decrease_per_epoch = (fee_max.saturating_sub(fee_min)).saturating_div(period as u128);

    // Calculate the current fee
    let cost = fee_max.saturating_sub(decrease_per_epoch.saturating_mul(cycle_epoch as u128));

    // Ensure the fee doesn't go below the minimum
    cost.max(fee_min)
  }

  fn get_registration_cost(
    current_epoch: u32, 
    last_registration_epoch: u32, 
    fee_min: u128, 
    fee_max: u128
  ) {
    let next_registration_lower_bound_epoch = Self::get_next_registration_epoch(last_registration_epoch);

  }

  pub fn can_subnet_register(current_epoch: u32) -> bool {
    current_epoch >= Self::get_next_registration_epoch(current_epoch)
  }

  /// Get the next registration epoch based on an epoch
  pub fn get_next_registration_epoch(current_epoch: u32) -> u32 {
    let last_registration_epoch: u32 = LastSubnetRegistrationEpoch::<T>::get();
    // --- Handle genesis
    if last_registration_epoch == 0 {
      return 0
    }
    let period: u32 = SubnetRegistrationInterval::<T>::get();
    last_registration_epoch.saturating_add(
      period.saturating_sub(last_registration_epoch % period)
    )
  }

  pub fn is_subnet_registering(subnet_id: u32, state: SubnetState, epoch: u32) -> bool {
    let subnet_registration_epochs = SubnetRegistrationEpochs::<T>::get();
    let is_registering: bool = state == SubnetState::Registered;

    match SubnetRegistrationEpoch::<T>::try_get(subnet_id) {
      Ok(registered_epoch) => {
        let max_registration_epoch = registered_epoch.saturating_add(subnet_registration_epochs);
        if is_registering && epoch <= max_registration_epoch {
          return true
        }
        false
      },
      Err(()) => false,
    }
  }

  pub fn is_subnet_in_enactment(subnet_id: u32, state: SubnetState, epoch: u32) -> bool {
    let subnet_registration_epochs = SubnetRegistrationEpochs::<T>::get();
    let subnet_activation_enactment_epochs = SubnetActivationEnactmentEpochs::<T>::get();

    let is_registering: bool = state == SubnetState::Registered;

    match SubnetRegistrationEpoch::<T>::try_get(subnet_id) {
      Ok(registered_epoch) => {
        let max_registration_epoch = registered_epoch.saturating_add(subnet_registration_epochs);
        let max_enactment_epoch = max_registration_epoch.saturating_add(subnet_activation_enactment_epochs);
    
        if is_registering && epoch <= max_registration_epoch {
          return false
        } else if is_registering && epoch <= max_enactment_epoch {
          return true
        }
        false
      },
      Err(()) => false,
    }





    // let registered_epoch: u32 = subnet_data.registered;
    // let max_registration_epoch = registered_epoch.saturating_add(subnet_registration_epochs);
    // let max_enactment_epoch = max_registration_epoch.saturating_add(subnet_activation_enactment_epochs);

    // if is_registering && epoch <= max_registration_epoch {
    //   return false
    // } else if is_registering && epoch <= max_enactment_epoch {
    //   return true
    // }
    // false
  }

  pub fn is_subnet_active(subnet_id: u32) -> bool {
    match SubnetsData::<T>::try_get(subnet_id) {
      Ok(subnet) => subnet.state == SubnetState::Active,
      Err(()) => false,
    }
  }

  pub fn is_subnet_owner(account_id: &T::AccountId, subnet_id: u32) -> bool {
    match SubnetOwner::<T>::try_get(subnet_id) {
      Ok(owner) => &owner == account_id,
      Err(()) => false,
    }
  }
}