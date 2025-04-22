use super::mock::*;
use crate::tests::test_utils::*;
use crate::Event;
use frame_support::{
	assert_noop, assert_ok, assert_err
};
use log::info;
use frame_support::traits::{OnInitialize, Currency};
use sp_std::collections::{btree_map::BTreeMap, btree_set::BTreeSet};
use frame_support::BoundedVec;
use sp_core::OpaquePeerId as PeerId;
use crate::{
  Error,
  LastSubnetRegistrationEpoch,
  MinSubnetRegistrationFee,
  MaxSubnetRegistrationFee,
  SubnetRegistrationInterval,
};

// #[test]
// fn test_registration_cost() {
//   new_test_ext().execute_with(|| {    
//     let last_registration_epoch = LastSubnetRegistrationEpoch::<Test>::get();
//     let fee_min: u128 = MinSubnetRegistrationFee::<Test>::get();
//     let fee_max: u128 = MaxSubnetRegistrationFee::<Test>::get();
//     let period: u32 = SubnetRegistrationInterval::<Test>::get();
//     // increase_epochs(period);

//     let epoch_length = EpochLength::get();
//     let block_number = System::block_number();
//     let epoch = System::block_number().saturating_div(epoch_length);
  
//     // division is not perfect due to epoch values being so little
//     let half_cycle_epoch = (period/2) as u32 % period;
//     let half_decrease_per_epoch = (fee_max.saturating_sub(fee_min)) / period as u128;
//     let half_fee = fee_max.saturating_sub(half_decrease_per_epoch * half_cycle_epoch as u128);
    
//     let cost = Network::registration_cost(0);
//     assert_eq!(cost, fee_min);

//     // Period is 100
//     LastSubnetRegistrationEpoch::<Test>::set(period-1);
//     let last_registration_epoch = LastSubnetRegistrationEpoch::<Test>::get();
//     let next_registration_epoch = Network::get_next_registration_epoch(last_registration_epoch);

//     // At 100, fee should be max
//     let cost = Network::registration_cost(next_registration_epoch);
//     assert_eq!(cost, fee_max);

//     // middle of registration period
//     let cost = Network::registration_cost(next_registration_epoch + (period/2));
//     assert_eq!(cost, half_fee);

//     // no registratin in current peroid, next period start should be min
//     let cost = Network::registration_cost(next_registration_epoch + period);
//     assert_eq!(cost, fee_min);

//     let cost = Network::registration_cost(next_registration_epoch + period + period/10);
//     assert_eq!(cost, fee_min);


//     // set to 150
//     LastSubnetRegistrationEpoch::<Test>::set(period+period/2);
//     let last_registration_epoch = LastSubnetRegistrationEpoch::<Test>::get();
//     let next_registration_epoch = Network::get_next_registration_epoch(last_registration_epoch); // 200

//     // Between 150-200 shouldn't be able to register


//     // At 200, fee should be max
//     let cost = Network::registration_cost(next_registration_epoch);
//     assert_eq!(cost, fee_max);

//     // middle of registration period
//     let cost = Network::registration_cost(next_registration_epoch + (period/2));
//     assert_eq!(cost, half_fee);

//     // no registratin in current peroid, next period start should be min
//     let cost = Network::registration_cost(next_registration_epoch + period);
//     assert_eq!(cost, fee_min);

//     let cost = Network::registration_cost(next_registration_epoch + period + period/10);
//     assert_eq!(cost, fee_min);


//   })
// }

