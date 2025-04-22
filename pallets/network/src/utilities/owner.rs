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
  pub fn do_owner_deactivate_subnet(origin: T::RuntimeOrigin, subnet_id: u32, path: Vec<u8>) -> DispatchResult {
    let coldkey: T::AccountId = ensure_signed(origin)?;

    ensure!(
      Self::is_subnet_owner(&coldkey, subnet_id),
      Error::<T>::NotSubnetOwner
    );

    Self::do_remove_subnet(
      path,
      SubnetRemovalReason::Owner,
    ).map_err(|e| e)?;

    Ok(())
  }

  pub fn do_owner_remove_subnet_node(origin: T::RuntimeOrigin, subnet_id: u32, subnet_node_id: u32) -> DispatchResult {
    let coldkey: T::AccountId = ensure_signed(origin)?;

    ensure!(
      Self::is_subnet_owner(&coldkey, subnet_id),
      Error::<T>::NotSubnetOwner
    );


    Ok(())
  }

  pub fn do_owner_update_registration_interval(origin: T::RuntimeOrigin, subnet_id: u32, value: u32) -> DispatchResult {
    let coldkey: T::AccountId = ensure_signed(origin)?;

    ensure!(
      Self::is_subnet_owner(&coldkey, subnet_id),
      Error::<T>::NotSubnetOwner
    );

    ensure!(
      value <= MaxSubnetRegistrationInterval::<T>::get(),
      Error::<T>::MaxSubnetRegistration
    );

    SubnetNodeRegistrationInterval::<T>::insert(subnet_id, value);

    Self::deposit_event(Event::SubnetEntryIntervalUpdate { 
      subnet_id: subnet_id,
      owner: coldkey, 
      value: value 
    });

    Ok(())
  }

  pub fn do_owner_update_activation_interval(origin: T::RuntimeOrigin, subnet_id: u32, value: u32) -> DispatchResult {
    let coldkey: T::AccountId = ensure_signed(origin)?;

    ensure!(
      Self::is_subnet_owner(&coldkey, subnet_id),
      Error::<T>::NotSubnetOwner
    );

    ensure!(
      value <= MaxSubnetActivationInterval::<T>::get(),
      Error::<T>::MaxSubnetActivation
    );

    SubnetNodeActivationInterval::<T>::insert(subnet_id, value);

    Self::deposit_event(Event::SubnetEntryIntervalUpdate { 
      subnet_id: subnet_id,
      owner: coldkey, 
      value: value 
    });

    Ok(())
  }

  pub fn do_owner_add_to_coldkey_whitelist(origin: T::RuntimeOrigin, subnet_id: u32, coldkeys: BTreeSet<T::AccountId>) -> DispatchResult {
    let coldkey: T::AccountId = ensure_signed(origin)?;

    ensure!(
      Self::is_subnet_owner(&coldkey, subnet_id),
      Error::<T>::NotSubnetOwner
    );

    ensure!(
      !Self::is_subnet_active(subnet_id),
      Error::<T>::SubnetMustBeRegistering
    );

    Ok(())
  }

  pub fn do_owner_remove_from_coldkey_whitelist(origin: T::RuntimeOrigin, subnet_id: u32, coldkeys: BTreeSet<T::AccountId>) -> DispatchResult {
    let coldkey: T::AccountId = ensure_signed(origin)?;

    ensure!(
      Self::is_subnet_owner(&coldkey, subnet_id),
      Error::<T>::NotSubnetOwner
    );

    ensure!(
      !Self::is_subnet_active(subnet_id),
      Error::<T>::SubnetMustBeRegistering
    );

    Ok(())
  }

  pub fn do_owner_set_max_subnet_registration_epochs(origin: T::RuntimeOrigin, subnet_id: u32, value: u32) -> DispatchResult {
    let coldkey: T::AccountId = ensure_signed(origin)?;

    ensure!(
      Self::is_subnet_owner(&coldkey, subnet_id),
      Error::<T>::NotSubnetOwner
    );

    ensure!(
      !Self::is_subnet_active(subnet_id),
      Error::<T>::SubnetMustBeRegistering
    );

    SubnetNodeRegistrationEpochs::<T>::insert(subnet_id, value);

    Ok(())
  }

  pub fn do_owner_update_queue_period(origin: T::RuntimeOrigin, subnet_id: u32, value: u32) -> DispatchResult {
    let coldkey: T::AccountId = ensure_signed(origin)?;

    ensure!(
      Self::is_subnet_owner(&coldkey, subnet_id),
      Error::<T>::NotSubnetOwner
    );

    SubnetNodeQueuePeriod::<T>::insert(subnet_id, value);

    Ok(())
  }

  pub fn do_owner_update_included_period(origin: T::RuntimeOrigin, subnet_id: u32, value: u32) -> DispatchResult {
    let coldkey: T::AccountId = ensure_signed(origin)?;

    ensure!(
      Self::is_subnet_owner(&coldkey, subnet_id),
      Error::<T>::NotSubnetOwner
    );

    Ok(())
  }

  /// Gives owner the ability to rearrange the queue, for instance, the owner can order the queue based on
  /// a validators performance
  pub fn do_owner_rearrange_queue(origin: T::RuntimeOrigin, subnet_id: u32, value: u32) -> DispatchResult {
    let coldkey: T::AccountId = ensure_signed(origin)?;

    ensure!(
      Self::is_subnet_owner(&coldkey, subnet_id),
      Error::<T>::NotSubnetOwner
    );

    Ok(())
  }

}