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
//
// Enables accounts to delegate stake to subnets for a portion of emissions

use super::*;
use sp_runtime::Saturating;

impl<T: Config> Pallet<T> {
  pub fn do_add_delegate_stake(
    origin: T::RuntimeOrigin,
    subnet_id: u32,
    delegate_stake_to_be_added: u128,
  ) -> DispatchResult {
    let account_id: T::AccountId = ensure_signed(origin)?;

    let (result, balance, shares) = Self::perform_do_add_delegate_stake(
      &account_id,
      subnet_id,
      delegate_stake_to_be_added,
      false
    );

    result?;
    
    let block: u32 = Self::get_current_block_as_u32();

    // Set last block for rate limiting
    Self::set_last_tx_block(&account_id, block);

    Self::deposit_event(Event::SubnetDelegateStakeAdded(subnet_id, account_id, delegate_stake_to_be_added));

    Ok(())
  }

  /// Add to the subnet delegate stake balance of a user
  ///
  /// # Arguments
  ///
  /// * `account_id` - Account adding to balance of subnet.
  /// * `subnet_id` - Subnet ID adding stake to.
  /// * `delegate_stake_to_be_added` - Balance to add or switch.
  /// * `switch` - If we are switching between subnets or nodes.
  ///              - True: Don't remove balance from users account
  ///              - False: Check user balance is withdrawable and withdraw balance
  ///
  pub fn perform_do_add_delegate_stake(
    account_id: &T::AccountId,
    subnet_id: u32,
    delegate_stake_to_be_added: u128,
    switch: bool
  ) -> (DispatchResult, u128, u128) {
    let delegate_stake_as_balance = Self::u128_to_balance(delegate_stake_to_be_added);
    
    if !delegate_stake_as_balance.is_some() {
      return (Err(Error::<T>::CouldNotConvertToBalance.into()), 0, 0);
    }

    if delegate_stake_to_be_added < MinDelegateStakeBalance::<T>::get() {
      return (Err(Error::<T>::CouldNotConvertToBalance.into()), 0, 0);
    }

    // --- Get accounts current balance
    // let account_delegate_stake_balance = Self::convert_to_balance(
    //   account_delegate_stake_shares,
    //   total_subnet_delegated_stake_shares,
    //   total_subnet_delegated_stake_balance
    // );

    // if account_delegate_stake_balance.saturating_add(delegate_stake_to_be_added) > MaxDelegateStakeBalance::<T>::get() {
    //   return (Err(Error::<T>::MaxDelegatedStakeReached.into()), 0, 0);
    // }

    // --- Ensure the callers account_id has enough delegate_stake to perform the transaction.
    if !switch {
      if !Self::can_remove_balance_from_coldkey_account(&account_id, delegate_stake_as_balance.unwrap()) {
        return (Err(Error::<T>::NotEnoughBalanceToStake.into()), 0, 0);
      }  
    }
  
    // to-do: add AddStakeRateLimit instead of universal rate limiter
    //        this allows peers to come in freely
    let block: u32 = Self::get_current_block_as_u32();
    if Self::exceeds_tx_rate_limit(Self::get_last_tx_block(&account_id), block) {
      return (Err(Error::<T>::TxRateLimitExceeded.into()), 0, 0);
    }

    // --- Ensure the remove operation from the account_id is a success.
    if !switch {
      if Self::remove_balance_from_coldkey_account(&account_id, delegate_stake_as_balance.unwrap()) == false {
        return (Err(Error::<T>::BalanceWithdrawalError.into()), 0, 0);
      }  
    }
    
    let total_subnet_delegated_stake_shares = match TotalSubnetDelegateStakeShares::<T>::get(subnet_id) {
      0 => {
         // --- Mitigate inflation attack
        TotalSubnetDelegateStakeShares::<T>::mutate(subnet_id, |mut n| n.saturating_accrue(1000));
        0
      },
      shares => shares,
    };
    let total_subnet_delegated_stake_balance = TotalSubnetDelegateStakeBalance::<T>::get(subnet_id);

    // --- Get amount to be added as shares based on stake to balance added to account
    let mut delegate_stake_to_be_added_as_shares = Self::convert_to_shares(
      delegate_stake_to_be_added,
      total_subnet_delegated_stake_shares,
      total_subnet_delegated_stake_balance
    );

    // --- Check rounding errors
    if delegate_stake_to_be_added_as_shares == 0 {
      return (Err(Error::<T>::CouldNotConvertToShares.into()), 0, 0);
    }

    Self::increase_account_delegate_stake_shares(
      &account_id,
      subnet_id, 
      delegate_stake_to_be_added,
      delegate_stake_to_be_added_as_shares,
    );

    (Ok(()), delegate_stake_to_be_added, delegate_stake_to_be_added_as_shares)
  }