#[test]
fn test_registration_cost2() {
  new_test_ext().execute_with(|| {    
    let last_registration_epoch = LastSubnetRegistrationEpoch::<Test>::get();
    let fee_min: u128 = MinSubnetRegistrationFee::<Test>::get();
    let fee_max: u128 = MaxSubnetRegistrationFee::<Test>::get();
    let period: u32 = SubnetRegistrationInterval::<Test>::get();
    // increase_epochs(period);

    let epoch_length = EpochLength::get();
    let block_number = System::block_number();
    let epoch = System::block_number().saturating_div(epoch_length);
  
    // division is not perfect due to epoch values being so little
    let half_cycle_epoch = (period/2) as u32 % period;
    let half_decrease_per_epoch = (fee_max.saturating_sub(fee_min)) / period as u128;
    let half_fee = fee_max.saturating_sub(half_decrease_per_epoch * half_cycle_epoch as u128);
    
    let cost = Network::registration_cost(0);
    assert_eq!(cost, fee_max);

    let cost = Network::registration_cost(period/2);
    assert_eq!(cost, half_fee);

    let cost = Network::registration_cost(period);
    assert_eq!(cost, fee_min);

    let cost = Network::registration_cost(period+1);
    assert_eq!(cost, fee_min);

    let cost = Network::registration_cost(period+period/2);
    assert_eq!(cost, fee_min);

    let cost = Network::registration_cost(period+period);
    assert_eq!(cost, fee_min);

    // Period is 100
    LastSubnetRegistrationEpoch::<Test>::set(period-1);
    let last_registration_epoch = LastSubnetRegistrationEpoch::<Test>::get();
    let next_registration_epoch = Network::get_next_registration_epoch(last_registration_epoch);

    // At 100, fee should be max
    let cost = Network::registration_cost(next_registration_epoch);
    assert_eq!(cost, fee_max);

    // middle of registration period
    let cost = Network::registration_cost(next_registration_epoch + (period/2));
    assert_eq!(cost, half_fee);

    // no registratin in current peroid, next period start should be min
    let cost = Network::registration_cost(next_registration_epoch + period);
    assert_eq!(cost, fee_min);

    let cost = Network::registration_cost(next_registration_epoch + period + period/10);
    assert_eq!(cost, fee_min);


    // set to 150
    LastSubnetRegistrationEpoch::<Test>::set(period+period/2);
    let last_registration_epoch = LastSubnetRegistrationEpoch::<Test>::get();
    let next_registration_epoch = Network::get_next_registration_epoch(last_registration_epoch); // 200

    // Between 150-200 shouldn't be able to register


    // At 200, fee should be max
    let cost = Network::registration_cost(next_registration_epoch);
    assert_eq!(cost, fee_max);

    // middle of registration period
    let cost = Network::registration_cost(next_registration_epoch + (period/2));
    assert_eq!(cost, half_fee);

    // no registratin in current peroid, next period start should be min
    let cost = Network::registration_cost(next_registration_epoch + period);
    assert_eq!(cost, fee_min);

    let cost = Network::registration_cost(next_registration_epoch + period + period/10);
    assert_eq!(cost, fee_min);


  })
}

#[test]
fn test_get_next_registration_epoch() {
  new_test_ext().execute_with(|| {
    let last_registration_epoch: u32 = LastSubnetRegistrationEpoch::<Test>::get();
    let period: u32 = SubnetRegistrationInterval::<Test>::get();

    // If no registrations yet, should return 0 to allow registration up to the SubnetRegistrationInterval
    let next_registration_epoch = Network::get_next_registration_epoch(0);
    assert_eq!(next_registration_epoch, 0);


    LastSubnetRegistrationEpoch::<Test>::set(1);

    let next_registration_epoch = Network::get_next_registration_epoch(1);
    assert_eq!(next_registration_epoch, period);


    LastSubnetRegistrationEpoch::<Test>::set(period);

    let next_registration_epoch = Network::get_next_registration_epoch(period);
    assert_eq!(next_registration_epoch, period*2);

    let next_registration_epoch = Network::get_next_registration_epoch(period-1);
    assert_eq!(next_registration_epoch, period*2);


    LastSubnetRegistrationEpoch::<Test>::set(period + period/2);

    let next_registration_epoch = Network::get_next_registration_epoch(period + period/2);
    assert_eq!(next_registration_epoch, period*2);
  })
}