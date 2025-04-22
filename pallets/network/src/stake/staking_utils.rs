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
  pub fn add_balance_to_unbonding_ledger(
    coldkey: &T::AccountId,
    amount: u128,
    cooldown_epoch_length: u64,
    block: u64,
  ) -> DispatchResult {
    let epoch_length: u64 = T::EpochLength::get();
    let epoch: u64 = block / epoch_length;

    let unbondings = StakeUnbondingLedger::<T>::get(coldkey.clone());

    // --- Ensure we don't surpass max unlockings by attempting to unlock unbondings
    if unbondings.len() as u32 == T::MaxStakeUnlockings::get() {
      Self::do_claim_unbondings(&coldkey);
    }

    // --- Get updated unbondings after claiming unbondings
    let mut unbondings = StakeUnbondingLedger::<T>::get(coldkey.clone());

    // We're about to add another unbonding to the ledger - it must be n-1
    ensure!(
      unbondings.len() < T::MaxStakeUnlockings::get() as usize,
      Error::<T>::MaxUnlockingsReached
    );

    StakeUnbondingLedger::<T>::mutate(&coldkey, |ledger| {
      ledger.entry(epoch).and_modify(|v| *v += amount).or_insert(amount);
    });

    Ok(())
  }

  // Infallible
  pub fn do_claim_unbondings(coldkey: &T::AccountId) -> u32 {
    let block: u64 = Self::get_current_block_as_u64();
    let epoch_length: u64 = T::EpochLength::get();
    let epoch: u64 = block / epoch_length;
    let unbondings = StakeUnbondingLedger::<T>::get(coldkey.clone());

    let mut unbondings_copy = unbondings.clone();

    let mut successful_unbondings = 0;

    for (unbonding_epoch, amount) in unbondings.iter() {
      if epoch <= unbonding_epoch + T::StakeCooldownEpochs::get() {
        continue
      }

      let stake_to_be_added_as_currency = Self::u128_to_balance(*amount);
      if !stake_to_be_added_as_currency.is_some() {
        // Redundant
        unbondings_copy.remove(&unbonding_epoch);
        continue
      }
      
      unbondings_copy.remove(&unbonding_epoch);
      Self::add_balance_to_coldkey_account(&coldkey, stake_to_be_added_as_currency.unwrap());
      successful_unbondings += 1;
    }

    if unbondings.len() != unbondings_copy.len() {
      StakeUnbondingLedger::<T>::insert(coldkey.clone(), unbondings_copy);
    }
    successful_unbondings
  }

  pub fn can_remove_balance_from_coldkey_account(
    coldkey: &T::AccountId,
    amount: <<T as pallet::Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance,
  ) -> bool {
    let current_balance = Self::get_coldkey_balance(coldkey);
    if amount > current_balance {
      return false;
    }

    // This bit is currently untested. @todo
    let new_potential_balance = current_balance - amount;
    let can_withdraw = T::Currency::ensure_can_withdraw(
      &coldkey,
      amount,
      WithdrawReasons::except(WithdrawReasons::TIP),
      new_potential_balance,
    )
    .is_ok();
    can_withdraw
  }

  pub fn remove_balance_from_coldkey_account(
    coldkey: &T::AccountId,
    amount: <<T as pallet::Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance,
  ) -> bool {
    return match T::Currency::withdraw(
      &coldkey,
      amount,
      WithdrawReasons::except(WithdrawReasons::TIP),
      ExistenceRequirement::KeepAlive,
    ) {
      Ok(_result) => true,
      Err(_error) => false,
    };
  }

  pub fn add_balance_to_coldkey_account(
    coldkey: &T::AccountId,
    amount: <<T as pallet::Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance,
  ) {
    T::Currency::deposit_creating(&coldkey, amount);
  }

  pub fn get_coldkey_balance(
    coldkey: &T::AccountId,
  ) -> <<T as pallet::Config>::Currency as Currency<<T as system::Config>::AccountId>>::Balance {
    return T::Currency::free_balance(&coldkey);
  }

  pub fn u128_to_balance(
    input: u128,
  ) -> Option<
    <<T as pallet::Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance,
  > {
    input.try_into().ok()
  }
}