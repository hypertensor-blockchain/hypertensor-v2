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
    // TODO: Needs to be updated for smoother curve
    //
    //
    //
    //

    // --- DEFAULT
    // --- Get min nodes based on default memory settings
    let simple_min_subnet_nodes: u128 = Self::percent_div(
      memory_mb * Self::PERCENTAGE_FACTOR, 
      base_node_memory * Self::PERCENTAGE_FACTOR
    );

    // --- Parameters
    let params: CurveParametersSet = MinNodesCurveParameters::<T>::get();
    let one_hundred = Self::PERCENTAGE_FACTOR;
    let x_curve_start = params.x_curve_start;
    let y_end = params.y_end;
    let y_start = params.y_start;
    let x_rise = Self::PERCENTAGE_FACTOR / 100;
    let max_x = params.max_x;

    let max_subnet_memory = MaxSubnetMemoryMB::<T>::get();

    let mut subnet_mem_position = Self::PERCENTAGE_FACTOR;
    
    // --- Get the x axis of the memory based on min and max
    // Redundant since subnet memory cannot be surpassed beyond the max subnet memory
    // If max subnet memory in curve is surpassed
    if memory_mb < max_subnet_memory {
      subnet_mem_position = memory_mb * Self::PERCENTAGE_FACTOR / max_subnet_memory;
    }

    let mut min_subnet_nodes: u128 = MinSubnetNodes::<T>::get() as u128 * Self::PERCENTAGE_FACTOR;

    if subnet_mem_position <= x_curve_start {
      if simple_min_subnet_nodes  > min_subnet_nodes {
        min_subnet_nodes = simple_min_subnet_nodes;
      }
      return (min_subnet_nodes / Self::PERCENTAGE_FACTOR) as u32
    }

    let mut x = 0;

    if subnet_mem_position >= x_curve_start && subnet_mem_position <= Self::PERCENTAGE_FACTOR {
      // If subnet memory position is in between range
      x = (subnet_mem_position-x_curve_start) * Self::PERCENTAGE_FACTOR / (Self::PERCENTAGE_FACTOR-x_curve_start);
    } else if subnet_mem_position > Self::PERCENTAGE_FACTOR {
      // If subnet memory is greater than 100%
      x = Self::PERCENTAGE_FACTOR;
    }

    let y = (y_start - y_end) * (Self::PERCENTAGE_FACTOR - x) / Self::PERCENTAGE_FACTOR + y_end;

    let y_ratio = Self::percent_div(y, y_start);

    let mut min_subnet_nodes_on_curve = Self::percent_mul(simple_min_subnet_nodes, y_ratio);

    // --- If over the max x position, we increase the min nodes to ensure the amount is never less than the previous position
    if subnet_mem_position > max_x {
      let x_max_x_ratio = Self::percent_div(max_x, subnet_mem_position);
      let comp_fraction = Self::PERCENTAGE_FACTOR - x_max_x_ratio;
      min_subnet_nodes_on_curve = Self::percent_mul(
        simple_min_subnet_nodes, 
        comp_fraction
      ) + min_subnet_nodes_on_curve;
    }

    // Redundant
    if min_subnet_nodes_on_curve > min_subnet_nodes {
      min_subnet_nodes = min_subnet_nodes_on_curve;
    }

    (min_subnet_nodes / Self::PERCENTAGE_FACTOR) as u32
  }

  pub fn get_target_subnet_nodes(min_subnet_nodes: u32) -> u32 {
    Self::percent_mul(
      min_subnet_nodes.into(), 
      TargetSubnetNodesMultiplier::<T>::get()
    ) as u32 + min_subnet_nodes
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
    let cycle_epoch = epoch - next_registration_lower_bound_epoch;

    // Calculate the fee decrease per epoch
    let decrease_per_epoch = (fee_max - fee_min) / period as u128;

    // Calculate the current fee
    let cost = fee_max - (decrease_per_epoch * cycle_epoch as u128);

    // Ensure the fee doesn't go below the minimum
    cost.max(fee_min)
  }

  // pub fn registration_cost(epoch: u32) -> u128 {
  //   let last_registration_epoch = LastSubnetRegistrationEpoch::<T>::get();
  //   let fee_min: u128 = MinSubnetRegistrationFee::<T>::get();

  //   // First registration is min fee (possibly fix this)
  //   if last_registration_epoch == 0 {
  //     return fee_min
  //   }

  //   // --- Get the nexr registration epoch based on the last subnet registered epoch
  //   // This can be lower than the current epoch if no registrations were in the previous period or prior to the previous period
  //   let next_registration_lower_bound_epoch = Self::get_next_registration_epoch(last_registration_epoch);
  //   let period: u32 = SubnetRegistrationInterval::<T>::get();
  //   let next_registration_upper_bound_epoch = next_registration_lower_bound_epoch + period;

  //   // If no registration within period or previous periods, keep at `fee_min`
  //   // # Example
  //   // *`epoch`: 100
  //   // *`next_registration_upper_bound_epoch`: 200
  //   // *`period`: 100
  //   if next_registration_upper_bound_epoch <= epoch {
  //     return fee_min
  //   }

  //   let fee_max: u128 = MaxSubnetRegistrationFee::<T>::get();

  //   // Epoch within the cycle
  //   let cycle_epoch = epoch % period;
  //   let decrease_per_epoch = (fee_max.saturating_sub(fee_min)).saturating_div(period as u128);
  //   let cost = fee_max.saturating_sub(decrease_per_epoch.saturating_mul(cycle_epoch as u128));

  //   // Ensures cost doesn't go below min
  //   cost.max(fee_min)
  // }

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
    last_registration_epoch + (
      period - (last_registration_epoch % period)
    )
  }
}