  pub fn do_remove_delegate_stake(
    origin: T::RuntimeOrigin, 
    subnet_id: u32,
    delegate_stake_shares_to_be_removed: u128,
  ) -> DispatchResult {
    let account_id: T::AccountId = ensure_signed(origin)?;

    let (result, delegate_stake_to_be_removed, _) = Self::perform_do_remove_delegate_stake(
      &account_id, 
      subnet_id,
      delegate_stake_shares_to_be_removed,
      true
    );

    result?;

    let block: u32 = Self::get_current_block_as_u32();

    // Set last block for rate limiting
    Self::set_last_tx_block(&account_id, block);

    Self::deposit_event(Event::SubnetDelegateStakeRemoved(subnet_id, account_id, delegate_stake_to_be_removed));

    Ok(())
  }

  /// Remove the subnet delegate stake balance of a user
  ///
  /// # Arguments
  ///
  /// * `account_id` - Account removing balance from subnet.
  /// * `subnet_id` - Subnet ID removing stake from.
  /// * `delegate_stake_shares_to_be_removed` - Shares of pool to remove.
  /// * `add_to_ledger` - If we are unstaking from network and not switching between staking options.
  ///              - True: Unstake user to unstaking ledger.
  ///              - False: Don't add balance to unstaking ledger.
  ///
  pub fn perform_do_remove_delegate_stake(
    account_id: &T::AccountId, 
    subnet_id: u32,
    delegate_stake_shares_to_be_removed: u128,
    add_to_ledger: bool
  ) -> (DispatchResult, u128, u128) {
    // --- Ensure that the delegate_stake amount to be removed is above zero.
    if delegate_stake_shares_to_be_removed == 0 {
      return (Err(Error::<T>::NotEnoughStakeToWithdraw.into()), 0, 0);
    }

    let account_delegate_stake_shares: u128 = AccountSubnetDelegateStakeShares::<T>::get(&account_id, subnet_id);

    // --- Ensure that the account has enough delegate_stake to withdraw.      
    if account_delegate_stake_shares < delegate_stake_shares_to_be_removed {
      return (Err(Error::<T>::NotEnoughStakeToWithdraw.into()), 0, 0);
    }

    let total_subnet_delegated_stake_shares = TotalSubnetDelegateStakeShares::<T>::get(subnet_id);
    let total_subnet_delegated_stake_balance = TotalSubnetDelegateStakeBalance::<T>::get(subnet_id);

    // --- Get accounts current balance
    let delegate_stake_to_be_removed = Self::convert_to_balance(
      delegate_stake_shares_to_be_removed,
      total_subnet_delegated_stake_shares,
      total_subnet_delegated_stake_balance
    );

    // --- Ensure that we can convert this u128 to a balance.
    // Redunant
    let delegate_stake_to_be_added_as_currency = Self::u128_to_balance(delegate_stake_to_be_removed);
    if !delegate_stake_to_be_added_as_currency.is_some() {
      return (Err(Error::<T>::CouldNotConvertToBalance.into()), 0, 0);
    }

    let block: u32 = Self::get_current_block_as_u32();
    if Self::exceeds_tx_rate_limit(Self::get_last_tx_block(&account_id), block) {
      return (Err(Error::<T>::TxRateLimitExceeded.into()), 0, 0);
    }

    // --- We remove the shares from the account and balance from the pool
    Self::decrease_account_delegate_stake_shares(&account_id, subnet_id, delegate_stake_to_be_removed, delegate_stake_shares_to_be_removed);
    
    // --- We add the balancer to the account_id.  If the above fails we will not credit this account_id.
    if add_to_ledger {
      let result = Self::add_balance_to_unbonding_ledger(
        &account_id, 
        delegate_stake_to_be_removed, 
        T::DelegateStakeCooldownEpochs::get(),
        block
      );
      
      if let Err(e) = result {
        return (Err(e), 0, 0);
      }  
    }

    (Ok(()), delegate_stake_to_be_removed, delegate_stake_shares_to_be_removed)
  }

