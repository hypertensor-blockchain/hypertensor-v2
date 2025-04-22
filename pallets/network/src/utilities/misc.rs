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
  pub fn validate_peer_id(peer_id: PeerId) -> bool {
    let peer_id_0 = peer_id.0;
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
  
  pub fn get_tx_rate_limit() -> u64 {
    TxRateLimit::<T>::get()
  }

  pub fn set_last_tx_block(key: &T::AccountId, block: u64) {
    LastTxBlock::<T>::insert(key, block)
  }

  pub fn get_last_tx_block(key: &T::AccountId) -> u64 {
    LastTxBlock::<T>::get(key)
  }

  pub fn exceeds_tx_rate_limit(prev_tx_block: u64, current_block: u64) -> bool {
    let rate_limit: u64 = Self::get_tx_rate_limit();
    if rate_limit == 0 || prev_tx_block == 0 {
      return false;
    }

    return current_block - prev_tx_block <= rate_limit;
  }
}