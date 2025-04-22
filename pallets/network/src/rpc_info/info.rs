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
  pub fn get_subnet_nodes(
    subnet_id: u32,
  ) -> Vec<SubnetNode<T::AccountId>> {
    if !SubnetsData::<T>::contains_key(subnet_id) {
      return Vec::new();
    }
    let epoch: u32 = Self::get_current_epoch_as_u32();
    Self::get_classified_subnet_nodes(subnet_id, &SubnetNodeClass::Queue, epoch)
  }

  pub fn get_subnet_nodes_included(
    subnet_id: u32,
  ) -> Vec<SubnetNode<T::AccountId>> {
    if !SubnetsData::<T>::contains_key(subnet_id) {
      return Vec::new();
    }
    let epoch: u32 = Self::get_current_epoch_as_u32();
    Self::get_classified_subnet_nodes(subnet_id, &SubnetNodeClass::Included, epoch)
  }

  pub fn get_subnet_nodes_submittable(
    subnet_id: u32,
  ) -> Vec<SubnetNode<T::AccountId>> {
    if !SubnetsData::<T>::contains_key(subnet_id) {
      return Vec::new();
    }
    let epoch: u32 = Self::get_current_epoch_as_u32();
    Self::get_classified_subnet_nodes(subnet_id, &SubnetNodeClass::Validator, epoch)
  }

  pub fn get_subnet_node_info(
    subnet_id: u32,
  ) -> Vec<SubnetNodeInfo<T::AccountId>> {
    if !SubnetsData::<T>::contains_key(subnet_id) {
      return Vec::new();
    }
    let epoch: u32 = Self::get_current_epoch_as_u32();
    Self::get_classified_subnet_node_info(subnet_id, &SubnetNodeClass::Validator, epoch)
  }

  pub fn get_subnet_nodes_subnet_unconfirmed_count(
    subnet_id: u32,
  ) -> u32 {
    if !SubnetsData::<T>::contains_key(subnet_id) {
      return 0;
    }

    0
  }

  pub fn get_subnet_node_by_params(
    subnet_id: u32,
    a: BoundedVec<u8, DefaultSubnetNodeUniqueParamLimit>,
  ) -> Option<SubnetNode<T::AccountId>> {
    if !SubnetsData::<T>::contains_key(subnet_id) {
      return None
    }

    SubnetNodesData::<T>::iter_prefix_values(subnet_id)
      .find(|x| {
        // Find by ``a``, a unique parameter
        x.a == Some(a.clone())
      })
  }

  // id is consensus ID
  pub fn get_consensus_data(
    subnet_id: u32,
    epoch: u32
  ) -> Option<RewardsData> {
    let data = SubnetRewardsSubmission::<T>::get(subnet_id, epoch);
    Some(data?)
  }

  // pub fn get_incentives_data(
  //   subnet_id: u32,
  //   epoch: u32
  // ) -> Option<RewardsData> {
  //   let data = SubnetRewardsSubmission::<T>::get(subnet_id, epoch);
  //   Some(data?)
  // }

  pub fn get_minimum_subnet_nodes(memory_mb: u128) -> u32 {
    MinSubnetNodes::<T>::get()
  }

  pub fn get_minimum_delegate_stake(memory_mb: u128) -> u128 {
    Self::get_min_subnet_delegate_stake_balance()
  }

  pub fn get_subnet_node_stake_by_peer_id(subnet_id: u32, peer_id: PeerId) -> u128 {
    match PeerIdSubnetNode::<T>::try_get(subnet_id, &peer_id) {
      Ok(subnet_node_id) => {
        let hotkey = SubnetNodeIdHotkey::<T>::get(subnet_id, subnet_node_id).unwrap(); // TODO: error fallback
        AccountSubnetStake::<T>::get(hotkey, subnet_id)
      },
      Err(()) => 0,
    }
  }

  // TODO: Make this only return true is Validator subnet node
  pub fn is_subnet_node_by_peer_id(subnet_id: u32, peer_id: Vec<u8>) -> bool {
    match PeerIdSubnetNode::<T>::try_get(subnet_id, PeerId(peer_id)) {
      Ok(_) => true,
      Err(()) => false,
    }
  }

  pub fn is_subnet_node_by_bootstrap_peer_id(subnet_id: u32, peer_id: Vec<u8>) -> bool {
    match BootstrapPeerIdSubnetNode::<T>::try_get(subnet_id, PeerId(peer_id)) {
      Ok(_) => true,
      Err(()) => false,
    }
  }

  pub fn are_subnet_nodes_by_peer_id(subnet_id: u32, peer_ids: Vec<Vec<u8>>) -> BTreeMap<Vec<u8>, bool> {
    let mut subnet_nodes: BTreeMap<Vec<u8>, bool> = BTreeMap::new();

    for peer_id in peer_ids.iter() {
      let is = match PeerIdSubnetNode::<T>::try_get(subnet_id, PeerId(peer_id.clone())) {
        Ok(_) => true,
        Err(()) => false,
      };
      subnet_nodes.insert(peer_id.clone(), is);
    }

    subnet_nodes
  }

  /// If subnet node exists under unique subnet node parameter ``a``
  pub fn is_subnet_node_by_a(
    subnet_id: u32, 
    a: BoundedVec<u8, DefaultSubnetNodeUniqueParamLimit>
  ) -> bool {
    if !SubnetsData::<T>::contains_key(subnet_id) {
      return false
    }

    match SubnetNodeUniqueParam::<T>::try_get(subnet_id, a) {
      Ok(_) => true,
      Err(()) => false,
    }
  }
}