  pub fn do_switch_delegate_stake(
    origin: T::RuntimeOrigin, 
    from_subnet_id: u32,
    to_subnet_id: u32,
    delegate_stake_shares_to_be_switched: u128,
  ) -> DispatchResult {
    let account_id: T::AccountId = ensure_signed(origin)?;

    let (result, delegate_stake_to_be_removed, _) = Self::perform_do_remove_delegate_stake(
      &account_id, 
      from_subnet_id,
      delegate_stake_shares_to_be_switched,
      false,
    );

    result?;

    // --- Add
    let (result, balance, shares) = Self::perform_do_add_delegate_stake(
      &account_id,
      to_subnet_id,
      delegate_stake_to_be_removed,
      true
    );

    result?;

    let block: u32 = Self::get_current_block_as_u32();

    // Set last block for rate limiting
    Self::set_last_tx_block(&account_id, block);

    Self::deposit_event(Event::SubnetDelegateStakeSwitched(
      from_subnet_id, 
      to_subnet_id, 
      account_id, 
      delegate_stake_to_be_removed
    ));

    Ok(())
  }

  pub fn increase_account_delegate_stake_shares(
    account_id: &T::AccountId,
    subnet_id: u32, 
    amount: u128,
    shares: u128,
  ) {
    // -- increase account subnet staking shares balance
    AccountSubnetDelegateStakeShares::<T>::mutate(account_id, subnet_id, |mut n| n.saturating_accrue(shares));

    // -- increase total subnet delegate stake balance
    TotalSubnetDelegateStakeBalance::<T>::mutate(subnet_id, |mut n| n.saturating_accrue(amount));

    // -- increase total subnet delegate stake shares
    TotalSubnetDelegateStakeShares::<T>::mutate(subnet_id, |mut n| n.saturating_accrue(shares));

    TotalDelegateStake::<T>::mutate(|mut n| n.saturating_accrue(amount));
  }
  
  pub fn decrease_account_delegate_stake_shares(
    account_id: &T::AccountId,
    subnet_id: u32, 
    amount: u128,
    shares: u128,
  ) {
    // -- decrease account subnet staking shares balance
    AccountSubnetDelegateStakeShares::<T>::mutate(account_id, subnet_id, |mut n| n.saturating_reduce(shares));

    // -- decrease total subnet delegate stake balance
    TotalSubnetDelegateStakeBalance::<T>::mutate(subnet_id, |mut n| n.saturating_reduce(amount));

    // -- decrease total subnet delegate stake shares
    TotalSubnetDelegateStakeShares::<T>::mutate(subnet_id, |mut n| n.saturating_reduce(shares));

    TotalDelegateStake::<T>::mutate(|mut n| n.saturating_reduce(amount));
  }

  /// Rewards are deposited here from the ``rewards.rs`` or by donations
  pub fn do_increase_delegate_stake(
    subnet_id: u32,
    amount: u128,
  ) {
    if TotalSubnetDelegateStakeBalance::<T>::get(subnet_id) == 0 || 
      TotalSubnetDelegateStakeShares::<T>::get(subnet_id) == 0 
    {
      TotalSubnetDelegateStakeShares::<T>::mutate(subnet_id, |mut n| n.saturating_accrue(1000));
    };

    // -- increase total subnet delegate stake 
    TotalSubnetDelegateStakeBalance::<T>::mutate(subnet_id, |mut n| n.saturating_accrue(amount));

    TotalDelegateStake::<T>::mutate(|mut n| n.saturating_accrue(amount));
  }

  pub fn convert_account_shares_to_balance(
    account_id: &T::AccountId,
    subnet_id: u32
  ) -> u128 {
    let account_delegate_stake_shares: u128 = AccountSubnetDelegateStakeShares::<T>::get(&account_id, subnet_id);
    if account_delegate_stake_shares == 0 {
      return 0;
    }
    let total_subnet_delegated_stake_shares = TotalSubnetDelegateStakeShares::<T>::get(subnet_id);
    let total_subnet_delegated_stake_balance = TotalSubnetDelegateStakeBalance::<T>::get(subnet_id);

    // --- Get accounts current balance
    Self::convert_to_balance(
      account_delegate_stake_shares,
      total_subnet_delegated_stake_shares,
      total_subnet_delegated_stake_balance
    )
  }
}