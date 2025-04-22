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
//
//
//
// @to-do: Increase precision to 100.0000

use super::*;
use libm::pow;
use sp_core::U256;

impl<T: Config> Pallet<T> {
  // Percentages are defined by default with 2 decimals of precision (100.00). 
	// The precision is indicated by PERCENTAGE_FACTOR
	// pub const PERCENTAGE_FACTOR: u128 = 10000;
  // pub const HALF_PERCENT: u128 = Self::PERCENTAGE_FACTOR / 2;
  
  
  pub const PERCENTAGE_FACTOR: u128 = 1000000000;
  pub const TWO_HUNDRED_PERCENT_FACTOR: u128 = Self::PERCENTAGE_FACTOR * 2;
  pub const HALF_PERCENT: u128 = Self::PERCENTAGE_FACTOR / 2;

  /// Percentage Math
  // Inspired by Aave PercentageMath

  /// `x` is value
  /// `y` is percentage
  /// Rounds down to the nearest 10th decimal
  pub fn percent_mul(x: u128, y: u128) -> u128 {
    if x == 0 || y == 0 {
      return 0
    }

    if x > ((u128::MAX - Self::HALF_PERCENT) / y) {
      return 0
    }

    // x * y / 100.0
    x.saturating_mul(y).saturating_div(Self::PERCENTAGE_FACTOR)
  }

  /// `x` is value
  /// `y` is percentage
  /// Rounds down to the nearest 10th decimal
  pub fn percent_div(x: u128, y: u128) -> u128 {
    if x == 0 || y == 0 {
      return 0
    }
    
    // x * 100.0 / y
    x.saturating_mul(Self::PERCENTAGE_FACTOR).saturating_div(y)
  }

  /// `x` is value
  /// `y` is percentage
  /// Rounds up to the nearest 10th decimal
  pub fn percent_mul_round_up(x: u128, y: u128) -> u128 {
    if x == 0 || y == 0 {
      return 0
    }

    if x > ((u128::MAX - Self::HALF_PERCENT) / y) {
      return u128::MAX
    }

    // (x * y + 50.0) / 100.0
    x.saturating_mul(y).saturating_div(Self::PERCENTAGE_FACTOR).saturating_add(u128::from(x % y != 0))
  }

  /// `x` is value
  /// `y` is percentage
  /// Rounds up to the nearest 10th decimal
  pub fn percent_div_round_up(x: u128, y: u128) -> u128 {
    if x == 0 || y == 0 {
      return 0
    }

    x.saturating_mul(Self::PERCENTAGE_FACTOR).saturating_div(y).saturating_add(u128::from(x % y == 0))
  }

  /// Get percentage in decimal format that uses `PERCENTAGE_FACTOR` as f64
  pub fn get_percent_as_f64(v: u128) -> f64 {
    v as f64 / Self::PERCENTAGE_FACTOR as f64
  }

  pub fn pow(x: f64, exp: f64) -> f64 {
    pow(x, exp)
  }


  /// 1e18 version
  pub const PERCENTAGE_FACTOR_V2: u128 = 1e+18 as u128;
  pub const HALF_PERCENT_V2: u128 = Self::PERCENTAGE_FACTOR_V2 / 2;

  pub const PERCENTAGE_FACTOR_V2_TEST: U256 = U256([0x8ac7230489e80000, 0x0, 0x0, 0x0]);
  pub const HALF_PERCENT_V2_TEST: U256 = U256([0x6f05b59d3b200000, 0x0, 0x0, 0x0]);

  /// Percentage Math
  /// Inspired by Aave PercentageMath

  /// `x` is value
  /// `y` is percentage
  /// Rounds down to the nearest 10th decimal
  pub fn percent_mul2(x: u128, y: u128) -> u128 {
    if x == 0 || y == 0 {
      return 0
    }
    
    let x = U256::from(x);
    let y = U256::from(y);

    if x > ((U256::MAX - Self::HALF_PERCENT_V2_TEST) / y) {
      return 0
    }

    // x * y / 100.0
    let result = x * y / Self::PERCENTAGE_FACTOR_V2_TEST;
    result.try_into().unwrap_or(u128::MAX)
  }

  /// `x` is value
  /// `y` is percentage
  /// Rounds down to the nearest 10th decimal
  pub fn percent_div2(x: u128, y: u128) -> u128 {
    if x == 0 || y == 0 {
      return 0
    }
    
    // x * 100.0 / y
    x.saturating_mul(Self::PERCENTAGE_FACTOR_V2).saturating_div(y)
  }

  // Inspired by DS Math
  // rounds to zero if x*y < WAD / 2
  pub fn wdiv(x: u128, y: u128) -> u128 {
    ((x * 1e+18 as u128) + (y / 2)) / y
  }

  //rounds to zero if x*y < WAD / 2
  pub fn wmul(x: u128, y: u128) -> u128 {
    ((x * y) + (1e+18 as u128 / 2)) / 1e+18 as u128
  }
}