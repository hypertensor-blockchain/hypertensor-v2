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

use crate::*;
use crate as pallet_network;
use frame_support::{
  parameter_types,
  traits::{
    Everything,
    tokens::{PayFromAccount, UnityAssetBalanceConversion},
  },
  PalletId,
  derive_impl,
  weights::{
		constants::WEIGHT_REF_TIME_PER_SECOND,
		Weight,
	},
};
use frame_system as system;
use sp_core::{ConstU128, ConstU32, ConstU64, H256, U256};
use sp_runtime::BuildStorage;
use sp_runtime::{
	traits::{
		BlakeTwo256, IdentifyAccount, Verify, IdentityLookup, AccountIdLookup
	},
	MultiSignature
};
use sp_runtime::Perbill;
pub use frame_system::{EnsureRoot, EnsureRootWithSuccess};
use sp_runtime::Permill;

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlockU32<Test>;

frame_support::construct_runtime!(
	pub enum Test
	{
    System: system,
    InsecureRandomnessCollectiveFlip: pallet_insecure_randomness_collective_flip,
    Balances: pallet_balances,
    Network: pallet_network,
    Collective: pallet_collective::<Instance1>,
    Treasury: pallet_treasury,
	}
);

// An index to a block.
pub type BlockNumber = u32;

pub type BalanceCall = pallet_balances::Call<Test>;

pub const MILLISECS_PER_BLOCK: u64 = 6000;

// NOTE: Currently it is not possible to change the slot duration after the chain has started.
//       Attempting to do so will brick block production.
pub const SLOT_DURATION: u64 = MILLISECS_PER_BLOCK;

// Time is measured by number of blocks.
pub const MINUTES: BlockNumber = 60_000 / (MILLISECS_PER_BLOCK as BlockNumber);
pub const HOURS: BlockNumber = MINUTES * 60;
pub const DAYS: BlockNumber = HOURS * 24;
pub const YEAR: BlockNumber = DAYS * 365;
pub const BLOCKS_PER_HALVING: BlockNumber = YEAR * 2;
pub const TARGET_MAX_TOTAL_SUPPLY: u128 = 2_800_000_000_000_000_000_000_000;
pub const INITIAL_REWARD_PER_BLOCK: u128 = (TARGET_MAX_TOTAL_SUPPLY / 2) / BLOCKS_PER_HALVING as u128;

pub const SECS_PER_BLOCK: u32 = 6000 / 1000;

pub const EPOCH_LENGTH: u32 = 10;
pub const BLOCKS_PER_EPOCH: u32 = SECS_PER_BLOCK * EPOCH_LENGTH;
pub const EPOCHS_PER_YEAR: u32 = YEAR as u32 / BLOCKS_PER_EPOCH;

parameter_types! {
  pub const BlockHashCount: BlockNumber = 250;
  pub const SS58Prefix: u8 = 42;
}

// pub type AccountId = U256;

pub type Signature = MultiSignature;

pub type AccountPublic = <Signature as Verify>::Signer;

pub type AccountId = <<Signature as Verify>::Signer as IdentifyAccount>::AccountId;

// The address format for describing accounts.
pub type Address = AccountId;

// Balance of an account.
pub type Balance = u128;

pub const EXISTENTIAL_DEPOSIT: u128 = 500;

impl pallet_insecure_randomness_collective_flip::Config for Test {}

impl pallet_balances::Config for Test {
  type Balance = Balance;
  type RuntimeEvent = RuntimeEvent;
  type DustRemoval = ();
  type ExistentialDeposit = ConstU128<EXISTENTIAL_DEPOSIT>;
  type AccountStore = System;
  type MaxLocks = ();
  type WeightInfo = ();
  type MaxReserves = ();
  type ReserveIdentifier = [u8; 8];
  type RuntimeHoldReason = ();
  type FreezeIdentifier = ();
  type MaxFreezes = ();
  type RuntimeFreezeReason = ();
}

#[derive_impl(frame_system::config_preludes::TestDefaultConfig)]
impl frame_system::Config for Test {
  type BaseCallFilter = Everything;
  type BlockWeights = ();
  type BlockLength = ();
  type Block = Block;
  type DbWeight = ();
  type RuntimeOrigin = RuntimeOrigin;
  type RuntimeCall = RuntimeCall;
  type Nonce = u32;
  type Hash = H256;
  type Hashing = BlakeTwo256;
  type AccountId = AccountId;
  type Lookup = AccountIdLookup<AccountId, ()>;
  type RuntimeEvent = RuntimeEvent;
  type BlockHashCount = BlockHashCount;
  type Version = ();
  type PalletInfo = PalletInfo;
  type AccountData = pallet_balances::AccountData<u128>;
  type OnNewAccount = ();
  type OnKilledAccount = ();
  type SystemWeightInfo = ();
  type SS58Prefix = SS58Prefix;
  type OnSetCode = ();
  type MaxConsumers = frame_support::traits::ConstU32<16>;
}

const NORMAL_DISPATCH_RATIO: Perbill = Perbill::from_percent(75);

