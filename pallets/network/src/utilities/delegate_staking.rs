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
  pub fn get_min_subnet_delegate_stake_balance() -> u128 {
    let total_network_issuance = Self::get_total_network_issuance();
    let factor: u128 = MinSubnetDelegateStakeFactor::<T>::get();
    Self::percent_mul(total_network_issuance, factor)
  }

  // TODO: Get min delegate stake on any epoch or block
}