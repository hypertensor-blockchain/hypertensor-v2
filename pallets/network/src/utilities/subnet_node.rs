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
  /// Remove subnet peer from subnet
  // to-do: Add slashing to subnet peers stake balance
  pub fn perform_remove_subnet_node(block: u32, subnet_id: u32, subnet_node_id: u32) {
    if let Ok(subnet_node) = SubnetNodesData::<T>::try_get(subnet_id, subnet_node_id) {
      let hotkey = subnet_node.hotkey;
      let peer_id = subnet_node.peer_id;

      // Remove from attestations
      let epoch_length: u32 = T::EpochLength::get();
			let epoch: u32 = block / epoch_length;

      let submittable_nodes: BTreeSet<T::AccountId> = Self::get_classified_hotkeys(subnet_id, &SubnetNodeClass::Validator, epoch);

      SubnetRewardsSubmission::<T>::try_mutate_exists(
        subnet_id,
        epoch as u32,
        |params| -> DispatchResult {
          let params = if let Some(params) = params {
            // --- Remove from consensus
            let mut data = &mut params.data;
            data.retain(|x| x.peer_id != peer_id);
            params.data = data.clone();
            
            // --- Remove from attestations
            let mut attests = &mut params.attests;
            if attests.remove(&subnet_node_id).is_some() {
              params.attests = attests.clone();
            }
          };
          Ok(())
        }
      );
    
      let subnet_node = SubnetNodesData::<T>::take(subnet_id, subnet_node_id);

      if subnet_node.a.is_some() {
        SubnetNodeUniqueParam::<T>::remove(subnet_id, subnet_node.a.unwrap())
      }

      // Remove all subnet node elements
      PeerIdSubnetNode::<T>::remove(subnet_id, &peer_id);
      BootstrapPeerIdSubnetNode::<T>::remove(subnet_id, subnet_node.bootstrap_peer_id);
      HotkeySubnetNodeId::<T>::remove(subnet_id, &hotkey);
      SubnetNodeIdHotkey::<T>::remove(subnet_id, subnet_node_id);

      // Update total subnet peers by substracting 1
      TotalSubnetNodes::<T>::mutate(subnet_id, |n: &mut u32| n.saturating_dec());

      // Reset sequential absent subnet node count
      SubnetNodePenalties::<T>::remove(subnet_id, subnet_node_id);

      TotalActiveNodes::<T>::mutate(|n: &mut u32| n.saturating_dec());

			Self::deposit_event(Event::SubnetNodeRemoved { subnet_id: subnet_id, subnet_node_id: subnet_node_id });
    }
  }

  pub fn get_classified_subnet_node_ids<C>(
    subnet_id: u32,
    classification: &SubnetNodeClass,
    epoch: u32,
  ) -> C
    where
      C: FromIterator<u32>,
  {
    SubnetNodesData::<T>::iter_prefix(subnet_id)
      .filter(|(_, subnet_node)| subnet_node.has_classification(classification, epoch))
      .map(|(subnet_node_id, _)| subnet_node_id)
      .collect()
  }
  
  /// Get subnet nodes by classification
  pub fn get_classified_subnet_nodes(subnet_id: u32, classification: &SubnetNodeClass, epoch: u32) -> Vec<SubnetNode<T::AccountId>> {
    SubnetNodesData::<T>::iter_prefix_values(subnet_id)
      .filter(|subnet_node| subnet_node.has_classification(classification, epoch))
      .collect()
  }

  pub fn get_classified_subnet_node_info(subnet_id: u32, classification: &SubnetNodeClass, epoch: u32) -> Vec<SubnetNodeInfo<T::AccountId>> {
    SubnetNodesData::<T>::iter_prefix(subnet_id)
      .filter(|(subnet_node_id, subnet_node)| subnet_node.has_classification(classification, epoch))
      .map(|(subnet_node_id, subnet_node)| {
        SubnetNodeInfo {
          subnet_node_id: subnet_node_id,
          coldkey: HotkeyOwner::<T>::get(&subnet_node.hotkey),
          hotkey: subnet_node.hotkey,
          peer_id: subnet_node.peer_id,
          classification: subnet_node.classification,
          a: subnet_node.a,
          b: subnet_node.b,
          c: subnet_node.c,
        }
      })
      .collect()
  }

  // Get subnet node ``hotkeys`` by classification
  pub fn get_classified_hotkeys<C>(
    subnet_id: u32,
    classification: &SubnetNodeClass,
    epoch: u32,
  ) -> C
    where
      C: FromIterator<T::AccountId>,
  {
    SubnetNodesData::<T>::iter_prefix(subnet_id)
      .filter(|(_, subnet_node)| subnet_node.has_classification(classification, epoch))
      .map(|(_, subnet_node)| subnet_node.hotkey)
      .collect()
  }

  pub fn is_subnet_node_owner(subnet_id: u32, subnet_node_id: u32, hotkey: T::AccountId) -> bool {
    match SubnetNodesData::<T>::try_get(subnet_id, subnet_node_id) {
      Ok(data) => {
        data.hotkey == hotkey
      },
      Err(()) => false,
    }
  }

  /// Is hotkey or coldkey owner for functions that allow both
  pub fn get_hotkey_coldkey(
    subnet_id: u32, 
    subnet_node_id: u32, 
  ) -> Option<(T::AccountId, T::AccountId)> {
    let hotkey = SubnetNodeIdHotkey::<T>::try_get(subnet_id, subnet_node_id).ok()?;
    let coldkey = HotkeyOwner::<T>::try_get(&hotkey).ok()?;

    Some((hotkey, coldkey))
  }

  pub fn is_keys_owner(
    subnet_id: u32, 
    subnet_node_id: u32, 
    key: T::AccountId, 
  ) -> bool {
    let (hotkey, coldkey) = match Self::get_hotkey_coldkey(subnet_id, subnet_node_id) {
      Some((hotkey, coldkey)) => {
        (hotkey, coldkey)
      }
      None => {
        return false
      }
    };

    key == hotkey || key == coldkey
  }

  pub fn is_subnet_node_coldkey(
    subnet_id: u32, 
    subnet_node_id: u32, 
    coldkey: T::AccountId, 
  ) -> bool {
    let hotkey = match SubnetNodeIdHotkey::<T>::try_get(subnet_id, subnet_node_id) {
      Ok(hotkey) => hotkey,
      Err(()) => return false
    };
    match HotkeyOwner::<T>::try_get(hotkey) {
      Ok(subnet_node_coldkey) => return subnet_node_coldkey == coldkey,
      Err(()) => return false
    }
  }

  pub fn increase_class(
    subnet_id: u32, 
    subnet_node_id: u32, 
    start_epoch: u32,
  ) {
    // TODO: Add querying epoch here
    SubnetNodesData::<T>::mutate(
      subnet_id,
      subnet_node_id,
      |params: &mut SubnetNode<T::AccountId>| {
        params.classification = SubnetNodeClassification {
          class: params.classification.class.next(),
          start_epoch: start_epoch,
        };
      },
    );
  }

  pub fn is_owner_of_peer_or_ownerless(subnet_id: u32, subnet_node_id: u32, peer_id: &PeerId) -> bool {
    let is_peer_owner_or_ownerless = match PeerIdSubnetNode::<T>::try_get(subnet_id, peer_id) {
      Ok(peer_subnet_node_id) => {
        if peer_subnet_node_id == subnet_node_id {
          return true
        }
        false
      },
      Err(()) => true,
    };

    is_peer_owner_or_ownerless && match BootstrapPeerIdSubnetNode::<T>::try_get(subnet_id, peer_id) {
      Ok(bootstrap_subnet_node_id) => {
        if bootstrap_subnet_node_id == subnet_node_id {
          return true
        }
        false
      },
      Err(()) => true,
    }
  }

  pub fn calculate_max_activation_epoch(subnet_id: u32) -> u32 {
    let prev_registration_epoch = 10;
    0
  }

  pub fn get_subnet_churn_limit(subnet_id: u32) -> u32 {
    let min_churn = 4;
    let active_nodes = TotalActiveSubnetNodes::<T>::get(subnet_id);
    let churn_denominator = ChurnDenominator::<T>::get(subnet_id);
    min_churn.max(active_nodes.saturating_div(churn_denominator))
  }
}