parameter_types! {
	pub const CouncilMotionDuration: BlockNumber = 5 * DAYS;
	pub const CouncilMaxProposals: u32 = 100;
	pub const CouncilMaxMembers: u32 = 100;
  pub BlockWeights: frame_system::limits::BlockWeights =
    frame_system::limits::BlockWeights::with_sensible_defaults(
      Weight::from_parts(2u64 * WEIGHT_REF_TIME_PER_SECOND, u64::MAX),
      NORMAL_DISPATCH_RATIO,
    );
	pub MaxCollectivesProposalWeight: Weight = Perbill::from_percent(50) * BlockWeights::get().max_block;
}

type CouncilCollective = pallet_collective::Instance1;
impl pallet_collective::Config<CouncilCollective> for Test {
	type RuntimeOrigin = RuntimeOrigin;
	type Proposal = RuntimeCall;
	type RuntimeEvent = RuntimeEvent;
	type MotionDuration = CouncilMotionDuration;
	type MaxProposals = CouncilMaxProposals;
	type MaxMembers = CouncilMaxMembers;
	type DefaultVote = pallet_collective::PrimeDefaultVote;
	type WeightInfo = ();
	type SetMembersOrigin = EnsureRoot<AccountId>;
	type MaxProposalWeight = MaxCollectivesProposalWeight;
}

parameter_types! {
	pub const Burn: Permill = Permill::from_percent(50);
	pub const TreasuryPalletId: PalletId = PalletId(*b"py/trsry");
	pub const SpendLimit: Balance = u128::MAX;
	pub TreasuryAccount: AccountId = Treasury::account_id();
}

impl pallet_treasury::Config for Test {
	type PalletId = TreasuryPalletId;
	type Currency = Balances;
	type RejectOrigin = EnsureRoot<AccountId>;
	type RuntimeEvent = RuntimeEvent;
	type SpendPeriod = ConstU32<2>;
	type Burn = Burn;
	type BurnDestination = (); // Just gets burned.
	type WeightInfo = ();
	type SpendFunds = ();
	type MaxApprovals = ConstU32<100>;
	type SpendOrigin = EnsureRootWithSuccess<AccountId, SpendLimit>;
	type AssetKind = ();
	type Beneficiary = AccountId;
	type BeneficiaryLookup = IdentityLookup<Self::Beneficiary>;
	type Paymaster = PayFromAccount<Balances, TreasuryAccount>;
	type BalanceConverter = UnityAssetBalanceConversion;
	type PayoutPeriod = ConstU32<10>;
	#[cfg(feature = "runtime-benchmarks")]
	type BenchmarkHelper = ();
}

parameter_types! {
	pub const EpochLength: u32 = EPOCH_LENGTH; // Testnet 600 blocks per erpoch / 69 mins per epoch, Local 10
	pub const EpochsPerYear: u32 = EPOCHS_PER_YEAR; // Testnet 600 blocks per erpoch / 69 mins per epoch, Local 10
  pub const NetworkPalletId: PalletId = PalletId(*b"/network");
  pub const MinProposalStake: u128 = 1_000_000_000_000_000_000;
  pub const DelegateStakeCooldownEpochs: u32 = 100;
  pub const NodeDelegateStakeCooldownEpochs: u32 = 100; 
  pub const StakeCooldownEpochs: u32 = 100;
	pub const DelegateStakeEpochsRemovalWindow: u32 = 10;
  pub const MaxDelegateStakeUnlockings: u32 = 32;
  pub const MaxStakeUnlockings: u32 = 32;
}

impl Config for Test {
  type WeightInfo = ();
	type RuntimeEvent = RuntimeEvent;
  type Currency = Balances;
  type MajorityCollectiveOrigin = pallet_collective::EnsureProportionAtLeast<AccountId, CouncilCollective, 2, 3>;
  type SuperMajorityCollectiveOrigin = pallet_collective::EnsureProportionAtLeast<AccountId, CouncilCollective, 4, 5>;
	type EpochLength = EpochLength;
	type EpochsPerYear = EpochsPerYear;
  type StringLimit = ConstU32<100>;
	type InitialTxRateLimit = ConstU32<0>;
  type Randomness = InsecureRandomnessCollectiveFlip;
	type PalletId = NetworkPalletId;
  type DelegateStakeCooldownEpochs = DelegateStakeCooldownEpochs;
  type NodeDelegateStakeCooldownEpochs = NodeDelegateStakeCooldownEpochs; 
  type StakeCooldownEpochs = DelegateStakeCooldownEpochs;
	type DelegateStakeEpochsRemovalWindow = DelegateStakeEpochsRemovalWindow;
  type MaxDelegateStakeUnlockings = MaxDelegateStakeUnlockings;
  type MaxStakeUnlockings = MaxStakeUnlockings;
  type MinProposalStake = MinProposalStake;
  type TreasuryAccount = TreasuryAccount;
}

pub fn new_test_ext() -> sp_io::TestExternalities {
	frame_system::GenesisConfig::<Test>::default()
		.build_storage()
		.unwrap()
		.into()
}

pub(crate) fn network_events() -> Vec<crate::Event<Test>> {
	System::events()
		.into_iter()
		.map(|r| r.event)
		.filter_map(|e| if let RuntimeEvent::Network(inner) = e { Some(inner) } else { None })
		.collect()
}
