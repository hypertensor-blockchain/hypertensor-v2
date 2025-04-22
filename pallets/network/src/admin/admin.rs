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
use sp_std::vec::Vec;

impl<T: Config> Pallet<T> {
  // TODO: update this for a smoother line
  pub fn set_min_nodes_slope_parameters(mut params: CurveParametersSet) -> DispatchResult {
    let x_curve_start = params.x_curve_start;
    let y_end = params.y_end;
    let y_start = params.y_start;
    let x_rise = Self::PERCENTAGE_FACTOR / 100;

    ensure!(
      y_start > y_end,
      Error::<T>::InvalidCurveParameters
    );

    // --- Linear Slope check
    let x_start_plus_1 = x_curve_start + x_rise;
    let x_start_plus_1_adj = (x_start_plus_1 - x_curve_start) * Self::PERCENTAGE_FACTOR / 
      (Self::PERCENTAGE_FACTOR - x_curve_start);
    let y_start_minus_1 = (y_start - y_end) * (Self::PERCENTAGE_FACTOR - x_start_plus_1_adj) / 
      Self::PERCENTAGE_FACTOR + y_end; 
    let y_rise = y_start - y_start_minus_1;
    let slope = y_rise * Self::PERCENTAGE_FACTOR / x_rise;
    let j = slope * Self::TWO_HUNDRED_PERCENT_FACTOR / Self::PERCENTAGE_FACTOR;
    let q = Self::PERCENTAGE_FACTOR * Self::PERCENTAGE_FACTOR / j * y_start / Self::PERCENTAGE_FACTOR;
    let max_x = 
      Self::PERCENTAGE_FACTOR * Self::PERCENTAGE_FACTOR / j * y_start / Self::PERCENTAGE_FACTOR + 
      (x_curve_start * Self::PERCENTAGE_FACTOR / Self::TWO_HUNDRED_PERCENT_FACTOR);
    
    ensure!(
      max_x >= Self::PERCENTAGE_FACTOR,
      Error::<T>::SubnetNotExist
    );

    params.max_x = max_x;

    MinNodesCurveParameters::<T>::put(params);

    Ok(())
  }

  pub fn set_base_subnet_node_memory_mb(value: u128) -> DispatchResult {
    BaseSubnetNodeMemoryMB::<T>::put(value);
    Ok(())
  }

  pub fn set_max_subnet_memory_mb(value: u128) -> DispatchResult {
    MaxSubnetMemoryMB::<T>::put(value);
    Ok(())
  }

  pub fn set_overall_max_subnet_memory_mb(value: u128) -> DispatchResult {
    MaxTotalSubnetMemoryMB::<T>::put(value);
    Ok(())
  }

  pub fn set_proposal_min_subnet_nodes(value: u32) -> DispatchResult {
    ProposalMinSubnetNodes::<T>::put(value);
    Ok(())
  }
  
  pub fn set_subnet_node_registration_epochs(value: u64) -> DispatchResult {
    SubnetNodeRegistrationEpochs::<T>::put(value);
    Ok(())
  }

  // TODO: remove target multipler logic from application
  pub fn set_target_subnet_node_multiplier(value: u128) -> DispatchResult {
    TargetSubnetNodesMultiplier::<T>::put(value);
    Ok(())
  }

  pub fn set_subnet_memory(subnet_id: u32, memory_mb: u128) -> DispatchResult {
    let subnet = match SubnetsData::<T>::try_get(subnet_id) {
      Ok(subnet) => subnet,
      Err(()) => return Err(Error::<T>::SubnetNotExist.into()),
    };

    ensure!(
      memory_mb <= MaxSubnetMemoryMB::<T>::get(),
      Error::<T>::InvalidMaxSubnetMemoryMB
    );

    let base_node_memory: u128 = BaseSubnetNodeMemoryMB::<T>::get();

    let min_subnet_nodes: u32 = Self::get_min_subnet_nodes(base_node_memory, memory_mb);
    let target_subnet_nodes: u32 = Self::get_target_subnet_nodes(min_subnet_nodes);

    let subnet_data = SubnetData {
      id: subnet_id,
      path: subnet.path,
      min_nodes: min_subnet_nodes,
      target_nodes: target_subnet_nodes,
      memory_mb: memory_mb,  
      initialized: subnet.initialized,
      registration_blocks: subnet.registration_blocks,
      activated: subnet.activated,
      entry_interval: subnet.entry_interval,
    };

    SubnetsData::<T>::insert(subnet_id, subnet_data);

    Ok(())
  }

  pub fn set_max_subnets(value: u32) -> DispatchResult {
    MaxSubnets::<T>::set(value);

    Self::deposit_event(Event::SetMaxSubnets(value));

    Ok(())
  }

  pub fn set_min_subnet_nodes(value: u32) -> DispatchResult {
    ensure!(
      value > 0 && value < MaxSubnetNodes::<T>::get(),
      Error::<T>::InvalidMinSubnetNodes
    );

    MinSubnetNodes::<T>::set(value);
    Self::deposit_event(Event::SetMinSubnetNodes(value));
    Ok(())
  }

  pub fn do_set_max_subnet_nodes(value: u32) -> DispatchResult {
    ensure!(
      value > MinSubnetNodes::<T>::get(),
      Error::<T>::InvalidMaxSubnetNodes
    );

    MaxSubnetNodes::<T>::set(value);

    Self::deposit_event(Event::SetMaxSubnetNodes(value));

    Ok(())
  }

  pub fn do_set_min_stake_balance(value: u128) -> DispatchResult {
    ensure!(
      value > 0,
      Error::<T>::InvalidMinStakeBalance
    );

    MinStakeBalance::<T>::set(value);

    Self::deposit_event(Event::SetMinStakeBalance(value));

    Ok(())
  }

  pub fn set_tx_rate_limit(value: u64) -> DispatchResult {
    TxRateLimit::<T>::set(value);

    Self::deposit_event(Event::SetTxRateLimit(value));

    Ok(())
  }
}