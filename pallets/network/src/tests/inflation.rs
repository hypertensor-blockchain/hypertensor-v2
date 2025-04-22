use super::mock::*;
use crate::tests::test_utils::*;
use crate::Event;
use log::info;
use crate::inflation::Inflation;
// use crate::{
//   EpochsPerYear,
// };
//
//
//
//
//
//
//
// Inflation
//
//
//
//
//
//
//

#[test]
fn test_inflation_total() {
  new_test_ext().execute_with(|| {
    let _ = env_logger::builder().is_test(true).try_init();

    let inflation = Inflation::default();

    let mut last = inflation.total(0.0);

    for year in &[0.1, 0.5, 1.0, 2.0, 3.0, 4.0, 5.0, 100.0] {
      log::error!("test_inflation_total year {:?}", year);
      let total = inflation.total(*year);
      log::error!("test_inflation_total total {:?}", total);
      assert!(total < last);
      assert!(total >= inflation.terminal);
      last = total;
    }
    assert_eq!(last, inflation.terminal);
    // assert!(false);
  });
}

// #[test]
// fn test_inflation_test() {
//   new_test_ext().execute_with(|| {
//     let inflation = Inflation::default();
//     let x1 = Network::test(1.0);
//     log::error!("test_inflation_test x1 {:?}", x1);
//     // assert!(false);
//   });
// }

#[test]
fn test_get_epoch_emissions() {
  new_test_ext().execute_with(|| {
    let inflation = Inflation::default();

    Network::get_epoch_emissions(0);
  });
}

// #[test]
// fn test_inflation_epoch() {
//   new_test_ext().execute_with(|| {
//     let _ = env_logger::builder().is_test(true).try_init();

//     let inflation = Inflation::default();

//     let mut last = inflation.total(0.0);

//     let epochs_per_year = EpochsPerYear::get();
//     log::error!("test_inflation_epoch epochs_per_year {:?}", epochs_per_year);

//     for epoch in &[1, 2, 3, 4, 5, 4, 5, 100] {
//       let year = epoch / epochs_per_year;
//       log::error!("test_inflation_epoch year {:?}", year);

//       log::error!("test_inflation_epoch epoch {:?}", epoch);
//       let total = inflation.epoch(*epoch, epochs_per_year, 1e+9 as u128);
//       log::error!("test_inflation_epoch total {:?}", total);
//       assert!(total < last);
//       assert!(total >= inflation.terminal);
//       last = total;
//     }
//     assert_eq!(last, inflation.terminal);
//     assert!(false);
//   });
// }

#[test]
fn test_inflation_math() {
  new_test_ext().execute_with(|| {
    log::error!("test_inflation_math adassd");
  });
}