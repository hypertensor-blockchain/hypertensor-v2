use super::mock::*;
use crate::tests::test_utils::*;
use frame_support::traits::OnInitialize;
use sp_runtime::traits::Header;
use log::info;

// ///
// ///
// ///
// ///
// ///
// ///
// ///
// /// Randomization
// ///
// ///
// ///
// ///
// ///
// ///
// ///

pub fn setup_blocks(blocks: u32) {
  let mut parent_hash = System::parent_hash();

  for i in 1..(blocks + 1) {
    System::reset_events();
    System::initialize(&i, &parent_hash, &Default::default());
    InsecureRandomnessCollectiveFlip::on_initialize(i);

    let header = System::finalize();
    parent_hash = header.hash();
    System::set_block_number(*header.number());
  }
}

#[test]
fn test_randomness() {
  new_test_ext().execute_with(|| {
    setup_blocks(38);
    let gen_rand_num = Network::generate_random_number(1);
    let rand_num = Network::get_random_number(96, 0);
  });
}
