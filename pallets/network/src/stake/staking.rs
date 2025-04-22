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
use sp_runtime::Saturating;

impl<T: Config> Pallet<T> {
  pub fn do_add_stake(
    origin: T::RuntimeOrigin,
    subnet_id: u32,
    hotkey: T::AccountId,
    stake_to_be_added: u128,
  ) -> DispatchResult {
    let coldkey: T::AccountId = ensure_signed(origin)?;

    let stake_as_balance = Self::u128_to_balance(stake_to_be_added);

    ensure!(
      stake_as_balance.is_some(),
      Error::<T>::CouldNotConvertToBalance
    );

    let account_stake_balance: u128 = AccountSubnetStake::<T>::get(&hotkey, subnet_id);

    ensure!(
      account_stake_balance.saturating_add(stake_to_be_added) >= MinStakeBalance::<T>::get(),
      Error::<T>::MinStakeNotReached
    );

    ensure!(
      account_stake_balance.saturating_add(stake_to_be_added) <= MaxStakeBalance::<T>::get(),
      Error::<T>::MaxStakeReached
    );

    // --- Ensure the callers coldkey has enough stake to perform the transaction.
    ensure!(
      Self::can_remove_balance_from_coldkey_account(&coldkey, stake_as_balance.unwrap()),
      Error::<T>::NotEnoughBalanceToStake
    );
  
    // to-do: add AddStakeRateLimit instead of universal rate limiter
    //        this allows peers to come in freely
    let block: u32 = Self::get_current_block_as_u32();
    ensure!(
      !Self::exceeds_tx_rate_limit(Self::get_last_tx_block(&coldkey), block),
      Error::<T>::TxRateLimitExceeded
    );

    // --- Ensure the remove operation from the coldkey is a success.
    ensure!(
      Self::remove_balance_from_coldkey_account(&coldkey, stake_as_balance.unwrap()) == true,
      Error::<T>::BalanceWithdrawalError
    );
  
    Self::increase_account_stake(
      &hotkey,
      subnet_id, 
      stake_to_be_added,
    );

    // Set last block for rate limiting
    Self::set_last_tx_block(&coldkey, block);

    // Self::deposit_event(Event::StakeAdded(subnet_id, coldkey, stake_to_be_added));
    Self::deposit_event(Event::StakeAdded(subnet_id, coldkey, hotkey, stake_to_be_added));

    Ok(())
  }

  pub fn do_remove_stake(
    origin: T::RuntimeOrigin, 
    subnet_id: u32,
    hotkey: T::AccountId,
    is_subnet_node: bool,
    is_active: bool,
    stake_to_be_removed: u128,
  ) -> DispatchResult {
    let coldkey: T::AccountId = ensure_signed(origin)?;

    // --- Ensure that the stake amount to be removed is above zero.
    ensure!(
      stake_to_be_removed > 0,
      Error::<T>::NotEnoughStakeToWithdraw
    );

    let account_stake_balance: u128 = AccountSubnetStake::<T>::get(&hotkey, subnet_id);

    // --- Ensure that the account has enough stake to withdraw.
    ensure!(
      account_stake_balance >= stake_to_be_removed,
      Error::<T>::NotEnoughStakeToWithdraw
    );
    
    // if user is still a subnet node they must keep the required minimum balance
    if is_subnet_node {
      ensure!(
        account_stake_balance.saturating_sub(stake_to_be_removed) >= MinStakeBalance::<T>::get(),
        Error::<T>::MinStakeNotReached
      );  
    }
  
    // --- Ensure that we can convert this u128 to a balance.
    let stake_to_be_removed_as_currency = Self::u128_to_balance(stake_to_be_removed);
    ensure!(
      stake_to_be_removed_as_currency.is_some(),
        Error::<T>::CouldNotConvertToBalance
    );

    let block: u32 = Self::get_current_block_as_u32();
    ensure!(
      !Self::exceeds_tx_rate_limit(Self::get_last_tx_block(&coldkey), block),
      Error::<T>::TxRateLimitExceeded
    );

    // --- 7. We remove the balance from the hotkey.
    Self::decrease_account_stake(&hotkey, subnet_id, stake_to_be_removed);

    // --- 9. We add the balancer to the coldkey.  If the above fails we will not credit this coldkey.
    if is_active {
      Self::add_balance_to_unbonding_ledger(
        &coldkey, 
        stake_to_be_removed, 
        T::StakeCooldownEpochs::get(),
        block
      ).map_err(|e| e)?;
    } else {
      // Unstaking cooldown for nodes that never activated
      Self::add_balance_to_unbonding_ledger(
        &coldkey, 
        stake_to_be_removed, 
        RegisteredStakeCooldownEpochs::<T>::get(),
        block
      ).map_err(|e| e)?;  
    }

    // Set last block for rate limiting
    Self::set_last_tx_block(&coldkey, block);

    Self::deposit_event(Event::StakeRemoved(subnet_id, coldkey, hotkey, stake_to_be_removed));

    Ok(())
  }

  pub fn do_swap_hotkey_balance(
    origin: T::RuntimeOrigin, 
    subnet_id: u32,
    old_hotkey: &T::AccountId,
    new_hotkey: &T::AccountId,
  ) {
    Self::swap_account_stake(
      old_hotkey,
      new_hotkey,
      subnet_id, 
    )
  }

  pub fn increase_account_stake(
    hotkey: &T::AccountId,
    subnet_id: u32, 
    amount: u128,
  ) {
    // -- increase account subnet staking balance
    AccountSubnetStake::<T>::mutate(hotkey, subnet_id, |mut n| n.saturating_accrue(amount));

    // -- increase total subnet stake
    TotalSubnetStake::<T>::mutate(subnet_id, |mut n| n.saturating_accrue(amount));

    // -- increase total stake overall
    TotalStake::<T>::mutate(|mut n| n.saturating_accrue(amount));
  }
  
  pub fn decrease_account_stake(
    hotkey: &T::AccountId,
    subnet_id: u32, 
    amount: u128,
  ) {
    // -- decrease account subnet staking balance
    AccountSubnetStake::<T>::mutate(hotkey, subnet_id, |mut n| n.saturating_reduce(amount));

    // -- decrease total subnet stake
    TotalSubnetStake::<T>::mutate(subnet_id, |mut n| n.saturating_reduce(amount));

    // -- decrease total stake overall
    TotalStake::<T>::mutate(|mut n| n.saturating_reduce(amount));
  }

  fn swap_account_stake(
    old_hotkey: &T::AccountId,
    new_hotkey: &T::AccountId,
    subnet_id: u32, 
  ) {
    // -- swap old_hotkey subnet staking balance
    let old_hotkey_stake_balance = AccountSubnetStake::<T>::take(old_hotkey, subnet_id);
    // --- Redundant take of new hotkeys stake balance
    // --- New hotkey is always checked before updating
    let new_hotkey_stake_balance = AccountSubnetStake::<T>::take(new_hotkey, subnet_id);
    AccountSubnetStake::<T>::insert(
      new_hotkey, 
      subnet_id, 
      old_hotkey_stake_balance.saturating_add(new_hotkey_stake_balance)
    );
  }
}