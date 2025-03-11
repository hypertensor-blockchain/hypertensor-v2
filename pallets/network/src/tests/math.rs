use super::mock::*;
use crate::tests::test_utils::*;
use log::info;

// ///
// ///
// ///
// ///
// ///
// ///
// ///
// /// Math
// ///
// ///
// ///
// ///
// ///
// ///
// ///

#[test]
fn test_percent_mul() {
  new_test_ext().execute_with(|| {
    let value = Network::percent_mul(53000000, 300000000);

    assert_eq!(value, 15900000, "percent_mul didn't round down");

    // let value = Network::percent_mul_round_up(53000000, 300000000);

    // assert_eq!(value, 15900000, "percent_mul_round_up didn't round up");

    let value = Network::percent_mul(100000000e+18 as u128, PERCENTAGE_FACTOR);

    assert_ne!(value, 0, "percent_mul didn't round down");
    assert_ne!(value, u128::MAX, "percent_mul didn't round down");

    // let value = Network::percent_mul_round_up(100000000e+18 as u128, PERCENTAGE_FACTOR);

    // assert_ne!(value, 0, "percent_mul_round_up didn't round down");
    // assert_ne!(value, u128::MAX, "percent_mul_round_up didn't round down");


    let value = Network::percent_mul(1000e+18 as u128, 500_000_000);
    assert_eq!(value, 500e+18 as u128, "percent_mul_round_up didn't round up");

  });
}

#[test]
fn test_percent_div() {
  new_test_ext().execute_with(|| {
    // // 100.00 | 10000
    // let value = Network::percent_div(1, 3000);

    // assert_eq!(value, 3, "percent_div didn't round down");

    // let value = Network::percent_div_round_up(1, 3000);

    // assert_eq!(value, 4, "percent_div_round_up didn't round up");

    // 100.0000000 | 1000000000
    let value = Network::percent_div(100000000, 300000000);

    assert_eq!(value, 333333333, "percent_div didn't round down");

    // let value = Network::percent_div_round_up(100000000, 300000000);

    // assert_eq!(value, 400000000, "percent_div_round_up didn't round up");
  });
}