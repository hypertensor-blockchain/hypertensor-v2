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
use libm::{exp, pow};

pub struct Inflation {
  /// Initial inflation percentage, from time=0
  pub initial: f64,

  /// Terminal inflation percentage, to time=INF
  pub terminal: f64,

  /// Rate per year, at which inflation is lowered until reaching terminal
  ///  i.e. inflation(year) == MAX(terminal, initial*((1-taper)^year))
  pub taper: f64,

  /// Percentage of total inflation allocated to the foundation
  pub foundation: f64,
  /// Duration of foundation pool inflation, in years
  pub foundation_term: f64,
}

const DEFAULT_INITIAL: f64 = 0.1;
const DEFAULT_TERMINAL: f64 = 0.015;
const DEFAULT_TAPER: f64 = 0.15;
const DEFAULT_FOUNDATION: f64 = 0.05;
const DEFAULT_FOUNDATION_TERM: f64 = 7.0;

impl Default for Inflation {
  fn default() -> Self {
    Self {
      initial: DEFAULT_INITIAL,
      terminal: DEFAULT_TERMINAL,
      taper: DEFAULT_TAPER,
      foundation: DEFAULT_FOUNDATION,
      foundation_term: DEFAULT_FOUNDATION_TERM,
    }
  }
}

impl Inflation {
  pub fn epoch(&self, epoch: u32, epochs_per_year: u32, denominator: u128) -> f64 {
    let years_elapsed = epoch as f64 / epochs_per_year as f64;
    // let rate = self.initial * pow(1.0 - self.terminal, years_elapsed);

    self.total(years_elapsed)

    // Ensure inflation does not go below the minimum taper rate
    // let final_rate = rate.max(self.taper);

    // final_rate
    // final_rate as u128 * denominator
  }

  /// portion of total that goes to validators
  pub fn validator(&self, year: f64) -> f64 {
    self.total(year) - self.foundation(year)
  }

  /// portion of total that goes to foundation
  pub fn foundation(&self, year: f64) -> f64 {
    if year < self.foundation_term {
      self.total(year) * self.foundation
    } else {
      0.0
    }
  }

  /// inflation rate at year
  pub fn total(&self, year: f64) -> f64 {
    let tapered = self.initial * pow(1.0 - self.taper, year);

    if tapered > self.terminal {
      tapered
    } else {
      self.terminal
    }
  }

  pub fn year_from_epoch(&self, epoch: u32, epochs_per_year: u32) -> f64 {
    epoch as f64 / epochs_per_year as f64
  }
}

impl<T: Config> Pallet<T> {
  /// Get the current epochs total emissions to subnets
  ///
  /// Inflation is based on the current network activity
  ///   - Subnet activity
  ///   - Subnet node activity
  ///
  /// # Steps
  ///
  /// 1. Gets utilization factors
  ///   - Subnet utilization
  ///   - Subnet node utilization
  ///
  /// 2. Combine the utilization factors based on the `SubnetInflationFactor`
  ///   e.g. SubnetInflationFactor == 80%, then subnet node factor will be 20% (1.0-sif)
  ///   - SIF is 80%
  ///   - SNIF is 20%
  ///
  /// If subnet utilization is 20% and subnet node utilization is 15%
  ///   e.g. 19% = 20% * 80% + 15% * 20% = `inflation_factor`
  ///
  /// 3. Get exponential adjustment of `inflation_factor`
  ///
  /// 4. Get network inflation on epoch
  ///
  /// 5. Adjust network inflation based on network activity
  ///
  pub fn get_epoch_emissions(
    epoch: u32, 
  ) -> u128 {
    let max_subnets: u32 = MaxSubnets::<T>::get();
    let mut total_activate_subnets: u32 = TotalActiveSubnets::<T>::get();
    // There can be n+1 subnets at this time before 1 is removed in the epoch steps
    if total_activate_subnets > max_subnets {
      total_activate_subnets = max_subnets;
    }

    // ==========================
    // --- Get subnet utilization
    // ==========================
    let subnet_utilization_rate: f64 = total_activate_subnets as f64 / max_subnets as f64;
    let adj_subnet_utilization_rate: f64 = Self::pow(
      subnet_utilization_rate, 
      Self::get_percent_as_f64(SubnetInflationAdjFactor::<T>::get())
    ).min(1.0);

    // Max subnet nodes per subnet
    let max_nodes: u32 = max_subnets.saturating_mul(MaxSubnetNodes::<T>::get());
    let total_active_nodes: u32 = TotalActiveNodes::<T>::get();

    // ==========================
    // --- Get subnet node utilization
    // ==========================
    let node_utilization_rate: f64 = total_active_nodes as f64 / max_nodes as f64;
    let adj_node_utilization_rate: f64 = Self::pow(
      node_utilization_rate, 
      Self::get_percent_as_f64(SubnetNodeInflationAdjFactor::<T>::get())
    ).min(1.0);

    // ==========================
    // --- Get final utilization factors
    // ==========================
    // Subnet inflation factor
    let sif: f64 = Self::get_percent_as_f64(SubnetInflationFactor::<T>::get());
    // Subnet node inflation factor
    let snif: f64 = 1.0 - sif;

    let adj_subnet_utilization_rate: f64 = subnet_utilization_rate * sif;
    let adj_node_utilization_rate: f64 = node_utilization_rate * snif;

    // --- Get percentage of inflation to use in current epoch
    // This is the network activity factor
    let inflation_factor: f64 = Self::get_inflation_factor(snif + adj_node_utilization_rate);

    // ==========================
    // --- Get current epochs total inflation
    //
    // * Adjusts the inflation based on network activity using `let inflation_factor`
    // ==========================
    // let total_issuance: f64 = Self::get_total_network_issuance() as f64;
    let total_issuance: f64 = 19536777003861893185536.0;
    let epochs_per_year: f64 = T::EpochsPerYear::get() as f64;

    log::error!("inflation total_issuance: {:?}", total_issuance as u128);

    let inflation = Inflation::default();

    let year: f64 = epoch as f64 / epochs_per_year;
    log::error!("inflation year: {:?}", year);

    // --- Get current yearly inflation
    let apr: f64 = inflation.total(year);
    log::error!("inflation apr: {:?}", apr);

    (total_issuance * apr * inflation_factor) as u128
  }

