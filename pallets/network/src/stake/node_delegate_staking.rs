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
  pub fn do_add_node_delegate_stake(
    origin: T::RuntimeOrigin,
    subnet_id: u32,
    subnet_node_id: u32,
    node_delegate_stake_to_be_added: u128,
  ) -> DispatchResult {
    let account_id: T::AccountId = ensure_signed(origin)?;

    let delegate_stake_as_balance = Self::u128_to_balance(node_delegate_stake_to_be_added);

    ensure!(
      delegate_stake_as_balance.is_some(),
      Error::<T>::CouldNotConvertToBalance
    );
    let account_node_delegate_stake_shares = AccountNodeDelegateStakeShares::<T>::get((&account_id, subnet_id, subnet_node_id));
    let total_node_delegated_stake_shares = TotalNodeDelegateStakeShares::<T>::get(subnet_id, subnet_node_id);
    let total_node_delegated_stake_balance = TotalNodeDelegateStakeBalance::<T>::get(subnet_id, subnet_node_id);

    // --- Get accounts current balance
    let account_delegate_stake_balance = Self::convert_to_balance(
      account_node_delegate_stake_shares,
      total_node_delegated_stake_shares,
      total_node_delegated_stake_balance
    );

    ensure!(
      account_delegate_stake_balance.saturating_add(node_delegate_stake_to_be_added) <= MaxDelegateStakeBalance::<T>::get(),
      Error::<T>::MaxDelegatedStakeReached
    );

    // --- Ensure the callers account_id has enough delegate_stake to perform the transaction.
    ensure!(
      Self::can_remove_balance_from_coldkey_account(&account_id, delegate_stake_as_balance.unwrap()),
      Error::<T>::NotEnoughBalanceToStake
    );
  
    // to-do: add AddStakeRateLimit instead of universal rate limiter
    //        this allows peers to come in freely
    let block: u64 = Self::get_current_block_as_u64();
    ensure!(
      !Self::exceeds_tx_rate_limit(Self::get_last_tx_block(&account_id), block),
      Error::<T>::TxRateLimitExceeded
    );

    // --- Ensure the remove operation from the account_id is a success.
    ensure!(
      Self::remove_balance_from_coldkey_account(&account_id, delegate_stake_as_balance.unwrap()) == true,
      Error::<T>::BalanceWithdrawalError
    );
  
    // --- Get amount to be added as shares based on stake to balance added to account
    let mut delegate_stake_to_be_added_as_shares = Self::convert_to_shares(
      node_delegate_stake_to_be_added,
      total_node_delegated_stake_shares,
      total_node_delegated_stake_balance
    );

    // --- Mitigate inflation attack
    if total_node_delegated_stake_shares == 0 {
      // no need for saturation here
      TotalNodeDelegateStakeShares::<T>::mutate(subnet_id, subnet_node_id, |mut n| *n += 1000);
      delegate_stake_to_be_added_as_shares = delegate_stake_to_be_added_as_shares.saturating_sub(1000);
    }
    
    // --- Check rounding errors
    ensure!(
      delegate_stake_to_be_added_as_shares != 0,
      Error::<T>::CouldNotConvertToShares
    );

    Self::increase_account_node_delegate_stake_shares(
      &account_id,
      subnet_id, 
      subnet_node_id,
      node_delegate_stake_to_be_added,
      delegate_stake_to_be_added_as_shares,
    );

    // Set last block for rate limiting
    Self::set_last_tx_block(&account_id, block);

    Self::deposit_event(Event::DelegateNodeStakeAdded { 
      account_id: account_id, 
      subnet_id: subnet_id, 
      subnet_node_id: subnet_node_id, 
      amount: node_delegate_stake_to_be_added
    });

    Ok(())
  }

  pub fn do_remove_node_delegate_stake(
    origin: T::RuntimeOrigin, 
    subnet_id: u32,
    subnet_node_id: u32,
    node_delegate_stake_shares_to_be_removed: u128,
  ) -> DispatchResult {
    let account_id: T::AccountId = ensure_signed(origin)?;

    // --- Ensure that the delegate_stake amount to be removed is above zero.
    ensure!(
      node_delegate_stake_shares_to_be_removed > 0,
      Error::<T>::NotEnoughStakeToWithdraw
    );

    let account_node_delegate_stake_shares = AccountNodeDelegateStakeShares::<T>::get((&account_id, subnet_id, subnet_node_id));
    
    // --- Ensure that the account has enough delegate_stake to withdraw.
    ensure!(
      account_node_delegate_stake_shares >= node_delegate_stake_shares_to_be_removed,
      Error::<T>::NotEnoughStakeToWithdraw
    );
      
    let total_node_delegated_stake_shares = TotalNodeDelegateStakeShares::<T>::get(subnet_id, subnet_node_id);
    let total_node_delegated_stake_balance = TotalNodeDelegateStakeBalance::<T>::get(subnet_id, subnet_node_id);

    // --- Get accounts current balance
    let delegate_stake_to_be_removed = Self::convert_to_balance(
      node_delegate_stake_shares_to_be_removed,
      total_node_delegated_stake_shares,
      total_node_delegated_stake_balance
    );

    // --- Ensure that we can convert this u128 to a balance.
    // Redunant
    let delegate_stake_to_be_added_as_currency = Self::u128_to_balance(delegate_stake_to_be_removed);
    ensure!(
      delegate_stake_to_be_added_as_currency.is_some(),
      Error::<T>::CouldNotConvertToBalance
    );

    let block: u64 = Self::get_current_block_as_u64();
    ensure!(
      !Self::exceeds_tx_rate_limit(Self::get_last_tx_block(&account_id), block),
      Error::<T>::TxRateLimitExceeded
    );

    // --- We remove the shares from the account and balance from the pool
    Self::decrease_account_node_delegate_stake_shares(
      &account_id, 
      subnet_id, 
      subnet_node_id, 
      delegate_stake_to_be_removed, 
      node_delegate_stake_shares_to_be_removed
    );
    
    // --- We add the balancer to the account_id.  If the above fails we will not credit this account_id.
    Self::add_balance_to_unbonding_ledger(
      &account_id, 
      delegate_stake_to_be_removed, 
      T::DelegateStakeCooldownEpochs::get(),
      block
    ).map_err(|e| e)?;

    // Set last block for rate limiting
    Self::set_last_tx_block(&account_id, block);

    Self::deposit_event(Event::DelegateNodeStakeRemoved { 
      account_id: account_id, 
      subnet_id: subnet_id, 
      subnet_node_id: subnet_node_id, 
      amount: delegate_stake_to_be_removed
    });

    Ok(())
  }

  pub fn do_switch_node_delegate_stake(
    origin: T::RuntimeOrigin, 
    from_subnet_id: u32,
    from_subnet_node_id: u32,
    to_subnet_id: u32,
    to_subnet_node_id: u32,
    node_delegate_stake_shares_to_be_switched: u128,
  ) -> DispatchResult {
    let account_id: T::AccountId = ensure_signed(origin)?;

    // --- Ensure that the delegate_stake amount to be removed is above zero.
    ensure!(
      node_delegate_stake_shares_to_be_switched > 0,
      Error::<T>::NotEnoughStakeToWithdraw
    );
    let from_account_delegate_stake_shares: u128 = AccountNodeDelegateStakeShares::<T>::get((&account_id.clone(), from_subnet_id, from_subnet_node_id.clone()));

    // --- Ensure that the account has enough delegate_stake to withdraw.
    ensure!(
      from_account_delegate_stake_shares >= node_delegate_stake_shares_to_be_switched,
      Error::<T>::NotEnoughStakeToWithdraw
    );
    
    let block: u64 = Self::get_current_block_as_u64();

    // --- Logic
    ensure!(
      block - LastDelegateStakeTransfer::<T>::get(account_id.clone()) > DelegateStakeTransferPeriod::<T>::get(),
      Error::<T>::DelegateStakeTransferPeriodExceeded
    );

    LastDelegateStakeTransfer::<T>::insert(account_id.clone(), block);

    let total_from_subnet_delegated_stake_shares = TotalNodeDelegateStakeShares::<T>::get(from_subnet_id, from_subnet_node_id);
    let total_from_subnet_delegated_stake_balance = TotalNodeDelegateStakeBalance::<T>::get(from_subnet_id, from_subnet_node_id);

    // --- Get accounts current balance
    let delegate_stake_to_be_transferred = Self::convert_to_balance(
      from_account_delegate_stake_shares,
      total_from_subnet_delegated_stake_shares,
      total_from_subnet_delegated_stake_balance
    );

    // --- Ensure that we can convert this u128 to a balance.
    // Redunant
    let delegate_stake_to_be_transferred_as_currency = Self::u128_to_balance(delegate_stake_to_be_transferred);
    ensure!(
      delegate_stake_to_be_transferred_as_currency.is_some(),
      Error::<T>::CouldNotConvertToBalance
    );

    // --- We remove the shares from the account and balance from the pool
    Self::decrease_account_node_delegate_stake_shares(
      &account_id, 
      from_subnet_id, 
      from_subnet_node_id.clone(),
      delegate_stake_to_be_transferred, 
      node_delegate_stake_shares_to_be_switched
    );





    // --- Add
    let to_account_delegate_stake_shares: u128 = AccountNodeDelegateStakeShares::<T>::get((&account_id.clone(), to_subnet_id, to_subnet_node_id.clone()));
    let total_to_subnet_delegated_stake_shares = TotalNodeDelegateStakeShares::<T>::get(to_subnet_id, to_subnet_node_id);
    let total_to_subnet_delegated_stake_balance = TotalNodeDelegateStakeBalance::<T>::get(to_subnet_id, to_subnet_node_id);

    // --- Get accounts current balance
    let to_account_delegate_stake_balance = Self::convert_to_balance(
      to_account_delegate_stake_shares,
      total_to_subnet_delegated_stake_shares,
      total_to_subnet_delegated_stake_balance
    );

    ensure!(
      to_account_delegate_stake_balance.saturating_add(delegate_stake_to_be_transferred) <= MaxDelegateStakeBalance::<T>::get(),
      Error::<T>::MaxDelegatedStakeReached
    );
  
    // to-do: add AddStakeRateLimit instead of universal rate limiter
    //        this allows peers to come in freely
    ensure!(
      !Self::exceeds_tx_rate_limit(Self::get_last_tx_block(&account_id), block),
      Error::<T>::TxRateLimitExceeded
    );
  
    // --- Get amount to be added as shares based on stake to balance added to account
    let mut delegate_stake_to_be_added_as_shares = Self::convert_to_shares(
      delegate_stake_to_be_transferred,
      total_to_subnet_delegated_stake_shares,
      total_to_subnet_delegated_stake_balance
    );

    // --- Mitigate inflation attack
    if total_to_subnet_delegated_stake_shares == 0 {
      // no need for saturation here
      TotalNodeDelegateStakeShares::<T>::mutate(to_subnet_id, to_subnet_node_id.clone(), |mut n| *n += 1000);
      delegate_stake_to_be_added_as_shares = delegate_stake_to_be_added_as_shares.saturating_sub(1000);
    }
    
    // --- Check rounding errors
    ensure!(
      delegate_stake_to_be_added_as_shares != 0,
      Error::<T>::CouldNotConvertToShares
    );

    Self::increase_account_node_delegate_stake_shares(
      &account_id,
      to_subnet_id, 
      to_subnet_node_id.clone(),
      delegate_stake_to_be_transferred,
      delegate_stake_to_be_added_as_shares,
    );

    // Set last block for rate limiting
    Self::set_last_tx_block(&account_id, block);

    Self::deposit_event(Event::DelegateNodeStakeSwitched { 
			account_id: account_id.clone(), 
			from_subnet_id: from_subnet_id, 
			from_subnet_node_id: from_subnet_node_id, 
			to_subnet_id: to_subnet_id, 
			to_subnet_node_id: to_subnet_node_id, 
			amount: delegate_stake_to_be_transferred 
    });

    Ok(())
  }

  pub fn increase_account_node_delegate_stake_shares(
    account_id: &T::AccountId,
    subnet_id: u32, 
    subnet_node_id: u32,
    amount: u128,
    shares: u128,
  ) {
    // -- increase account subnet staking shares balance
    AccountNodeDelegateStakeShares::<T>::mutate((account_id, subnet_id, subnet_node_id), |mut n| n.saturating_accrue(shares));

    // -- increase total subnet delegate stake balance
    TotalNodeDelegateStakeBalance::<T>::mutate(subnet_id, subnet_node_id, |mut n| n.saturating_accrue(amount));

    // -- increase total subnet delegate stake shares
    TotalNodeDelegateStakeShares::<T>::mutate(subnet_id, subnet_node_id, |mut n| n.saturating_accrue(shares));
  }
  
  pub fn decrease_account_node_delegate_stake_shares(
    account_id: &T::AccountId,
    subnet_id: u32, 
    subnet_node_id: u32,
    amount: u128,
    shares: u128,
  ) {
    // -- decrease account subnet staking shares balance
    AccountNodeDelegateStakeShares::<T>::mutate((account_id, subnet_id, subnet_node_id), |mut n| n.saturating_reduce(shares));

    // -- decrease total subnet delegate stake balance
    TotalNodeDelegateStakeBalance::<T>::mutate(subnet_id, subnet_node_id, |mut n| n.saturating_reduce(amount));

    // -- decrease total subnet delegate stake shares
    TotalNodeDelegateStakeShares::<T>::mutate(subnet_id, subnet_node_id, |mut n| n.saturating_reduce(shares));
  }

  /// Rewards are deposited here from the ``rewards.rs`` or by donations
  pub fn do_increase_node_delegate_stake(
    subnet_id: u32,
    subnet_node_id: u32,
    amount: u128,
  ) {
    // -- increase total subnet delegate stake 
    TotalNodeDelegateStakeBalance::<T>::mutate(subnet_id, subnet_node_id, |mut n| n.saturating_accrue(amount));
  }
}