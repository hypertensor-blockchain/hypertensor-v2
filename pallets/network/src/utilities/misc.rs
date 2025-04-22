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
  // Loosely validates Node ID
  pub fn validate_peer_id(peer_id: &PeerId) -> bool {
    let peer_id_0 = &peer_id.0;
    let len = peer_id_0.len();

    // PeerId must be equal to or greater than 32 chars
    // PeerId must be equal to or less than 128 chars
    if len < 32 || len > 128 {
      return false
    };

    let first_char = peer_id_0[0];
    let second_char = peer_id_0[1];
    if first_char == 49 {
      // Node ID (ed25519, using the "identity" multihash) encoded as a raw base58btc multihash
      return len <= 128
    } else if first_char == 81 && second_char == 109 {
      // Node ID (sha256) encoded as a raw base58btc multihash
      return len <= 128;
    } else if first_char == 102 || first_char == 98 || first_char == 122 || first_char == 109 {
      // Node ID (sha256) encoded as a CID
      return len <= 128;
    }
    
    false
  }
  
  pub fn get_tx_rate_limit() -> u32 {
    TxRateLimit::<T>::get()
  }

  pub fn set_last_tx_block(key: &T::AccountId, block: u32) {
    LastTxBlock::<T>::insert(key, block)
  }

  pub fn get_last_tx_block(key: &T::AccountId) -> u32 {
    LastTxBlock::<T>::get(key)
  }

  pub fn exceeds_tx_rate_limit(prev_tx_block: u32, current_block: u32) -> bool {
    let rate_limit: u32 = Self::get_tx_rate_limit();
    if rate_limit == 0 || prev_tx_block == 0 {
      return false;
    }

    return current_block - prev_tx_block <= rate_limit;
  }

  pub fn balance_to_u128(
    input: <<T as pallet::Config>::Currency as frame_support::traits::Currency<<T as frame_system::Config>::AccountId>>::Balance,
  ) -> Option<u128> {
    input.try_into().ok()
  }

  /// Get total tokens in circulation
  pub fn get_total_network_issuance() -> u128 {
    let total_issuance_as_balance = T::Currency::total_issuance();
    let total_issuance: u128 = total_issuance_as_balance.try_into().unwrap_or(0);
    let total_staked: u128 = TotalStake::<T>::get();
    let total_delegate_staked: u128 = TotalDelegateStake::<T>::get();
    let total_node_delegate_staked: u128 = TotalNodeDelegateStake::<T>::get();
    total_issuance
      .saturating_add(total_staked)
      .saturating_add(total_delegate_staked)
      .saturating_add(total_node_delegate_staked)
  }
  
  pub fn send_to_treasury(
    who: &T::AccountId, 
    amount: <<T as pallet::Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance
  ) -> DispatchResult {
    let treasury_account = T::TreasuryAccount::get();

    T::Currency::transfer(
      who,
      &treasury_account,
      amount,
      ExistenceRequirement::KeepAlive,
    )?;

    Ok(())
  }

  pub fn is_paused() -> DispatchResult {
    ensure!(
      !TxPause::<T>::get(),
      Error::<T>::Paused
    );
    Ok(())
  }

  // pub fn calculate_registration_delay(
  //   subnet_id: u32,
  //   base_delay: u32, 
  //   scaling_factor: f64, 
  //   current_nodes: u32
  // ) -> u32 {
  //   let delay = base_delay as f64 / (1.0 + scaling_factor * (1.0 + current_nodes as f64).log2());
  //   delay.round() as u32
  // }
}