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
  /// The minimum delegate stake balance for a subnet to stay live
  pub fn get_min_subnet_delegate_stake_balance(min_subnet_nodes: u32) -> u128 {
    // --- Get minimum stake balance per subnet node
    let min_stake_balance = MinStakeBalance::<T>::get();
    // --- Get minimum subnet stake balance
    let min_subnet_stake_balance = min_stake_balance * min_subnet_nodes as u128;
    // --- Get required delegate stake balance for a subnet to have to stay live
    let min_subnet_delegate_stake_balance = Self::percent_mul(
      min_subnet_stake_balance, 
      MinSubnetDelegateStakePercentage::<T>::get()
    );
    // --- Get absolute minimum required subnet delegate stake balance
    let min_subnet_delegate_stake = MinSubnetDelegateStake::<T>::get();
    // --- Return here if the absolute minimum required subnet delegate stake balance is greater
    //     than the calculated minimum requirement
    if min_subnet_delegate_stake > min_subnet_delegate_stake_balance {
      return min_subnet_delegate_stake
    }
    min_subnet_delegate_stake_balance
  }
}