  pub fn get_epoch_emissions_adj(
    epoch: u32, 
    increase_issuance: u128,
    decrease_issuance: u128,
  ) -> u128 {
    let max_subnets: u32 = MaxSubnets::<T>::get();
    let mut total_activate_subnets: u32 = TotalActiveSubnets::<T>::get();
    // There can be n+1 subnets at this time before 1 is removed in the epoch steps
    if total_activate_subnets > max_subnets {
      total_activate_subnets = max_subnets;
    }

    // ==========================
    // --- Get subnet utilization
    // ==========================
    let subnet_utilization_rate: f64 = total_activate_subnets as f64 / max_subnets as f64;
    let adj_subnet_utilization_rate: f64 = Self::pow(
      subnet_utilization_rate, 
      Self::get_percent_as_f64(SubnetInflationAdjFactor::<T>::get())
    ).min(1.0);

    // Max subnet nodes per subnet
    let max_nodes: u32 = max_subnets.saturating_mul(MaxSubnetNodes::<T>::get());
    let total_active_nodes: u32 = TotalActiveNodes::<T>::get();

    // ==========================
    // --- Get subnet node utilization
    // ==========================
    let node_utilization_rate: f64 = total_active_nodes as f64 / max_nodes as f64;
    let adj_node_utilization_rate: f64 = Self::pow(
      node_utilization_rate, 
      Self::get_percent_as_f64(SubnetNodeInflationAdjFactor::<T>::get())
    ).min(1.0);

    // ==========================
    // --- Get final utilization factors
    // ==========================
    // Subnet inflation factor
    let sif: f64 = Self::get_percent_as_f64(SubnetInflationFactor::<T>::get());
    // Subnet node inflation factor
    let snif: f64 = 1.0 - sif;

    let adj_subnet_utilization_rate: f64 = subnet_utilization_rate * sif;
    let adj_node_utilization_rate: f64 = node_utilization_rate * snif;

    // --- Get percentage of inflation to use in current epoch
    // This is the network activity factor
    let inflation_factor: f64 = Self::get_inflation_factor(snif + adj_node_utilization_rate);

    // ==========================
    // --- Get current epochs total inflation
    //
    // * Adjusts the inflation based on network activity using `let inflation_factor`
    // ==========================
    let total_issuance: f64 = Self::get_total_network_issuance() as f64 + increase_issuance as f64 - decrease_issuance as f64;
    let epochs_per_year: f64 = T::EpochsPerYear::get() as f64;

    log::error!("inflation total_issuance: {:?}", total_issuance);

    let inflation = Inflation::default();

    let year: f64 = epoch as f64 / epochs_per_year;
    log::error!("inflation year: {:?}", year);

    // --- Get current yearly inflation
    let apr: f64 = inflation.total(year);
    log::error!("inflation apr: {:?}", apr);

    (total_issuance * apr * inflation_factor) as u128
  }

  pub fn get_inflation_factor(x: f64) -> f64 {
    if x >= 1.0 {
      return 1.0
    }
    
    let k: f64 = Self::get_percent_as_f64(InflationAdjFactor::<T>::get());
    if k == 0.0 {
      return 1.0
    }

    pow(x, k).min(1.0)
  }
}