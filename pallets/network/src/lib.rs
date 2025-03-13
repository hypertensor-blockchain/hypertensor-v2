//! # Template Pallet
//!
//! A pallet with minimal functionality to help developers understand the essential components of
//! writing a FRAME pallet. It is typically used in beginner tutorials or in Substrate template
//! nodes as a starting point for creating a new pallet and **not meant to be used in production**.
//!
//! ## Overview
//!
//! This template pallet contains basic examples of:
//! - declaring a storage item that stores a single `u32` value
//! - declaring and using events
//! - declaring and using errors
//! - a dispatchable function that allows a user to set a new value to storage and emits an event
//!   upon success
//! - another dispatchable function that causes a custom error to be thrown
//!
//! Each pallet section is annotated with an attribute using the `#[pallet::...]` procedural macro.
//! This macro generates the necessary code for a pallet to be aggregated into a FRAME runtime.
//!
//! Learn more about FRAME macros [here](https://docs.substrate.io/reference/frame-macros/).
//!
//! ### Pallet Sections
//!
//! The pallet sections in this template are:
//!
//! - A **configuration trait** that defines the types and parameters which the pallet depends on
//!   (denoted by the `#[pallet::config]` attribute). See: [`Config`].
//! - A **means to store pallet-specific data** (denoted by the `#[pallet::storage]` attribute).
//!   See: [`storage_types`].
//! - A **declaration of the events** this pallet emits (denoted by the `#[pallet::event]`
//!   attribute). See: [`Event`].
//! - A **declaration of the errors** that this pallet can throw (denoted by the `#[pallet::error]`
//!   attribute). See: [`Error`].
//! - A **set of dispatchable functions** that define the pallet's functionality (denoted by the
//!   `#[pallet::call]` attribute). See: [`dispatchables`].
//!
//! Run `cargo doc --package pallet-template --open` to view this pallet's documentation.

// We make sure this pallet uses `no_std` for compiling to Wasm.
#![cfg_attr(not(feature = "std"), no_std)]

// extern crate alloc;

// Re-export pallet items so that they can be accessed from the crate namespace.
pub use pallet::*;
use codec::{Decode, Encode};
use frame_support::{
	dispatch::{DispatchResult},
	traits::{tokens::WithdrawReasons, Get, Currency, ReservableCurrency, ExistenceRequirement, Randomness, EnsureOrigin},
	PalletId,
	ensure,
	fail,
	storage::bounded_vec::BoundedVec,
};
use frame_system::{self as system, ensure_signed};
use scale_info::prelude::string::String;
use scale_info::prelude::vec::Vec;
use sp_std::collections::{btree_map::BTreeMap, btree_set::BTreeSet};
use sp_core::OpaquePeerId as PeerId;
use strum::IntoEnumIterator;
use strum_macros::{EnumIter, FromRepr};
use sp_runtime::traits::TrailingZeroInput;
use frame_system::pallet_prelude::OriginFor;
use sp_std::ops::BitAnd;
use sp_runtime::Saturating;

// FRAME pallets require their own "mock runtimes" to be able to run unit tests. This module
// contains a mock runtime specific for testing this pallet's functionality.
// #[cfg(test)]
// mod mock;

// This module contains the unit tests for this pallet.
// Learn about pallet unit testing here: https://docs.substrate.io/test/unit-testing/
#[cfg(test)]
mod tests;

// ./target/release/solochain-template-node --dev
// Every callable function or "dispatchable" a pallet exposes must have weight values that correctly
// estimate a dispatchable's execution time. The benchmarking module is used to calculate weights
// for each dispatchable and generates this pallet's weight.rs file. Learn more about benchmarking here: https://docs.substrate.io/test/benchmark/
#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;
pub mod weights;
pub use weights::*;

pub mod utilities;
pub use utilities::*;
pub mod stake;
pub use stake::*;
pub mod rpc_info;
pub use rpc_info::*;
pub mod admin;
pub use admin::*;

mod subnet_validator;
mod rewards;
mod proposal;

// All pallet logic is defined in its own module and must be annotated by the `pallet` attribute.
#[frame_support::pallet]
pub mod pallet {
	// Import various useful types required by all FRAME pallets.
	use super::*;
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;
	use sp_std::vec;
	use sp_std::vec::Vec;
	
	// The `Pallet` struct serves as a placeholder to implement traits, methods and dispatchables
	// (`Call`s) in this pallet.
	#[pallet::pallet]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

	/// The pallet's configuration trait.
	///
	/// All our types and constants a pallet depends on must be declared here.
	/// These types are defined generically and made concrete when the pallet is declared in the
	/// `runtime/src/lib.rs` file of your chain.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// The overarching runtime event type.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		/// A type representing the weights required by the dispatchables of this pallet.
		type WeightInfo: WeightInfo;

		type Currency: Currency<Self::AccountId> + ReservableCurrency<Self::AccountId> + Send + Sync;

		/// Majority council 2/3s
    type MajorityCollectiveOrigin: EnsureOrigin<Self::RuntimeOrigin>;

		/// Majority council 4/5s - Used in functions that include tokenization
		type SuperMajorityCollectiveOrigin: EnsureOrigin<Self::RuntimeOrigin>;

		#[pallet::constant]
		type EpochLength: Get<u64>;

		#[pallet::constant]
		type StringLimit: Get<u32>;
	
		#[pallet::constant] // Initial transaction rate limit.
		type InitialTxRateLimit: Get<u64>;
			
		#[pallet::constant]
		type PalletId: Get<PalletId>;

		#[pallet::constant]
		type DelegateStakeCooldownEpochs: Get<u64>;

		#[pallet::constant]
		type StakeCooldownEpochs: Get<u64>;

		#[pallet::constant]
		type DelegateStakeEpochsRemovalWindow: Get<u64>;

		#[pallet::constant]
		type MaxDelegateStakeUnlockings: Get<u32>;
		
		#[pallet::constant]
		type MaxStakeUnlockings: Get<u32>;

		type Randomness: Randomness<Self::Hash, BlockNumberFor<Self>>;

		#[pallet::constant]
		type MinProposalStake: Get<u128>;
	}

	/// A storage item for this pallet.
	///
	/// In this template, we are declaring a storage item called `Something` that stores a single
	/// `u32` value. Learn more about runtime storage here: <https://docs.substrate.io/build/runtime-storage/>
	#[pallet::storage]
	pub type Something<T> = StorageValue<_, u32>;

	/// Events that functions in this pallet can emit.
	///
	/// Events are a simple means of indicating to the outside world (such as dApps, chain explorers
	/// or other users) that some notable update in the runtime has occurred. In a FRAME pallet, the
	/// documentation for each event field and its parameters is added to a node's metadata so it
	/// can be used by external interfaces or tools.
	///
	///	The `generate_deposit` macro generates a function on `Pallet` called `deposit_event` which
	/// will convert the event type of your pallet into `RuntimeEvent` (declared in the pallet's
	/// [`Config`] trait) and deposit it using [`frame_system::Pallet::deposit_event`].
	/// Events for the pallet.
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		// Subnets
		SubnetRegistered { account_id: T::AccountId, path: Vec<u8>, subnet_id: u32 },
		SubnetActivated { subnet_id: u32 },
		SubnetDeactivated { subnet_id: u32, reason: SubnetRemovalReason },

		// Subnet Nodes
		SubnetNodeRegistered { 
			subnet_id: u32, 
			subnet_node_id: u32, 
			coldkey: T::AccountId, 
			hotkey: T::AccountId, 
			peer_id: PeerId, 
		},
		SubnetNodeActivated { subnet_id: u32, subnet_node_id: u32 },
		SubnetNodeDeactivated { subnet_id: u32, subnet_node_id: u32 },
		SubnetNodeRemoved { subnet_id: u32, subnet_node_id: u32 },

		// Stake
		StakeAdded(u32, T::AccountId, T::AccountId, u128),
		StakeRemoved(u32, T::AccountId, T::AccountId, u128),

		DelegateStakeAdded(u32, T::AccountId, u128),
		DelegateStakeRemoved(u32, T::AccountId, u128),
		DelegateStakeSwitched(u32, u32, T::AccountId, u128),

		DelegateNodeStakeAdded { account_id: T::AccountId, subnet_id: u32, subnet_node_id: u32, amount: u128 },
		DelegateNodeStakeRemoved { account_id: T::AccountId, subnet_id: u32, subnet_node_id: u32, amount: u128 },
		DelegateNodeStakeSwitched { 
			account_id: T::AccountId, 
			from_subnet_id: u32, 
			from_subnet_node_id: u32, 
			to_subnet_id: u32, 
			to_subnet_node_id: u32, 
			amount: u128 
		},

		// Admin 
    SetMaxSubnets(u32),
    SetMinSubnetNodes(u32),
    SetMaxSubnetNodes(u32),
    SetMinStakeBalance(u128),
    SetTxRateLimit(u64),

		// Proposals
		Proposal { subnet_id: u32, proposal_id: u32, epoch: u32, plaintiff: T::AccountId, defendant: T::AccountId, plaintiff_data: Vec<u8> },
		ProposalChallenged { subnet_id: u32, proposal_id: u32, defendant: T::AccountId, defendant_data: Vec<u8> },
		ProposalAttested { subnet_id: u32, proposal_id: u32, account_id: T::AccountId, attestor_data: Vec<u8> },
		ProposalVote { subnet_id: u32, proposal_id: u32, account_id: T::AccountId, vote: VoteType },
		ProposalFinalized { subnet_id: u32, proposal_id: u32 },
		ProposalCanceled { subnet_id: u32, proposal_id: u32 },

		// Validation and Attestation
		ValidatorSubmission { subnet_id: u32, account_id: T::AccountId, epoch: u32},
		Attestation { subnet_id: u32, account_id: T::AccountId, epoch: u32},

		Slashing { subnet_id: u32, account_id: T::AccountId, amount: u128},

		// Rewards data
		RewardResult { subnet_id: u32, attestation_percentage: u128 },
	}

	/// Errors that can be returned by this pallet.
	///
	/// Errors tell users that something went wrong so it's important that their naming is
	/// informative. Similar to events, error documentation is added to a node's metadata so it's
	/// equally important that they have helpful documentation associated with them.
	///
	/// This type of runtime error can be up to 4 bytes in size should you want to return additional
	/// information.
	#[pallet::error]
	pub enum Error<T> {
		/// Errors should have helpful documentation associated with them.

		/// Subnet must be registering or activated, this error usually occurs during the enactment period
		SubnetMustBeRegisteringOrActivated,
		/// Node hasn't been initialized for required epochs to be an accountant
		NodeAccountantEpochNotReached,
		/// Maximum subnets reached
		MaxSubnets,
		/// Account has subnet peer under subnet already
		SubnetNodeExist,
		/// Not Uid owner
		NotUidOwner,
		/// Subnet node already activated
		SubnetNodeAlreadyActivated,
		///
		SubnetNodeNotActivated,
		/// Node ID already in use
		PeerIdExist,
		/// Node ID already in use
		PeerIdNotExist,
		/// Subnet peer doesn't exist
		SubnetNodeNotExist,
		/// Subnet already exists
		SubnetExist,
		/// Subnet registration cooldown period not met
		SubnetRegistrationCooldown,
		/// Max total subnet memory exceeded
		MaxTotalSubnetMemory,
		/// Max subnet memory size exceeded
		MaxSubnetMemory,
		/// Invalid registration block
		InvalidSubnetRegistrationBlocks,
		/// Subnet node must be unstaked to re-register to use the same balance
		InvalidSubnetRegistrationCooldown,
		/// Subnet doesn't exist
		SubnetNotExist,
		/// Minimum required subnet nodes not reached
		SubnetNodesMin,
		/// Maximum allowed subnet nodes reached
		SubnetNodesMax,
		/// Transaction rate limiter exceeded
		TxRateLimitExceeded,
		/// PeerId format invalid
		InvalidPeerId,
		/// The provided signature is incorrect.
		WrongSignature,
		InvalidSubnetId,

		/// Maximum amount of subnet entries surpassed, see subnet `entry_interval` for more information
		MaxSubnetEntryIntervalReached,
		/// Maximum `entry_interval` parameter entered during subnet registration
		MaxSubnetEntryInterval,

		DelegateStakeTransferPeriodExceeded,
		MustUnstakeToRegister,
		// Admin
		/// Consensus block epoch_length invalid, must reach minimum
		InvalidEpochLengthsInterval,
		/// Invalid maximimum subnets, must not exceed maximum allowable
		InvalidMaxSubnets,
		/// Invalid min subnet nodes, must not be less than minimum allowable
		InvalidMinSubnetNodes,
		/// Invalid maximimum subnet nodes, must not exceed maximimum allowable
		InvalidMaxSubnetNodes,
		/// Invalid minimum stake balance, must be greater than or equal to minimim required stake balance
		InvalidMinStakeBalance,
		/// Invalid percent number, must be in 1e4 format. Used for elements that only require correct format
		InvalidPercent,
		InvalidMaxSubnetMemoryMB,

		// Staking
		/// u128 -> BalanceOf conversion error
		CouldNotConvertToBalance,
		/// Not enough balance on Account to stake and keep alive
		NotEnoughBalanceToStake,
		NotEnoughBalance,
		/// Required unstake epochs not met based on
		RequiredUnstakeEpochsNotMet,
		/// Amount will kill account
		BalanceWithdrawalError,
		/// Not enough stake to withdraw
		NotEnoughStakeToWithdraw,
		MaxStakeReached,
		// if min stake not met on both stake and unstake
		MinStakeNotReached,
		// delegate staking
		CouldNotConvertToShares,
		// 
		MaxDelegatedStakeReached,
		//
		InsufficientCooldown,
		//
		UnstakeWindowFinished,
		//
		MaxUnlockingsPerEpochReached,
		//
		MaxUnlockingsReached,
		//
		NoDelegateStakeUnbondingsOrCooldownNotMet,
		NoStakeUnbondingsOrCooldownNotMet,
		// Conversion to balance was zero
		InsufficientBalanceToSharesConversion,

		/// Not enough balance to withdraw bid for proposal
		NotEnoughBalanceToBid,

		QuorumNotReached,

		/// Dishonest propsal type
		PropsTypeInvalid,

		PartiesCannotVote,

		ProposalNotExist,
		ProposalNotChallenged,
		ProposalChallenged,
		ProposalChallengePeriodPassed,
		PropsalAlreadyChallenged,
		NotChallenger,
		NotEligible,
		AlreadyVoted,
		VotingPeriodInvalid,
		ChallengePeriodPassed,
		DuplicateVote,
		PlaintiffIsDefendant,

		InvalidSubnetRewardsSubmission,
		SubnetInitializing,
		SubnetActivatedAlready,
		InvalidSubnetRemoval,

		// Validation and Attestation
		/// Subnet rewards data already submitted by validator
		SubnetRewardsAlreadySubmitted,
		/// Not epoch validator
		InvalidValidator,
		/// Already attested validator data
		AlreadyAttested,
		/// Invalid rewards data length
		InvalidRewardsDataLength,


		ProposalInvalid,
		NotDefendant,
		NotPlaintiff,
		ProposalUnchallenged,
		ProposalComplete,
		/// Subnet node as defendant has proposal activated already
		NodeHasActiveProposal,
		/// Not the key owner
		NotKeyOwner,
		/// Not owner of hotkey that owns subnet node
		NotSubnetNodeOwner,
		/// Subnet Node param A must be unique
		SubnetNodeUniqueParamTaken,
		/// Subnet node param A is already set
		SubnetNodeUniqueParamIsSet,
		/// Subnet node params must be Some
		SubnetNodeNonUniqueParamMustBeSome,
		/// Non unique subnet node parameters can be updated once per SubnetNodeNonUniqueParamUpdateInterval
		SubnetNodeNonUniqueParamUpdateIntervalNotReached,
		/// Key owner taken
		KeyOwnerTaken,
		/// No change between current and new delegate reward rate, make sure to increase or decrease it
		NoDelegateRewardRateChange,
		/// Invalid delegate reward rate above 100%
		InvalidDelegateRewardRate,
		/// Rate of change to great for decreasing reward rate, see MaxRewardRateDecrease
		SurpassesMaxRewardRateDecrease,
		/// Too many updates to reward rate in the RewardRateUpdatePeriod
		MaxRewardRateUpdates,
		/// Invalid curve parameters
		InvalidCurveParameters,
	}
	
	/// hotkey: 				Hotkey of subnet node for interacting with subnet on-chain communication
	/// peer_id: 				Peer ID of subnet node within subnet
	/// initialized:		Block initialized
	/// classification:	Subnet node classification for on-chain permissions
	/// a:							(Optional) Unique data for subnet to use and lookup via RPC, can only be added at registration
	/// b:							(Optional) Data for subnet to use and lookup via RPC
	/// c:							(Optional) Data for subnet to use and lookup via RPC
	#[derive(Default, Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, PartialOrd, Ord, scale_info::TypeInfo)]
	pub struct SubnetNode<AccountId> {
		pub hotkey: AccountId,
		pub peer_id: PeerId,
		pub initialized: u64,
		pub classification: SubnetNodeClassification,
		pub delegate_reward_rate: u128,
		pub last_delegate_reward_rate_update: u64,
		pub a: Option<BoundedVec<u8, DefaultSubnetNodeUniqueParamLimit>>,
		pub b: Option<BoundedVec<u8, DefaultSubnetNodeUniqueParamLimit>>,
		pub c: Option<BoundedVec<u8, DefaultSubnetNodeUniqueParamLimit>>,
	}

	#[derive(Default, Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, scale_info::TypeInfo)]
	pub struct SubnetNodeInfo<AccountId> {
		pub subnet_node_id: u32,
		pub coldkey: AccountId,
		pub hotkey: AccountId,
		pub peer_id: PeerId,
		pub classification: SubnetNodeClassification,
		pub a: Option<BoundedVec<u8, DefaultSubnetNodeUniqueParamLimit>>,
		pub b: Option<BoundedVec<u8, DefaultSubnetNodeUniqueParamLimit>>,
		pub c: Option<BoundedVec<u8, DefaultSubnetNodeUniqueParamLimit>>,
	}

	/// Subnet node classes
	/// 
	/// # Arguments
	///
	/// *Deactivated: Subnet node is temporarily activated (done manually). Available to Validator class only.
	/// *Registered: Subnet node registered, not included in consensus
	/// *Idle: Subnet node is activated as idle, unless subnet is registering, and automatically updates on the first successful consensus epoch
	/// *Included: Subnet node automatically updates to Included from Idle on the first successful consensus epoch after being Idle
	/// *Validator: Subnet node updates to Submittble from Included on the first successful consensus epoch they are included in consensus data
	#[derive(Default, EnumIter, FromRepr, Copy, Encode, Decode, Clone, PartialOrd, PartialEq, Eq, RuntimeDebug, Ord, scale_info::TypeInfo)]
  pub enum SubnetNodeClass {
		Deactivated,
		#[default] Registered,
    Idle,
    Included,
		Validator,
  }

	#[derive(Default, Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, Ord, PartialOrd, scale_info::TypeInfo)]
	pub struct SubnetNodeClassification {
		pub class: SubnetNodeClass,
		pub start_epoch: u64,
	}

	impl<AccountId> SubnetNode<AccountId> {
		pub fn has_classification(&self, required: &SubnetNodeClass, epoch: u64) -> bool {
			self.classification.class >= *required && self.classification.start_epoch <= epoch
		}
	}


	/// Incentives protocol format
	///
	/// Scoring is calculated off-chain between subnet nodes hosting AI subnets together
	#[derive(Default, Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, scale_info::TypeInfo)]
	pub struct SubnetNodeData {
		pub peer_id: PeerId,
		pub score: u128,
	}

	/// Incentives protocol format V2 (not in use)
	///
	/// Scoring is calculated off-chain between subnet nodes hosting AI subnets together
	#[derive(Default, Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, scale_info::TypeInfo)]
	pub struct SubnetNodeIncentives {
		pub uid: u32,
		pub score: u128,
	}

	/// Reasons for a subnets removal
	///
	/// # Enums
	///
	/// *MaxPenalties: Subnet has surpasses the maximum penalties.
	/// *MinSubnetNodes: Subnet has went under the minumum subnet nodes required.
	/// *MinSubnetDelegateStake: Subnet delegate stake balance went under minimum required supply.
	/// *Council: Removed by council.
	/// *EnactmentPeriod: Subnet registered but never activated within the enactment period.
	/// *MaxSubnets: Lowest rated subnet removed if there are maximum subnets
	#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, scale_info::TypeInfo)]
  pub enum SubnetRemovalReason {
    MaxPenalties,
		MinSubnetNodes,
		MinSubnetDelegateStake,
		Council,
		EnactmentPeriod,
		MaxSubnets,
  }

	/// Attests format for consensus
	/// ``u64`` is the block number of the accounts attestation for subnets to utilize to measure attestation speed
	/// The blockchain itself doesn't utilize this data
	// pub type Attests<AccountId> = BTreeMap<AccountId, u64>;

	#[derive(Default, Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, scale_info::TypeInfo)]
	pub struct RewardsData {
		pub validator_id: u32, // Chosen validator of the epoch
		pub attests: BTreeMap<u32, u64>, // Count of attestations of the submitted data
		pub data: Vec<SubnetNodeData>, // Data submitted by chosen validator
		pub args: Option<BoundedVec<u8, DefaultValidatorArgsLimit>>, // Optional arguements to pass for subnet to validate
	}

	#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, scale_info::TypeInfo)]
  pub struct CurveParametersSet {
		pub x_curve_start: u128, // The range of ``max-min`` to start descending the curve
		pub y_end: u128, // The ``y`` end point on descending curve
		pub y_start: u128, // The ``y`` start point on descending curve
		pub x_rise: u128, // The rise from 0, usually should be 1/100
		pub max_x: u128,
	}

	/// Vote types
	///
	/// # Enums
	///
	/// *Yay: Vote yay.
	/// *Nay: Vote nay.
	#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, scale_info::TypeInfo)]
  pub enum VoteType {
    Yay,
    Nay,
  }

	/// Subnet data used before activation
	///
	/// # Arguments
	///
	/// * `path` - Path to download the model, this can be HuggingFace, IPFS, anything.
	/// * `memory_mb` - The memory requirement to serve the entirety of the model
	/// * `registration_blocks` - Registration blocks the subnet registerer wants to use
	#[derive(Default, Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, scale_info::TypeInfo)]
	pub struct RegistrationSubnetData {
		pub path: Vec<u8>,
		pub memory_mb: u128,
		pub registration_blocks: u64,
		pub entry_interval: u64,
	}
	
	/// Subnet data used before activation
	#[derive(Default, Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, scale_info::TypeInfo)]
	pub struct RegisteredSubnetNodesData<AccountId> {
		pub subnet_id: u32,
		pub subnet_node: SubnetNode<AccountId>,
	}

	/// Subnet data
	///
	/// # Arguments
	///
	/// * `id` - Unique identifier.
	/// * `path` - Path to download the model, this can be HuggingFace, IPFS, anything.
	/// * `min_nodes` - Minimum required nodes for subnet.
	/// * `target_nodes` - Target nodes of subnet based on minimum nodes.
	/// * `memory_mb` - Memory requirements of subnet.
	/// * `initialized` - Block subnet registered.
	/// * `registration_blocks` - Registration blocks the subnet registerer wants to use.
	/// * `activated` - Block subnet activated.
	/// * `entry_interval` - Blocks between each node entry into the subnet.
	#[derive(Default, Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, scale_info::TypeInfo)]
	pub struct SubnetData {
		pub id: u32,
		pub path: Vec<u8>,
		pub min_nodes: u32,
		pub target_nodes: u32,
		pub memory_mb: u128,
		pub initialized: u64,
		pub registration_blocks: u64,
		pub activated: u64,
		pub entry_interval: u64,
	}

	/// Mapping of votes of a subnet proposal
	///
	/// # Arguments
	///
	/// * `yay` - Mapping of subnet node IDs voted yay.
	/// * `nay` - Mapping of subnet node IDs voted nay.
	#[derive(Default, Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, scale_info::TypeInfo)]
	pub struct VoteParams {
		pub yay: BTreeSet<u32>,
		pub nay: BTreeSet<u32>,
	}

	/// Proposal parameters
	///
	/// # Arguments
	///
	/// * `subnet_id` - Subnet ID.
	/// * `plaintiff_id` - Proposers subnet node ID.
	/// * `defendant_id` - Defendants subnet node ID.
	/// * `plaintiff_bond` - Plaintiffs bond to create proposal (minimum proposal bond at time of proposal).
	/// * `defendant_bond` - Defendants bond to create proposal (matches plaintiffs bond).
	/// * `eligible_voters` - Mapping of subnet node IDs eligible to vote at time of proposal.
	/// * `votes` - Mapping of votes (`yay` and `nay`).
	/// * `start_block` - Block when proposer proposes proposal.
	/// * `challenge_block` - Block when defendant disputes proposal.
	/// * `plaintiff_data` - Proposers data to prove removal. Data is based on subnet removal reasons off-chain.
	/// * `defendant_data` - Defedants data to prove dispute.
	/// * `complete` - If proposal is complete.
	#[derive(Default, Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, scale_info::TypeInfo)]
	pub struct ProposalParams {
		pub subnet_id: u32,
		pub plaintiff_id: u32,
		pub defendant_id: u32,
		pub plaintiff_bond: u128,
		pub defendant_bond: u128,
		pub eligible_voters: BTreeSet<u32>, // Those eligible to vote at time of the proposal
		pub votes: VoteParams,
		pub start_block: u64,
		pub challenge_block: u64,
		pub plaintiff_data: Vec<u8>,
		pub defendant_data: Vec<u8>,
		pub complete: bool,
	}

	#[pallet::type_value]
	pub fn DefaultZeroU32() -> u32 {
		0
	}
	#[pallet::type_value]
	pub fn DefaultZeroU64() -> u64 {
		0
	}
	#[pallet::type_value]
	pub fn DefaultAccountId<T: Config>() -> T::AccountId {
		T::AccountId::decode(&mut TrailingZeroInput::zeroes()).unwrap()
	}
	#[pallet::type_value]
	pub fn DefaultPeerId() -> PeerId {
		PeerId(Vec::new())
	}
	#[pallet::type_value]
	pub fn DefaultTxRateLimit<T: Config>() -> u64 {
		T::InitialTxRateLimit::get()
	}
	#[pallet::type_value]
	pub fn DefaultLastTxBlock() -> u64 {
		0
	}
	#[pallet::type_value]
	pub fn DefaultMaxSubnetPenaltyCount() -> u32 {
		16
	}
	#[pallet::type_value]
	pub fn DefaultSubnetNodeRegistrationEpochs() -> u64 {
		16
	}
	#[pallet::type_value]
	pub fn DefaultSubnetNodesClasses<T: Config>() -> BTreeMap<T::AccountId, u64> {
		BTreeMap::new()
	}
	#[pallet::type_value]
	pub fn DefaultSubnetNode<T: Config>() -> SubnetNode<T::AccountId> {
		return SubnetNode {
			hotkey: T::AccountId::decode(&mut TrailingZeroInput::zeroes()).unwrap(),
			peer_id: PeerId(Vec::new()),
			initialized: 0,
			classification: SubnetNodeClassification {
				class: SubnetNodeClass::Registered,
				start_epoch: 0,
			},
			delegate_reward_rate: 0,
			last_delegate_reward_rate_update: 0,
			a: Some(BoundedVec::new()),
			b: Some(BoundedVec::new()),
      c: Some(BoundedVec::new()),
		};
	}
	#[pallet::type_value]
	pub fn DefaultMaxSubnetNodes() -> u32 {
		// 94
		// 1024
		// 254
		512
	}
	#[pallet::type_value]
	pub fn DefaultAccountTake() -> u128 {
		0
	}
		#[pallet::type_value]
	pub fn DefaultMaxStakeBalance() -> u128 {
		280000000000000000000000
	}
	#[pallet::type_value]
	pub fn DefaultMinStakeBalance() -> u128 {
		1000e+18 as u128
	}
	#[pallet::type_value]
	pub fn DefaultMinSubnetDelegateStakePercentage() -> u128 {
		// 10000
		1000000000
	}
	#[pallet::type_value]
	pub fn DefaultMinSubnetDelegateStake() -> u128 {
		1000e+18 as u128
	}
	#[pallet::type_value]
	pub fn DefaultMaxDelegateStakeBalance() -> u128 {
		280000000000000000000000
	}
	#[pallet::type_value]
	pub fn DefaultDelegateStakeTransferPeriod() -> u64 {
		1000
	}
	#[pallet::type_value]
	pub fn DefaultDelegateStakeRewardsPercentage() -> u128 {
		// 1100
		110000000
	}
	#[pallet::type_value]
	pub fn DefaultDelegateStakeCooldown() -> u64 {
		0
	}
	#[pallet::type_value]
	pub fn DefaultDelegateStakeUnbondingLedger() -> BTreeMap<u64, u128> {
		// We use epochs because cooldowns are based on epochs
		// {
		// 	epoch_start: u64, // cooldown begin epoch (+ cooldown duration for unlock)
		// 	balance: u128,
		// }
		BTreeMap::new()
	}
	#[pallet::type_value]
	pub fn DefaultStakeUnbondingLedger() -> BTreeMap<u64, u128> {
		// We use epochs because cooldowns are based on epochs
		// {
		// 	epoch_start: u64, // cooldown begin epoch (+ cooldown duration for unlock)
		// 	balance: u128,
		// }
		BTreeMap::new()
	}
	#[pallet::type_value]
	pub fn DefaultBaseRewardPerMB() -> u128 {
		1e+18 as u128
	}
	#[pallet::type_value]
	pub fn DefaultBaseValidatorReward() -> u128 {
		1e+18 as u128
	}
	#[pallet::type_value]
	pub fn DefaultRewardPerSubnet() -> u128 {
		1e+9 as u128
	}
	#[pallet::type_value]
	pub fn DefaultMaxSubnetNodePenalties() -> u32 {
		3
	}
	#[pallet::type_value]
	pub fn DefaultSlashPercentage() -> u128 {
		// 312
		31250000
	}
	#[pallet::type_value]
	pub fn DefaultMaxSlashAmount() -> u128 {
		1e+18 as u128
	}
	#[pallet::type_value]
	pub fn DefaultMinAttestationPercentage() -> u128 {
		// 2/3
		660000000
	}
	#[pallet::type_value]
	pub fn DefaultMinVastMajorityAttestationPercentage() -> u128 {
		// 7/8
		875000000
	}
	#[pallet::type_value]
	pub fn DefaultTargetSubnetNodesMultiplier() -> u128 {
		// 1/3
		333333333
	}
	#[pallet::type_value]
	pub fn DefaultBaseSubnetNodeMemoryMB() -> u128 {
		16_000
	}
	#[pallet::type_value]
	pub fn DefaultMaxSubnetMemoryMB() -> u128 {
		1_000_000
	}
	#[pallet::type_value]
	pub fn DefaultMaxTotalSubnetMemoryMB() -> u128 {
		10_000_000
	}
	#[pallet::type_value]
	pub fn DefaultMinSubnetNodes() -> u32 {
		// development and mainnet
		// 6
		// local && testnet
		1
	}
	#[pallet::type_value]
	pub fn DefaultMinSubnetRegistrationBlocks() -> u64 {
		// 9 days at 6s blocks
		// 129_600
		
		// Testnet && Local 24 blocks
		25
	}
	#[pallet::type_value]
	pub fn DefaultMaxSubnetRegistrationBlocks() -> u64 {
		// 21 days at 6s blocks
		// 302_400

		// Testnet 3 days
		43200
	}
	#[pallet::type_value]
	pub fn DefaultMaxSubnetNodeRegistrationEpochs() -> u32 {
		16
	}
	#[pallet::type_value]
	pub fn DefaultSubnetActivationEnactmentPeriod() -> u64 {
		// 3 days at 6s blocks
		43_200
	}
	#[pallet::type_value]
	pub fn DefaultMinNodesCurveParameters() -> CurveParametersSet {
		// math.rs PERCENT_FACTOR format
		return CurveParametersSet {
			x_curve_start: 15 * 1000000000 / 100, // 0.15
			y_end: 10 * 1000000000 / 100, // 0.10
			y_start: 75 * 1000000000 / 100, // 0.75
			x_rise: 1000000000 / 100, // 0.01
			max_x: 56 * 1000000000 / 100, // 0.56
		}
	}
	#[pallet::type_value]
	pub fn DefaultMaxSubnets() -> u32 {
		64
	}
	#[pallet::type_value]
	pub fn DefaultTotalSubnetMemoryMB() -> u128 {
		0
	}
	#[pallet::type_value]
	pub fn DefaultSubnetNodeUniqueParamLimit() -> u32 {
		2024
	}
	#[pallet::type_value]
	pub fn DefaultValidatorArgsLimit() -> u32 {
		4096
	}
	#[pallet::type_value]
	pub fn DefaultMinSubnetRegistrationFee() -> u128 {
		100e+18 as u128
	}
	#[pallet::type_value]
	pub fn DefaultMaxSubnetRegistrationFee() -> u128 {
		1000e+18 as u128
	}
	#[pallet::type_value]
	pub fn DefaultSubnetRegistrationInterval() -> u32 {
		// 1 week based on 6s blocks using epochs
		// 1008
		// Testnet:
		// * 1 hour, 600 blocks, 6 epochs
		6
	}
	#[pallet::type_value]
	pub fn DefaultMaxSubnetEntryInterval() -> u64 {
		// 1 week based on 6s blocks
		// 100800
		0
	}

	/// Count of subnets
	#[pallet::storage]
	#[pallet::getter(fn total_subnets)]
	pub type TotalSubnets<T> = StorageValue<_, u32, ValueQuery>;
	
	#[pallet::storage]
	#[pallet::getter(fn max_subnets)]
	pub type MaxSubnets<T> = StorageValue<_, u32, ValueQuery, DefaultMaxSubnets>;

	// Mapping of each subnet stored by ID, uniqued by `SubnetPaths`
	// Stores subnet data by a unique id
	#[pallet::storage] // subnet_id => data struct
	pub type SubnetsData<T: Config> = StorageMap<_, Blake2_128Concat, u32, SubnetData>;

	// Max per subnet node entry interval to any given subnet
	#[pallet::storage] // subnet_id => block_interval
	pub type MaxSubnetEntryInterval<T: Config> = StorageValue<_, u64, ValueQuery, DefaultMaxSubnetEntryInterval>;

	/// The maximum a single node can enter a subnet per blocks interval
	#[pallet::storage] // subnet_id => block
	pub type LastSubnetEntry<T: Config> = StorageMap<_, Blake2_128Concat, u32, u64, ValueQuery, DefaultZeroU64>;

	/// Maximum subnet memory per subnet
	#[pallet::storage]
	pub type MaxSubnetMemoryMB<T> = StorageValue<_, u128, ValueQuery, DefaultMaxSubnetMemoryMB>;

	/// Total sum of subnet memory available in the network
	#[pallet::storage]
	pub type MaxTotalSubnetMemoryMB<T> = StorageValue<_, u128, ValueQuery, DefaultMaxTotalSubnetMemoryMB>;
	
	/// Total subnet memory across all subnets
	#[pallet::storage]
	pub type TotalSubnetMemoryMB<T: Config> = StorageValue<_, u128, ValueQuery, DefaultTotalSubnetMemoryMB>;

	// Ensures no duplicate subnet paths within the network at one time
	// If a subnet path is voted out, it can be voted up later on and any
	// stakes attached to the subnet_id won't impact the re-initialization
	// of the subnet path.
	#[pallet::storage]
	#[pallet::getter(fn subnet_paths)]
	pub type SubnetPaths<T: Config> = StorageMap<_, Blake2_128Concat, Vec<u8>, u32>;

	/// Minimum blocks required from subnet registration to activation
	#[pallet::storage]
	pub type MinSubnetRegistrationBlocks<T> = StorageValue<_, u64, ValueQuery, DefaultMinSubnetRegistrationBlocks>;

	/// Maximum blocks required from subnet registration to activation
	#[pallet::storage]
	pub type MaxSubnetRegistrationBlocks<T> = StorageValue<_, u64, ValueQuery, DefaultMaxSubnetRegistrationBlocks>;

	/// Time period allowable for subnet activation following registration period
	#[pallet::storage]
	pub type SubnetActivationEnactmentPeriod<T> = StorageValue<_, u64, ValueQuery, DefaultSubnetActivationEnactmentPeriod>;

	// Minimum amount of nodes required per subnet
	// required for subnet activity
	#[pallet::storage]
	#[pallet::getter(fn min_subnet_nodes)]
	pub type MinSubnetNodes<T> = StorageValue<_, u32, ValueQuery, DefaultMinSubnetNodes>;

	#[pallet::storage]
	pub type MinNodesCurveParameters<T> = StorageValue<_, CurveParametersSet, ValueQuery, DefaultMinNodesCurveParameters>;

	// Maximim nodes in a subnet at any given time
	#[pallet::storage]
	#[pallet::getter(fn max_subnet_nodes)]
	pub type MaxSubnetNodes<T> = StorageValue<_, u32, ValueQuery, DefaultMaxSubnetNodes>;

	// Max epochs where consensus isn't formed before subnet being removed
	#[pallet::storage]
	pub type MaxSubnetPenaltyCount<T> = StorageValue<_, u32, ValueQuery, DefaultMaxSubnetPenaltyCount>;
	
	// Count of epochs a subnet has consensus errors
	#[pallet::storage] // subnet_id => count
	pub type SubnetPenaltyCount<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		u32,
		u32,
		ValueQuery,
	>;
	
	#[pallet::storage] // subnet_uid --> u32
	#[pallet::getter(fn total_subnet_nodes)]
	pub type TotalSubnetNodes<T: Config> =
		StorageMap<_, Blake2_128Concat, u32, u32, ValueQuery>;
	
	// #[pallet::storage]
	// #[pallet::getter(fn pending_actions)]
	// pub type PendingActionsStorage<T: Config> = StorageValue<_, Option<PendingActions<T::AccountId>>, ValueQuery>;

	#[pallet::type_value]
	pub fn DefaultDeactivationLedger<T: Config>() -> BTreeSet<SubnetNodeDeactivation> {
		BTreeSet::new()
	}

	#[pallet::type_value]
	pub fn DefaultMaxDeactivations() -> u32 {
		512
	}
	#[pallet::type_value]
	pub fn DefaultSubnetNodeNonUniqueParamUpdateInterval() -> u32 {
		1
	}
	#[pallet::type_value]
	pub fn DefaultRewardRateUpdatePeriod() -> u64 {
		// 1 day at 6 seconds a block (86,000s per day)
		14400
	}
	#[pallet::type_value]
	pub fn DefaultMaxRewardRateDecrease() -> u128 {
		// 1%
		10_000_000
	}

	

	#[derive(Default, Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, scale_info::TypeInfo, PartialOrd, Ord)]
	pub struct SubnetNodeDeactivation {
		pub subnet_id: u32,
		pub subnet_node_id: u32,
	}

	#[pallet::storage]
	pub type MaxDeactivations<T: Config> = 
		StorageValue<_, u32, ValueQuery, DefaultMaxDeactivations>;

	#[pallet::storage]
	pub type DeactivationLedger<T: Config> = 
		StorageValue<_, BTreeSet<SubnetNodeDeactivation>, ValueQuery, DefaultDeactivationLedger<T>>;
		
	/// Total epochs a subnet node can stay in registration phase. If surpassed, they are removed on the first successful
	/// consensus epoch
	#[pallet::storage]
	pub type SubnetNodeRegistrationEpochs<T: Config> = StorageValue<_, u64, ValueQuery, DefaultSubnetNodeRegistrationEpochs>;
	
	#[pallet::storage] // subnet_id --> u32
	pub type TotalSubnetNodeUids<T: Config> = StorageMap<_, Identity, u32, u32, ValueQuery>;

	// Hotkey => Coldkey
	#[pallet::storage]
	pub type HotkeyOwner<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, T::AccountId, ValueQuery, DefaultAccountId<T>>;

	// Subnet ID => Hotkey => Subnet Node ID
	#[pallet::storage]
	pub type HotkeySubnetNodeId<T: Config> = StorageDoubleMap<_, Identity, u32, Blake2_128Concat, T::AccountId, u32, OptionQuery>;
	
	// Subnet ID => Subnet Node ID => Hotkey
	#[pallet::storage]
	pub type SubnetNodeIdHotkey<T: Config> = StorageDoubleMap<_, Identity, u32, Identity, u32, T::AccountId, OptionQuery>;
	
	#[pallet::storage] // subnet_id --> uid --> data
	#[pallet::getter(fn subnet_nodes2)]
	pub type SubnetNodesData<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		u32,
		Identity,
		u32,
		SubnetNode<T::AccountId>,
		ValueQuery,
		DefaultSubnetNode<T>,
	>;

	#[pallet::storage] // subnet_id --> peer_id --> subnet_node_id
	#[pallet::getter(fn subnet_node_account)]
	pub type SubnetNodeAccount<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		u32,
		Identity,
		PeerId,
		u32,
		ValueQuery,
		DefaultZeroU32,
	>;

	// Used for unique parameters
	#[pallet::storage] // subnet_id --> param --> peer_id
	pub type SubnetNodeUniqueParam<T> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		u32,
		Identity,
		BoundedVec<u8, DefaultSubnetNodeUniqueParamLimit>,
		PeerId,
		ValueQuery,
		DefaultPeerId,
	>;
	
	#[pallet::storage]
	pub type SubnetNodeNonUniqueParamUpdateInterval<T: Config> = 
		StorageValue<_, u32, ValueQuery, DefaultSubnetNodeNonUniqueParamUpdateInterval>;

	// // Last update of non unique subnet node params
	// #[pallet::storage]
	// pub type SubnetNodeNonUniqueParamLastSet<T: Config> = StorageDoubleMap<
	// 	_,
	// 	Blake2_128Concat,
	// 	u32,
	// 	Identity,
	// 	T::AccountId,
	// 	u32,
	// 	ValueQuery,
	// 	DefaultZeroU32,
	// >;
	
	#[pallet::storage]
	pub type SubnetNodeNonUniqueParamLastSet<T: Config> = StorageDoubleMap<
		_,
		Identity,
		u32,
		Identity,
		u32,
		u32,
		ValueQuery,
		DefaultZeroU32,
	>;

	/// Base subnet node memory used for calculating minimum and target nodes for a subnet
	#[pallet::storage]
	pub type BaseSubnetNodeMemoryMB<T> = StorageValue<_, u128, ValueQuery, DefaultBaseSubnetNodeMemoryMB>;

	#[pallet::storage]
	pub type TargetSubnetNodesMultiplier<T> = StorageValue<_, u128, ValueQuery, DefaultTargetSubnetNodesMultiplier>;

	// Rate limit
	#[pallet::storage] // ( tx_rate_limit )
	pub type TxRateLimit<T> = StorageValue<_, u64, ValueQuery, DefaultTxRateLimit<T>>;

	// Last transaction on rate limited functions
	#[pallet::storage] // key --> last_block
	pub type LastTxBlock<T: Config> =
		StorageMap<_, Identity, T::AccountId, u64, ValueQuery, DefaultLastTxBlock>;


	//
	// Validate / Attestation
	//

	#[pallet::storage] // subnet ID => epoch  => subnet node ID
	pub type SubnetRewardsValidator<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		u32,
		Identity,
		u32,
		u32,
	>;

	#[pallet::storage] // subnet ID => epoch  => data
	pub type SubnetRewardsSubmission<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		u32,
		Identity,
		u32,
		RewardsData,
	>;

	#[pallet::storage]
	pub type MinAttestationPercentage<T> = StorageValue<_, u128, ValueQuery, DefaultMinAttestationPercentage>;

	#[pallet::storage]
	pub type MinVastMajorityAttestationPercentage<T> = StorageValue<_, u128, ValueQuery, DefaultMinVastMajorityAttestationPercentage>;

	//
	// Rewards (validator, scoring consensus)
	//

	// Base reward per epoch for validators
	// This is the base reward to subnet validators on successful attestation
	#[pallet::storage]
	pub type BaseValidatorReward<T> = StorageValue<_, u128, ValueQuery, DefaultBaseValidatorReward>;

	/// Base reward per MB per epoch based on 4,380 MB per year
	#[pallet::storage]
	pub type BaseRewardPerMB<T> = StorageValue<_, u128, ValueQuery, DefaultBaseRewardPerMB>;

	/// Assumed cost per MB for each epoch
	// TODO: (not included in logic yet)
	// This will help determine inflation for each epoch on the cost to run a subnet node
	#[pallet::storage]
	pub type ServerCostPerMB<T> = StorageValue<_, u128, ValueQuery, DefaultBaseRewardPerMB>;
	
	#[pallet::storage]
	pub type SlashPercentage<T> = StorageValue<_, u128, ValueQuery, DefaultSlashPercentage>;

	#[pallet::storage]
	pub type MaxSlashAmount<T> = StorageValue<_, u128, ValueQuery, DefaultMaxSlashAmount>;

	// The total rewards that go into the rewards pool per epoch per subnet
	#[pallet::storage]
	pub type RewardPerSubnet<T> = StorageValue<_, u128, ValueQuery, DefaultRewardPerSubnet>;

	// Maximum epochs in a row a subnet node can be absent from validator submitted consensus data
	#[pallet::storage]
	pub type MaxSubnetNodePenalties<T> = StorageValue<_, u32, ValueQuery, DefaultMaxSubnetNodePenalties>;
	
	// If subnet node is absent from inclusion in consensus information or attestings, or validator data isn't attested
	// We don't count penalties per account because a user can bypass this by having multiple accounts
	#[pallet::storage]
	pub type SubnetNodePenalties<T> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		u32,
		Identity,
		u32,
		u32,
		ValueQuery,
		DefaultZeroU32,
	>;

	#[pallet::type_value]
	pub fn DefaultNodeAttestationRemovalThreshold() -> u128 {
		// 8500
		850000000
	}

	// Attestion percentage required to increment a nodes penalty count up
	#[pallet::storage]
	pub type NodeAttestationRemovalThreshold<T: Config> = StorageValue<_, u128, ValueQuery, DefaultNodeAttestationRemovalThreshold>;


	//
	// Staking
	// 

	#[pallet::storage] // stores epoch balance of rewards from block rewards to be distributed to nodes/stakers
	#[pallet::getter(fn stake_vault_balance)]
	pub type StakeVaultBalance<T> = StorageValue<_, u128, ValueQuery>;

	#[pallet::storage] // ( total_stake )
	#[pallet::getter(fn total_stake)]
	pub type TotalStake<T: Config> = StorageValue<_, u128, ValueQuery>;

	// Total stake sum of all nodes in specified subnet
	#[pallet::storage] // subnet_uid --> peer_data
	#[pallet::getter(fn total_subnet_stake)]
	pub type TotalSubnetStake<T: Config> =
		StorageMap<_, Blake2_128Concat, u32, u128, ValueQuery>;

	// An accounts stake per subnet
	#[pallet::storage] // account--> subnet_id --> u128
	#[pallet::getter(fn account_subnet_stake)]
	pub type AccountSubnetStake<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		T::AccountId,
		Identity,
		u32,
		u128,
		ValueQuery,
		DefaultAccountTake,
	>;
	
	#[pallet::storage]
	pub type StakeUnbondingLedger<T: Config> = 
		StorageMap<_, Blake2_128Concat, T::AccountId, BTreeMap<u64, u128>, ValueQuery, DefaultStakeUnbondingLedger>;

	// Maximum stake balance per subnet
	// Only checked on `do_add_stake` and ``
	// A subnet staker can have greater than the max stake balance although any rewards
	// they would receive based on their stake balance will only account up to the max stake balance allowed
	#[pallet::storage]
	pub type MaxStakeBalance<T: Config> = StorageValue<_, u128, ValueQuery, DefaultMaxStakeBalance>;

	// Minimum required subnet peer stake balance per subnet
	#[pallet::storage]
	pub type MinStakeBalance<T: Config> = StorageValue<_, u128, ValueQuery, DefaultMinStakeBalance>;

	//
	// Delegate Staking
	// 

	/// The minimum delegate stake percentage for a subnet to have at any given time
	// Based on the minimum subnet stake balance based on minimum nodes for a subnet
	// i.e. if a subnet has a minimum node required of 10 and 100 TENSOR per node, and a MinSubnetDelegateStakePercentage of 100%
	//			then the minimum delegate stake balance towards a subnet must be 1000 TENSOR
	#[pallet::storage]
	pub type MinSubnetDelegateStakePercentage<T: Config> = StorageValue<_, u128, ValueQuery, DefaultMinSubnetDelegateStakePercentage>;

	/// The absolute minimum delegate stake balance required for a subnet to stay activated
	#[pallet::storage]
	pub type MinSubnetDelegateStake<T: Config> = StorageValue<_, u128, ValueQuery, DefaultMinSubnetDelegateStake>;
	
	#[pallet::storage]
	pub type MaxDelegateStakeBalance<T: Config> = StorageValue<_, u128, ValueQuery, DefaultMaxDelegateStakeBalance>;

	/// The required blocks between delegate stake transfers
	#[pallet::storage]
	pub type DelegateStakeTransferPeriod<T: Config> = StorageValue<_, u64, ValueQuery, DefaultDelegateStakeTransferPeriod>;

	#[pallet::storage]
	pub type LastDelegateStakeTransfer<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, u64, ValueQuery, DefaultZeroU64>;

	// Percentage of epoch rewards that go towards delegate stake pools
	#[pallet::storage]
	pub type DelegateStakeRewardsPercentage<T: Config> = StorageValue<_, u128, ValueQuery, DefaultDelegateStakeRewardsPercentage>;

	// Total stake sum of all nodes in specified subnet
	#[pallet::storage] // subnet_uid --> peer_data
	pub type TotalSubnetDelegateStakeShares<T: Config> =
		StorageMap<_, Blake2_128Concat, u32, u128, ValueQuery>;

	// Total stake sum of all nodes in specified subnet
	#[pallet::storage] // subnet_uid --> peer_data
	pub type TotalSubnetDelegateStakeBalance<T: Config> =
		StorageMap<_, Blake2_128Concat, u32, u128, ValueQuery>;

	// An accounts delegate stake per subnet
	#[pallet::storage] // account --> subnet_id --> u128
	pub type AccountSubnetDelegateStakeShares<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		T::AccountId,
		Identity,
		u32,
		u128,
		ValueQuery,
		DefaultAccountTake,
	>;

	#[pallet::storage] // account --> subnet_id --> u64
	pub type DelegateStakeCooldown<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		T::AccountId,
		Identity,
		u32,
		u64,
		ValueQuery,
		DefaultDelegateStakeCooldown,
	>;

	#[pallet::storage]
	pub type DelegateStakeUnbondingLedger<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		T::AccountId,
		Identity,
		u32,
		BTreeMap<u64, u128>,
		ValueQuery,
		DefaultDelegateStakeUnbondingLedger,
	>;
	
	//
	// Node Delegate Stake
	//

	#[pallet::storage]
	pub type RewardRateUpdatePeriod<T: Config> = StorageValue<_, u64, ValueQuery, DefaultRewardRateUpdatePeriod>;

	#[pallet::storage]
	pub type MaxRewardRateDecrease<T: Config> = StorageValue<_, u128, ValueQuery, DefaultMaxRewardRateDecrease>;

	// Total stake sum of shares in specified subnet node
	#[pallet::storage]
	pub type TotalNodeDelegateStakeShares<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		u32,
		Identity,
		u32,
		u128,
		ValueQuery,
		DefaultAccountTake,
	>;

	// Total stake sum of balance in specified subnet node
	#[pallet::storage]
	pub type TotalNodeDelegateStakeBalance<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		u32,
		Identity,
		u32,
		u128,
		ValueQuery,
		DefaultAccountTake,
	>;
	
	// account_id -> subnet_id -> subnet_node_id -> shares
	#[pallet::storage] 
	pub type AccountNodeDelegateStakeShares<T: Config> = StorageNMap<
		_,
		(
			NMapKey<Blake2_128Concat, T::AccountId>,
			NMapKey<Identity, u32>,
			NMapKey<Identity, u32>,
		),
		u128,
		ValueQuery,
	>;

	//
	// Props
	//
	#[pallet::type_value]
	pub fn DefaultProposalParams<T: Config>() -> ProposalParams {
		return ProposalParams {
			subnet_id: 0,
			plaintiff_id: 0,
			defendant_id: 0,
			plaintiff_bond: 0,
			defendant_bond: 0,
			eligible_voters: BTreeSet::new(),
			votes: VoteParams {
				yay: BTreeSet::new(),
				nay: BTreeSet::new(),
			},
			start_block: 0,
			challenge_block: 0,
			plaintiff_data: Vec::new(),
			defendant_data: Vec::new(),
			complete: false,
		};
	}

	#[pallet::storage] // subnet => proposal_id => proposal
	pub type Proposals<T> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		u32,
		Identity,
		u32,
		ProposalParams,
		ValueQuery,
		DefaultProposalParams<T>,
	>;
	
	#[pallet::type_value]
	pub fn DefaultProposalMinSubnetNodes() -> u32 {
		16
	}

	/// The minimum subnet nodes for a subnet to have to be able to use the proposal mechanism
	// Because of slashing of funds is possible, we ensure the subnet is well decentralized
	// If a subnet is under this amount, it's best to have logic in the subnet to have them absent
	// from the incentives consensus data and have them removed after the required consecutive epochs
	#[pallet::storage] 
	pub type ProposalMinSubnetNodes<T> = StorageValue<_, u32, ValueQuery, DefaultProposalMinSubnetNodes>;

	#[pallet::type_value]
	pub fn DefaultProposalsCount() -> u32 {
		0
	}

	#[pallet::storage] 
	pub type ProposalsCount<T> = StorageValue<_, u32, ValueQuery, DefaultProposalsCount>;

	#[pallet::type_value]
	pub fn DefaultProposalBidAmount() -> u128 {
		1e+18 as u128
	}

	// Amount required to put up as a proposer and challenger
	#[pallet::storage] 
	pub type ProposalBidAmount<T> = StorageValue<_, u128, ValueQuery, DefaultProposalBidAmount>;

	#[pallet::type_value]
	pub fn DefaultVotingPeriod() -> u64 {
		// 7 days
		100800
	}

	#[pallet::storage] // Period in blocks for votes after challenge
	pub type VotingPeriod<T> = StorageValue<_, u64, ValueQuery, DefaultVotingPeriod>;

	#[pallet::type_value]
	pub fn DefaultChallengePeriod() -> u64 {
		// 7 days in blocks
		100800
	}

	#[pallet::storage] // Period in blocks after proposal to challenge proposal
	pub type ChallengePeriod<T> = StorageValue<_, u64, ValueQuery, DefaultChallengePeriod>;

	#[pallet::type_value]
	pub fn DefaultProposalQuorum() -> u128 {
		// 75.0%
		750000000
	}

	#[pallet::storage] // How many voters are needed in a subnet proposal
	pub type ProposalQuorum<T> = StorageValue<_, u128, ValueQuery, DefaultProposalQuorum>;

	#[pallet::type_value]
	pub fn DefaultProposalConsensusThreshold() -> u128 {
		// 66.0%
		660000000
	}

	// Consensus required to pass proposal
	#[pallet::storage]
	pub type ProposalConsensusThreshold<T> = StorageValue<_, u128, ValueQuery, DefaultProposalConsensusThreshold>;

	// Lower bound of registration fee
	#[pallet::storage]
	pub type MinSubnetRegistrationFee<T> = StorageValue<_, u128, ValueQuery, DefaultMinSubnetRegistrationFee>;

	// Upper bound of registration fee
	#[pallet::storage]
	pub type MaxSubnetRegistrationFee<T> = StorageValue<_, u128, ValueQuery, DefaultMaxSubnetRegistrationFee>;

	// Last epoch a subnet was registered
	#[pallet::storage]
	pub type LastSubnetRegistrationEpoch<T> = StorageValue<_, u32, ValueQuery, DefaultZeroU32>;

	// Epochs per subnet registration
	// Also used for calculating the fee between the max and min registration fee
	// e.g. Amount of epochs required to go by after a subnet registers before another can
	#[pallet::storage]
	pub type SubnetRegistrationInterval<T> = StorageValue<_, u32, ValueQuery, DefaultSubnetRegistrationInterval>;

	/// The pallet's dispatchable functions ([`Call`]s).
	///
	/// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	/// These functions materialize as "extrinsics", which are often compared to transactions.
	/// They must always return a `DispatchResult` and be annotated with a weight and call index.
	///
	/// The [`call_index`] macro is used to explicitly
	/// define an index for calls in the [`Call`] enum. This is useful for pallets that may
	/// introduce new dispatchables over time. If the order of a dispatchable changes, its index
	/// will also change which will break backwards compatibility.
	///
	/// The [`weight`] macro is used to assign a weight to each call.
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Register a new subnet.
		///
		/// # Arguments
		///
		/// * `subnet_data` - Subnet registration data `RegistrationSubnetData`.
		///
		#[pallet::call_index(0)]
		#[pallet::weight({0})]
		pub fn register_subnet(
			origin: OriginFor<T>, 
			subnet_data: RegistrationSubnetData,
		) -> DispatchResult {
			let account_id: T::AccountId = ensure_signed(origin)?;
	
			Self::do_register_subnet(
				account_id,
				subnet_data,
			)
		}

		/// Try activation a registered subnet.
		///
		/// # Arguments
		///
		/// * `subnet_id` - Subnet ID assigned on registration.
		///
		#[pallet::call_index(1)]
		#[pallet::weight({0})]
		pub fn activate_subnet(
			origin: OriginFor<T>, 
			subnet_id: u32,
		) -> DispatchResult {
			let account_id: T::AccountId = ensure_signed(origin)?;
	
			Self::do_activate_subnet(subnet_id)
		}

		/// Try removing a subnet.
		///
		/// # Arguments
		///
		/// * `subnet_id` - Subnet ID.
		///
		/// # Requirements
		/// 
		/// * `SubnetPenaltyCount` surpasses `MaxSubnetPenaltyCount`
		/// * Subnet delegate stake balance is below the required balance
		/// 
		#[pallet::call_index(2)]
		#[pallet::weight({0})]
		pub fn remove_subnet(
			origin: OriginFor<T>, 
			subnet_id: u32,
		) -> DispatchResult {
			ensure_signed(origin)?;

			let subnet = match SubnetsData::<T>::try_get(subnet_id) {
        Ok(subnet) => subnet,
        Err(()) => return Err(Error::<T>::SubnetNotExist.into()),
			};
			
			// --- Ensure the subnet has passed it's required period to begin consensus submissions
			ensure!(
				subnet.activated != 0,
				Error::<T>::SubnetInitializing
			);

			let penalties = SubnetPenaltyCount::<T>::get(subnet_id);

			let subnet_delegate_stake_balance = TotalSubnetDelegateStakeBalance::<T>::get(subnet_id);
			let min_subnet_delegate_stake_balance = Self::get_min_subnet_delegate_stake_balance(subnet.min_nodes);

			if penalties > MaxSubnetPenaltyCount::<T>::get() {
				// --- If the subnet has reached max penalty, remove it
        Self::deactivate_subnet(
          subnet.path,
          SubnetRemovalReason::MaxPenalties,
        ).map_err(|e| e)?;
			} else if subnet_delegate_stake_balance < min_subnet_delegate_stake_balance {
				// --- If the delegate stake balance is below minimum threshold, remove it
        Self::deactivate_subnet(
          subnet.path,
          SubnetRemovalReason::MinSubnetDelegateStake,
        ).map_err(|e| e)?;
			}

			// --- If we make it to here, fail the extrinsic
			Err(Error::<T>::InvalidSubnetRemoval.into())
		}

		/// Add a subnet node to the subnet by registering and activating in one call
		///
		/// The subnet node will be assigned a class (`SubnetNodeClass`)
		/// * If the subnet is in its registration period it will be assigned the Validator class
		/// * If the subnet is active, it will be assigned as `Registered` and must be inducted by consensus
		/// - See `SubnetNodeClass` for more information on class levels
		///
		/// # Arguments
		///
		/// * `subnet_id` - Subnet ID.
		/// * `hotkey` - Hotkey of the subnet node.
		/// * `peer_id` - The Peer ID of the subnet node within the subnet P2P network.
		/// * `stake_to_be_added` - The balance to add to stake.
		/// * `a` - A subnet node parameter unique to each subnet.
		/// * `b` - A non-unique parameter.
		/// * `c` - A non-unique parameter.
		///
		/// # Requirements
		/// 
		/// * `stake_to_be_added` must be the minimum required stake balance
		/// 
		#[pallet::call_index(3)]
		// #[pallet::weight(T::WeightInfo::add_subnet_node())]
		#[pallet::weight({0})]
		pub fn add_subnet_node(
			origin: OriginFor<T>, 
			subnet_id: u32, 
			hotkey: T::AccountId,
			peer_id: PeerId, 
			delegate_reward_rate: u128,
			stake_to_be_added: u128,
			a: Option<BoundedVec<u8, DefaultSubnetNodeUniqueParamLimit>>,
			b: Option<BoundedVec<u8, DefaultSubnetNodeUniqueParamLimit>>,
			c: Option<BoundedVec<u8, DefaultSubnetNodeUniqueParamLimit>>,
		) -> DispatchResult {
			Self::do_register_subnet_node(
				origin.clone(),
				subnet_id,
				hotkey.clone(),
				peer_id,
				delegate_reward_rate,
				stake_to_be_added,
				a,
				b,
				c,
			).map_err(|e| e)?;

			let subnet_node_id = HotkeySubnetNodeId::<T>::get(subnet_id, hotkey.clone());

			// This is redundant to check, but check we do
			match HotkeySubnetNodeId::<T>::try_get(subnet_id, hotkey.clone()) {
				Ok(subnet_node_id) => Self::do_activate_subnet_node(
					origin.clone(),
					subnet_id,
					subnet_node_id
				),
				Err(()) => return Err(Error::<T>::NotUidOwner.into()),
			}
		}

		/// Register a subnet node to the subnet
		///
		/// A registered subnet node will not be included in consensus data, therefor no incentives until
		/// the subnet node activates itself (see `activate_subnet_node`)
		///
		/// Subnet nodes register by staking the minimum required balance to pass authentication in any P2P
		/// networks, such as a subnet.
		///
		/// # Arguments
		///
		/// * `subnet_id` - Subnet ID.
		/// * `hotkey` - Hotkey of the subnet node.
		/// * `peer_id` - The Peer ID of the subnet node within the subnet P2P network.
		/// * `stake_to_be_added` - The balance to add to stake.
		/// * `a` - A subnet node parameter unique to each subnet.
		/// * `b` - A non-unique parameter.
		/// * `c` - A non-unique parameter.
		///
		/// # Requirements
		/// 
		/// * `stake_to_be_added` must be the minimum required stake balance
		/// 
		#[pallet::call_index(4)]
		#[pallet::weight({0})]
		pub fn register_subnet_node(
			origin: OriginFor<T>, 
			subnet_id: u32, 
			hotkey: T::AccountId,
			peer_id: PeerId, 
			delegate_reward_rate: u128,
			stake_to_be_added: u128,
			a: Option<BoundedVec<u8, DefaultSubnetNodeUniqueParamLimit>>,
			b: Option<BoundedVec<u8, DefaultSubnetNodeUniqueParamLimit>>,
			c: Option<BoundedVec<u8, DefaultSubnetNodeUniqueParamLimit>>,
		) -> DispatchResult {
			Self::do_register_subnet_node(
				origin,
				subnet_id,
				hotkey,
				peer_id,
				delegate_reward_rate,
				stake_to_be_added,
				a,
				b,
				c,
			)
		}
		
		/// Activate a subnet node
		///
		/// Subnet nodes should activate their subnet node once they are in the subnet and have completed any
		/// steps the subnet requires, such as any consensus mechanisms.
		///
		/// # Arguments
		///
		/// * `subnet_id` - Subnet ID.
		/// * `subnet_node_id` - Subnet node ID assigned during registration
		/// 
		#[pallet::call_index(5)]
		#[pallet::weight({0})]
		pub fn activate_subnet_node(
			origin: OriginFor<T>, 
			subnet_id: u32, 
			subnet_node_id: u32,
		) -> DispatchResult {
			Self::do_activate_subnet_node(
				origin,
				subnet_id,
				subnet_node_id
			)
		}	

		/// Deactivate a subnet node temporarily
		///
		/// A subnet node can deactivate themselves temporarily. This can be to make updates to the node, etc.
		///
		/// Deactivation will set the subnet nodes class to `Deactivated`. The subnet node will have up to the
		/// `SubnetNodeRegistrationEpochs` to re-activate or be removed after this period of time on the first
		/// successfully attested epoch.
		///
		/// * If the subnet node has attested the current epochs consensus data, they will be added to the 
		///	  `DeactivationLedger` and deactivated on the following epoch to avoid missing out on the emissions.
		/// * * Otherwise, they are deactivated immediately.
		///
		/// # Arguments
		///
		/// * `subnet_id` - Subnet ID.
		/// * `subnet_node_id` - Subnet node ID assigned during registration
		///
		/// # Requirements
		/// 
		/// * Must be a Validator class to deactivate, otherwise the subnet node must
		/// 
		#[pallet::call_index(6)]
		#[pallet::weight({0})]
		pub fn deactivate_subnet_node(
			origin: OriginFor<T>, 
			subnet_id: u32, 
			subnet_node_id: u32,
		) -> DispatchResult {
			Self::do_deactivate_subnet_node(
				origin,
				subnet_id,
				subnet_node_id
			)
		}
		
		/// Remove subnet node of caller
		///
		/// # Arguments
		///
		/// * `subnet_id` - Subnet ID.
		/// * `subnet_node_id` - Subnet node ID assigned during registration
		///
		/// # Requirements
		///
		/// * Caller must be owner of subnet node, hotkey or coldkey
		///
		#[pallet::call_index(7)]
		// #[pallet::weight(T::WeightInfo::remove_subnet_node())]
		#[pallet::weight({0})]
		pub fn remove_subnet_node(
			origin: OriginFor<T>, 
			subnet_id: u32, 
			subnet_node_id: u32,
		) -> DispatchResult {
			let key: T::AccountId = ensure_signed(origin)?;

			ensure!(
				Self::is_keys_owner(
					subnet_id, 
					subnet_node_id, 
					key, 
				),
				Error::<T>::NotKeyOwner
			);

			Self::do_remove_subnet_node(subnet_id, subnet_node_id)
		}

		/// Remove subnet node of caller
		///
		/// # Arguments
		///
		/// * `subnet_id` - Subnet ID.
		/// * `subnet_node_id` - Subnet node ID.
		/// * `new_delegate_reward_rate` - New delegate reward rate.
		///
		/// # Requirements
		///
		/// * Caller must be coldkey owner of subnet node ID.
		/// * If decreasing rate, new rate must not be more than a 1% decrease nominally (10_000_000 using 1e9)
		///
		#[pallet::call_index(8)]
		#[pallet::weight({0})]
		pub fn update_delegate_reward_rate(
			origin: OriginFor<T>, 
			subnet_id: u32,
			subnet_node_id: u32,
			new_delegate_reward_rate: u128
		) -> DispatchResult {
			let coldkey: T::AccountId = ensure_signed(origin)?;

			ensure!(
				Self::is_subnet_node_coldkey(
					subnet_id, 
					subnet_node_id, 
					coldkey, 
				),
				Error::<T>::NotKeyOwner
			);

			// --- Ensure rate doesn't surpass 100%
			ensure!(
				new_delegate_reward_rate <= Self::PERCENTAGE_FACTOR,
				Error::<T>::InvalidDelegateRewardRate
			);

			let block: u64 = Self::get_current_block_as_u64();
			let max_reward_rate_decrease = MaxRewardRateDecrease::<T>::get();
			let reward_rate_update_period = RewardRateUpdatePeriod::<T>::get();

			SubnetNodesData::<T>::try_mutate_exists(
				subnet_id,
				subnet_node_id,
				|maybe_params| -> DispatchResult {
					let params = maybe_params.as_mut().ok_or(Error::<T>::SubnetNodeExist)?;
					let mut curr_delegate_reward_rate = params.delegate_reward_rate;

					// --- Ensure rate change surpasses minimum update period
					ensure!(
						block - params.last_delegate_reward_rate_update >= reward_rate_update_period,
						Error::<T>::MaxRewardRateUpdates
					);
					
					// --- Ensure rate is being updated
					ensure!(
						new_delegate_reward_rate != curr_delegate_reward_rate,
						Error::<T>::NoDelegateRewardRateChange
					);

					let mut delegate_reward_rate = params.delegate_reward_rate;

					if new_delegate_reward_rate > curr_delegate_reward_rate {
						// Freely increase reward rate
						delegate_reward_rate = new_delegate_reward_rate;
					} else {
						// Ensure reward rate decrease doesn't surpass max rate of change
						let delta = curr_delegate_reward_rate - new_delegate_reward_rate;
						ensure!(
							delta <= max_reward_rate_decrease,
							Error::<T>::SurpassesMaxRewardRateDecrease
						);
						delegate_reward_rate = new_delegate_reward_rate
					}

					params.last_delegate_reward_rate_update = block;
					params.delegate_reward_rate = delegate_reward_rate;
					Ok(())
				}
			)?;

			Ok(())
		}

		/// Add to subnet node stake
		///
		/// # Arguments
		///
		/// * `subnet_id` - Subnet ID.
		/// * `subnet_node_id` - Subnet node ID assigned during registration
		/// * `hotkey` - Hotkey of subnet node
		/// * `stake_to_be_added` - Amount to add to stake
		///
		/// # Requirements
		///
		/// * Coldkey caller only
		/// * Subnet must exist
		/// * Must have amount free in wallet
		///
		#[pallet::call_index(9)]
		// #[pallet::weight(T::WeightInfo::add_to_stake())]
		#[pallet::weight({0})]
		pub fn add_to_stake(
			origin: OriginFor<T>, 
			subnet_id: u32,
			subnet_node_id: u32,
			hotkey: T::AccountId,
			stake_to_be_added: u128,
		) -> DispatchResult {
			let coldkey: T::AccountId = ensure_signed(origin.clone())?;
			// Each account can only have one peer
			// Staking is accounted for per account_id per subnet_id
			// We only check that origin exists within SubnetNodesData

			// --- Ensure subnet exists to add to stake
			ensure!(
				SubnetsData::<T>::contains_key(subnet_id),
				Error::<T>::SubnetNotExist
			);

			// --- Ensure coldkey owns the hotkey
			ensure!(
				HotkeyOwner::<T>::get(hotkey.clone()) == coldkey,
				Error::<T>::NotKeyOwner
			);

			// --- Ensure hotkey owns the subnet_node_id
			ensure!(
				Self::is_subnet_node_owner(subnet_id, subnet_node_id, hotkey.clone()),
				Error::<T>::NotSubnetNodeOwner
			);
						
			Self::do_add_stake(
				origin, 
				subnet_id,
				hotkey,
				stake_to_be_added,
			)
		}

		/// Remove from subnet node stake and add to unstaking ledger
		///
		/// # Arguments
		///
		/// * `subnet_id` - Subnet ID.
		/// * `hotkey` - Hotkey of subnet node
		/// * `stake_to_be_removed` - Amount to remove from stake
		///
		/// # Requirements
		///
		/// * Coldkey caller only
		/// * If subnet node, must have available staked balance greater than minimum required stake balance
		///
		#[pallet::call_index(10)]
		#[pallet::weight({0})]
		pub fn remove_stake(
			origin: OriginFor<T>, 
			subnet_id: u32, 
			hotkey: T::AccountId,
			stake_to_be_removed: u128
		) -> DispatchResult {
			let coldkey: T::AccountId = ensure_signed(origin.clone())?;

			// --- Ensure the hotkey stake owner is owned by the caller
			ensure!(
				HotkeyOwner::<T>::get(hotkey.clone()) == coldkey,
				Error::<T>::NotKeyOwner
			);

			// If account is a subnet node they can remove stake up to minimum required stake balance
			// Else they can remove entire balance because they are not hosting subnets according to consensus
			//		They are removed in `do_remove_subnet_node()` when self or consensus removed
			let is_subnet_node: bool = match HotkeySubnetNodeId::<T>::try_get(subnet_id, hotkey.clone()) {
				Ok(_) => true,
				Err(()) => false,
			};

			// Remove stake
			// 		is_subnet_node: cannot remove stake below minimum required stake
			// 		else: can remove total stake balance
			Self::do_remove_stake(
				origin, 
				subnet_id,
				hotkey,
				is_subnet_node,
				stake_to_be_removed,
			)
		}

		/// Transfer unstaking ledger balance to coldkey
		///
		/// # Requirements
		///
		/// * Coldkey caller only
		/// * Must be owner of stake balance
		///
		#[pallet::call_index(11)]
		#[pallet::weight({0})]
		pub fn claim_unbondings(
			origin: OriginFor<T>, 
		) -> DispatchResult {
			let coldkey: T::AccountId = ensure_signed(origin)?;
			let successful_unbondings: u32 = Self::do_claim_unbondings(&coldkey);
			// Give error if there is no unbondings
			// TODO: Give more info
			ensure!(
				successful_unbondings > 0,
        Error::<T>::NoStakeUnbondingsOrCooldownNotMet
			);
			Ok(())
		}

		/// Increase subnet delegate stake
		///
		/// # Arguments
		///
		/// * `subnet_id` - Subnet ID.
		/// * `stake_to_be_added` - Amount of add to delegate stake
		///
		/// # Requirements
		///
		/// * Subnet must exist
		///
		#[pallet::call_index(12)]
		// #[pallet::weight(T::WeightInfo::add_to_delegate_stake())]
		#[pallet::weight({0})]
		pub fn add_to_delegate_stake(
			origin: OriginFor<T>, 
			subnet_id: u32,
			stake_to_be_added: u128,
		) -> DispatchResult {
			let account_id: T::AccountId = ensure_signed(origin.clone())?;

			// --- Ensure subnet exists
			ensure!(
				SubnetsData::<T>::contains_key(subnet_id),
				Error::<T>::SubnetNotExist
			);

			Self::do_add_delegate_stake(
				origin, 
				subnet_id,
				stake_to_be_added,
			)
		}
		
		/// Increase subnet delegate stake
		///
		/// * Swaps delegate stake from one subnet to another subnet in one call
		///
		/// # Arguments
		///
		/// * `from_subnet_id` - from subnet ID.
		/// * `to_subnet_id` - To subnet ID
		/// * `delegate_stake_shares_to_be_switched` - Shares of `from_subnet_id` to swap to `to_subnet_id`
		///
		/// # Requirements
		///
		/// * `to_subnet_id` subnet must exist
		///
		#[pallet::call_index(13)]
		#[pallet::weight({0})]
		pub fn transfer_delegate_stake(
			origin: OriginFor<T>, 
			from_subnet_id: u32, 
			to_subnet_id: u32, 
			delegate_stake_shares_to_be_switched: u128
		) -> DispatchResult {
			// --- Ensure ``to`` subnet exists
			ensure!(
				SubnetsData::<T>::contains_key(to_subnet_id),
				Error::<T>::SubnetNotExist
			);

			// TODO: Ensure users aren't hopping from one subnet to another to get both rewards
			// Check if both subnets have generated rewards
			// --- Only allow one ``transfer_delegate_stake`` per epoch or every other epoch

			// Handles ``ensure_signed``
			Self::do_switch_delegate_stake(
				origin, 
				from_subnet_id,
				to_subnet_id,
				delegate_stake_shares_to_be_switched,
			)
		}

		/// Remove subnet delegate stake balance and add to unstaking ledger.
		///
		/// # Arguments
		///
		/// * `subnet_id` - Subnet ID.
		/// * `shares_to_be_removed` - Shares to remove
		///
		/// # Requirements
		///
		/// * Must have balance
		///
		#[pallet::call_index(14)]
		// #[pallet::weight(T::WeightInfo::remove_delegate_stake())]
		#[pallet::weight({0})]
		pub fn remove_delegate_stake(
			origin: OriginFor<T>, 
			subnet_id: u32, 
			shares_to_be_removed: u128
		) -> DispatchResult {
			Self::do_remove_delegate_stake(
				origin, 
				subnet_id,
				shares_to_be_removed,
			)
		}
		
		/// Increase the delegate stake pool balance of a subnet
		///
		/// * Anyone can perform this action as a donation
		///
		/// # Notes
		///
		/// *** THIS DOES ''NOT'' INCREASE A USERS BALANCE ***
		///
		/// # Arguments
		///
		/// * `subnet_id` - Subnet ID to increase delegate pool balance of.
		/// * `amount` - Amount TENSOR to add to pool
		///
		#[pallet::call_index(15)]
		#[pallet::weight({0})]
		pub fn increase_delegate_stake(
			origin: OriginFor<T>, 
			subnet_id: u32,
			amount: u128,
		) -> DispatchResult {
			let account_id: T::AccountId = ensure_signed(origin)?;
			
			// --- Ensure subnet exists, otherwise at risk of burning tokens
			ensure!(
				SubnetsData::<T>::contains_key(subnet_id),
				Error::<T>::SubnetNotExist
			);

			let amount_as_balance = Self::u128_to_balance(amount);

			ensure!(
				amount_as_balance.is_some(),
				Error::<T>::CouldNotConvertToBalance
			);
	
			// --- Ensure the callers account_id has enough balance to perform the transaction.
			ensure!(
				Self::can_remove_balance_from_coldkey_account(&account_id, amount_as_balance.unwrap()),
				Error::<T>::NotEnoughBalance
			);
	
			// --- Ensure the remove operation from the account_id is a success.
			ensure!(
				Self::remove_balance_from_coldkey_account(&account_id, amount_as_balance.unwrap()) == true,
				Error::<T>::BalanceWithdrawalError
			);
			
			Self::do_increase_delegate_stake(
				subnet_id,
				amount,
			);

			Ok(())
		}

		/// Delegate stake to a subnet node
		///
		/// # Arguments
		///
		/// * `subnet_id` - Subnet ID
		/// * `node_account_id` - Subnet node ID
		/// * `node_delegate_stake_to_be_added` - Amount TENSOR to delegate stake
		///
		#[pallet::call_index(16)]
		#[pallet::weight({0})]
		pub fn add_to_node_delegate_stake(
			origin: OriginFor<T>, 
			subnet_id: u32,
			subnet_node_id: u32,
			node_delegate_stake_to_be_added: u128
		) -> DispatchResult {
			Self::do_add_node_delegate_stake(
				origin,
				subnet_id,
				subnet_node_id,
				node_delegate_stake_to_be_added,
			)
		}

		/// Increase subnet delegate stake
		///
		/// * Swaps delegate stake from one subnet to another subnet in one call
		///
		/// # Arguments
		///
		/// * `from_subnet_id` - from subnet ID.
		/// * `to_subnet_id` - To subnet ID
		/// * `delegate_stake_shares_to_be_switched` - Shares of `from_subnet_id` to swap to `to_subnet_id`
		///
		/// # Requirements
		///
		/// * `to_subnet_id` subnet must exist
		///
		#[pallet::call_index(17)]
		#[pallet::weight({0})]
		pub fn transfer_node_delegate_stake(
			origin: OriginFor<T>, 
			from_subnet_id: u32,
			from_subnet_node_id: u32, 
			to_subnet_id: u32, 
			to_subnet_node_id: u32, 
			node_delegate_stake_shares_to_be_switched: u128
		) -> DispatchResult {
			// --- Ensure ``to`` subnet node exists
			ensure!(
				SubnetNodesData::<T>::contains_key(to_subnet_id, to_subnet_node_id),
				Error::<T>::SubnetNodeNotExist
			);

			Self::do_switch_node_delegate_stake(
				origin,
				from_subnet_id,
				from_subnet_node_id,
				to_subnet_id,
				to_subnet_node_id,
				node_delegate_stake_shares_to_be_switched,
			)
		}

		/// Remove delegate stake from a subnet node and add to unbonding ledger.
		///
		/// # Arguments
		///
		/// * `subnet_id` - Subnet ID
		/// * `node_account_id` - Subnet node ID
		/// * `node_delegate_stake_shares_to_be_removed` - Pool shares to remove
		///
		#[pallet::call_index(18)]
		#[pallet::weight({0})]
		pub fn remove_node_delegate_stake(
			origin: OriginFor<T>, 
			subnet_id: u32,
			subnet_node_id: u32,
			node_delegate_stake_shares_to_be_removed: u128
		) -> DispatchResult {
			Self::do_remove_node_delegate_stake(
				origin,
				subnet_id,
				subnet_node_id,
				node_delegate_stake_shares_to_be_removed,
			)
		}

		/// Increase the node delegate stake pool balance of a subnet node
		///
		/// * Anyone can perform this action as a donation
		///
		/// # Notes
		///
		/// *** THIS DOES ''NOT'' INCREASE A USERS BALANCE ***
		///
		/// # Arguments
		///
		/// * `subnet_id` - Subnet ID to increase delegate pool balance of.
		/// * `subnet_node_id` - Subnet node ID.
		/// * `amount` - Amount TENSOR to add to pool
		///
		#[pallet::call_index(19)]
		#[pallet::weight({0})]
		pub fn increase_node_delegate_stake(
			origin: OriginFor<T>, 
			subnet_id: u32,
			subnet_node_id: u32,
			amount: u128,
		) -> DispatchResult {
			let account_id: T::AccountId = ensure_signed(origin)?;
			
			// --- Ensure subnet node exists, otherwise at risk of burning tokens
			ensure!(
				SubnetNodesData::<T>::contains_key(subnet_id, subnet_node_id),
				Error::<T>::SubnetNodeNotExist
			);
			
			let amount_as_balance = Self::u128_to_balance(amount);

			ensure!(
				amount_as_balance.is_some(),
				Error::<T>::CouldNotConvertToBalance
			);
	
			// --- Ensure the callers account_id has enough balance to perform the transaction.
			ensure!(
				Self::can_remove_balance_from_coldkey_account(&account_id, amount_as_balance.unwrap()),
				Error::<T>::NotEnoughBalance
			);
	
			// --- Ensure the remove operation from the account_id is a success.
			ensure!(
				Self::remove_balance_from_coldkey_account(&account_id, amount_as_balance.unwrap()) == true,
				Error::<T>::BalanceWithdrawalError
			);
			
			Self::do_increase_node_delegate_stake(
				subnet_id,
				subnet_node_id,
				amount,
			);

			Ok(())
		}

		/// Validator extrinsic for submitting incentives protocol data of the validators view of of the subnet
		/// This is used t oscore each subnet node for allocation of emissions
		///
		/// # Arguments
		///
		/// * `subnet_id` - Subnet ID to increase delegate pool balance of.
		/// * `data` - Vector of SubnetNodeData on each subnet node for scoring each
		/// * `args` (Optional) - Data that can be used by the subnet 
		/// 
		#[pallet::call_index(20)]
		#[pallet::weight({0})]
		pub fn validate(
			origin: OriginFor<T>, 
			subnet_id: u32,
			data: Vec<SubnetNodeData>,
			args: Option<BoundedVec<u8, DefaultValidatorArgsLimit>>,
		) -> DispatchResultWithPostInfo {
			let hotkey: T::AccountId = ensure_signed(origin)?;

			let block: u64 = Self::get_current_block_as_u64();
			let epoch_length: u64 = T::EpochLength::get();
			let epoch: u64 = block / epoch_length;

			Self::do_validate(
				subnet_id, 
				hotkey,
				block,
				epoch_length,
				epoch as u32,
				data,
				args,
			)
		}

		/// Attest validators view of the subnet
		///
		/// # Arguments
		///
		/// * `subnet_id` - Subnet ID to increase delegate pool balance of.
		/// 
		#[pallet::call_index(21)]
		#[pallet::weight({0})]
		pub fn attest(
			origin: OriginFor<T>, 
			subnet_id: u32,
		) -> DispatchResultWithPostInfo {
			let hotkey: T::AccountId = ensure_signed(origin)?;

			let block: u64 = Self::get_current_block_as_u64();
			let epoch_length: u64 = T::EpochLength::get();
			let epoch: u64 = block / epoch_length;

			Self::do_attest(
				subnet_id, 
				hotkey,
				block, 
				epoch_length,
				epoch as u32,
			)
		}

		/// Propose to remove someone from subnet
		///
		/// This acts as a governance system for each subnet
		///
		/// Each proposal requires a bond of TENSOR
		///
		/// # Arguments
		///
		/// * `subnet_id` - Subnet ID.
		/// * `subnet_node_id` - The proposers subnet node ID
		/// * `peer_id` - The defendants subnet node peer ID
		/// * `data` - Data used to justify dispute for subnet use
		/// 
		#[pallet::call_index(22)]
		#[pallet::weight({0})]
		pub fn propose(
			origin: OriginFor<T>, 
			subnet_id: u32,
			subnet_node_id: u32,
			peer_id: PeerId,
			data: Vec<u8>,
		) -> DispatchResult {
			let account_id: T::AccountId = ensure_signed(origin)?;
	
			Self::do_propose(
				account_id,
				subnet_id,
				subnet_node_id,
				peer_id,
				data
			)
		}

		/// * INCOMPLETE *
		/// Attest a proposal
		///
		/// This acts as a governance system for each subnet
		///
		/// Attest requires a bond of TENSOR
		///
		/// # Arguments
		///
		/// * `subnet_id` - Subnet ID.
		/// * `subnet_node_id` - The proposers subnet node ID
		/// * `peer_id` - The defendants subnet node peer ID
		/// * `data` - Data used to justify dispute for subnet use
		/// 
		#[pallet::call_index(23)]
		#[pallet::weight({0})]
		pub fn attest_proposal(
			origin: OriginFor<T>, 
			subnet_id: u32,
			subnet_node_id: u32,
			peer_id: PeerId,
			data: Vec<u8>,
		) -> DispatchResult {
			let account_id: T::AccountId = ensure_signed(origin)?;
	
			Self::do_propose(
				account_id,
				subnet_id,
				subnet_node_id,
				peer_id,
				data
			)
		}

		/// Cancel proposal
		///
		/// # Arguments
		///
		/// * `subnet_id` - Subnet ID.
		/// * `subnet_node_id` - The proposers subnet node ID
		/// * `proposal_id` - The proposal ID
		/// 
		#[pallet::call_index(24)]
		#[pallet::weight({0})]
		pub fn cancel_proposal(
			origin: OriginFor<T>, 
			subnet_id: u32,
			subnet_node_id: u32,
			proposal_id: u32,
		) -> DispatchResult {
			let account_id: T::AccountId = ensure_signed(origin)?;
	
			Self::do_cancel_proposal(
				account_id,
				subnet_id,
				subnet_node_id,
				proposal_id,
			)
		}

		/// Challenge proposal as the defendant
		///
		/// # Arguments
		///
		/// * `subnet_id` - Subnet ID.
		/// * `proposal_id` - The proposers subnet node ID
		/// * `data` - Data used to justify challenge for subnet use
		/// 
		#[pallet::call_index(25)]
		#[pallet::weight({0})]
		pub fn challenge_proposal(
			origin: OriginFor<T>, 
			subnet_id: u32,
			proposal_id: u32,
			data: Vec<u8>,
		) -> DispatchResult {
			let account_id: T::AccountId = ensure_signed(origin)?;
	
			Self::do_challenge_proposal(
				account_id,
				subnet_id,
				proposal_id,
				data
			)
		}

		/// Challenge proposal as the defendant
		///
		/// # Arguments
		///
		/// * `subnet_id` - Subnet ID.
		/// * `subnet_node_id` - The voter subnet node ID
		/// * `proposal_id` - The proposal ID
		/// * `vote` - YAY or NAY
		/// 
		#[pallet::call_index(26)]
		#[pallet::weight({0})]
		pub fn vote(
			origin: OriginFor<T>, 
			subnet_id: u32,
			subnet_node_id: u32,
			proposal_id: u32,
			vote: VoteType
		) -> DispatchResult {
			let account_id: T::AccountId = ensure_signed(origin)?;
	
			Self::do_vote(
				account_id,
				subnet_id,
				subnet_node_id,
				proposal_id,
				vote
			)
		}

		/// Finalize and compute votes to complete the proposal
		///
		/// If quorum and consensus is reached, bonded TENSOR is distributed to participants in favor
		///
		/// # Arguments
		///
		/// * `subnet_id` - Subnet ID.
		/// * `proposal_id` - The proposal ID
		/// 
		#[pallet::call_index(27)]
		#[pallet::weight({0})]
		pub fn finalize_proposal(
			origin: OriginFor<T>, 
			subnet_id: u32,
			proposal_id: u32,
		) -> DispatchResult {
			let account_id: T::AccountId = ensure_signed(origin)?;
	
			Self::do_finalize_proposal(
				account_id,
				subnet_id,
				proposal_id,
			)
		}

		/// Register unique subnet node parameter if not already added
		///
		/// # Arguments
		///
		/// * `subnet_id` - Subnet ID.
		/// * `subnet_node_id` - Callers subnet node ID
		/// * `a` - The unique parameter
		/// 
		#[pallet::call_index(28)]
		#[pallet::weight({0})]
		pub fn register_subnet_node_a_parameter(
			origin: OriginFor<T>, 
			subnet_id: u32,
			subnet_node_id: u32,
			a: BoundedVec<u8, DefaultSubnetNodeUniqueParamLimit>,
		) -> DispatchResult {
			let key: T::AccountId = ensure_signed(origin)?;

			ensure!(
				Self::is_keys_owner(
					subnet_id, 
					subnet_node_id, 
					key, 
				),
				Error::<T>::NotKeyOwner
			);

			ensure!(
				!SubnetNodeUniqueParam::<T>::contains_key(subnet_id, a.clone()),
				Error::<T>::SubnetNodeUniqueParamTaken
			);

			SubnetNodesData::<T>::try_mutate_exists(
				subnet_id,
				subnet_node_id,
				|maybe_params| -> DispatchResult {
					let params = maybe_params.as_mut().ok_or(Error::<T>::SubnetNodeExist)?;
					ensure!(
						params.a.is_none(),
						Error::<T>::SubnetNodeUniqueParamIsSet
					);
					SubnetNodeUniqueParam::<T>::insert(subnet_id, a.clone(), params.peer_id.clone());
					params.a = Some(a.clone());
					Ok(())
				}
			)
		}

		/// Register non-unique subnet node parameter `b` or `c`
		///
		/// # Arguments
		///
		/// * `subnet_id` - Subnet ID.
		/// * `subnet_node_id` - Callers subnet node ID
		/// * `b` (Optional) - The non-unique parameter
		/// * `c` (Optional) - The non-unique parameter
		/// 
		#[pallet::call_index(29)]
		#[pallet::weight({0})]
		pub fn set_subnet_node_non_unique_parameter(
			origin: OriginFor<T>, 
			subnet_id: u32,
			subnet_node_id: u32,
			b: Option<BoundedVec<u8, DefaultSubnetNodeUniqueParamLimit>>,
			c: Option<BoundedVec<u8, DefaultSubnetNodeUniqueParamLimit>>,
		) -> DispatchResult {
			let key: T::AccountId = ensure_signed(origin)?;

			ensure!(
				Self::is_keys_owner(
					subnet_id, 
					subnet_node_id, 
					key, 
				),
				Error::<T>::NotKeyOwner
			);

			let block: u64 = Self::get_current_block_as_u64();
			let epoch_length: u64 = T::EpochLength::get();
			let epoch: u64 = block / epoch_length;

			let last_update_epoch = SubnetNodeNonUniqueParamLastSet::<T>::get(subnet_id, subnet_node_id);
			let interval = SubnetNodeNonUniqueParamUpdateInterval::<T>::get();

			ensure!(
				last_update_epoch + interval <= epoch as u32,
				Error::<T>::SubnetNodeNonUniqueParamUpdateIntervalNotReached
			);

			ensure!(
				b.is_some() || c.is_some(),
				Error::<T>::SubnetNodeNonUniqueParamMustBeSome
			);

			SubnetNodesData::<T>::try_mutate_exists(
				subnet_id,
				subnet_node_id,
				|maybe_params| -> DispatchResult {
					let params = maybe_params.as_mut().ok_or(Error::<T>::SubnetNodeExist)?;

					if b.is_some() {
						params.b = Some(b.clone().unwrap());
					}

					if c.is_some() {
						params.c = Some(c.clone().unwrap());
					}

					SubnetNodeNonUniqueParamLastSet::<T>::insert(subnet_id, subnet_node_id, epoch as u32);

					Ok(())
				}
			)
		}

		/// Update coldkey
		///
		/// # Arguments
		///
		/// * `hotkey` - Current hotkey.
		/// * `new_coldkey` - New coldkey
		/// 
		#[pallet::call_index(30)]
		#[pallet::weight({0})]
		pub fn update_coldkey(
			origin: OriginFor<T>, 
			hotkey: T::AccountId,
			new_coldkey: T::AccountId,
		) -> DispatchResult {
			let curr_coldkey: T::AccountId = ensure_signed(origin)?;

			HotkeyOwner::<T>::try_mutate_exists(hotkey, |maybe_coldkey| -> DispatchResult {
        match maybe_coldkey {
					Some(status) if *status == curr_coldkey => {
						// Condition met, update or remove
						*maybe_coldkey = Some(new_coldkey.clone());
						// Update StakeUnbondingLedger
						StakeUnbondingLedger::<T>::swap(curr_coldkey, new_coldkey);
						Ok(())
					},
					// --- Revert from here if not exist
					Some(_) => {
						Err(Error::<T>::NotKeyOwner.into())
					},
					None => {
						Err(Error::<T>::NotKeyOwner.into())
					}
				}
			})
		}

		/// Update hotkey
		///
		/// # Arguments
		///
		/// * `old_hotkey` - Old hotkey to be replaced.
		/// * `new_hotkey` - New hotkey to replace the old hotkey.
		/// 
		#[pallet::call_index(31)]
		#[pallet::weight({0})]
		pub fn update_hotkey(
			origin: OriginFor<T>, 
			old_hotkey: T::AccountId,
			new_hotkey: T::AccountId,
		) -> DispatchResult {
			let coldkey: T::AccountId = ensure_signed(origin.clone())?;

			// --- Ensure hotkey not taken
			ensure!(
				!HotkeyOwner::<T>::contains_key(new_hotkey.clone()),
				Error::<T>::KeyOwnerTaken
			);

			// Each subnet node hotkey is unique across the entire network
			ensure!(
				HotkeyOwner::<T>::get(old_hotkey.clone()) == coldkey,
				Error::<T>::NotKeyOwner
			);

			HotkeyOwner::<T>::remove(old_hotkey.clone());
			HotkeyOwner::<T>::insert(new_hotkey.clone(), coldkey.clone());

			for (subnet_id, _) in SubnetsData::<T>::iter() {
				let subnet_node_owner: (bool, u32) = match HotkeySubnetNodeId::<T>::try_get(subnet_id, old_hotkey.clone()) {
					Ok(subnet_node_id) => (true, subnet_node_id),
					Err(()) => (false, 0),
				};

				if subnet_node_owner.0 {
					// --- might not be needed here `SubnetNodeIdHotkey`
					SubnetNodeIdHotkey::<T>::insert(subnet_id, subnet_node_owner.1, new_hotkey.clone());
					HotkeySubnetNodeId::<T>::swap(subnet_id, old_hotkey.clone(), subnet_id, new_hotkey.clone());
					SubnetNodesData::<T>::try_mutate_exists(
						subnet_id,
						subnet_node_owner.1,
						|maybe_params| -> DispatchResult {
							let params = maybe_params.as_mut().ok_or(Error::<T>::SubnetNodeExist)?;
							params.hotkey = new_hotkey.clone();
							Ok(())
						}
					);
				}

				// --- Swap stake balance
				// If a subnet node or subnet is no longer active, the stake can still be available for unstaking
				let account_stake_balance: u128 = AccountSubnetStake::<T>::get(&old_hotkey, subnet_id);
				if account_stake_balance != 0 {
					Self::do_swap_hotkey_balance(
						origin.clone(), 
						subnet_id,
						&old_hotkey.clone(), 
						&new_hotkey.clone(), 
					);
				}
			}

			Ok(())
		}

		#[pallet::call_index(32)]
		#[pallet::weight({0})]
		pub fn update_peer_id(
			origin: OriginFor<T>, 
			subnet_id: u32,
			subnet_node_id: u32,
			new_peer_id: PeerId,
		) -> DispatchResult {
			let coldkey: T::AccountId = ensure_signed(origin.clone())?;

			ensure!(
				Self::is_subnet_node_coldkey(
					subnet_id, 
					subnet_node_id, 
					coldkey, 
				),
				Error::<T>::NotKeyOwner
			);

			// Must be deactivated to update peer_id


			// let subnet_node = match SubnetNodesData::<T>::try_get(subnet_id) {
      //   Ok(subnet_node) => subnet_node,
      //   Err(()) => return Err(Error::<T>::SubnetNodeNotExist.into()),
			// };

			Ok(())
		}

		#[pallet::call_index(33)]
		#[pallet::weight({0})]
		pub fn set_max_subnet_nodes(
			origin: OriginFor<T>, 
			value: u32
		) -> DispatchResult {
			T::MajorityCollectiveOrigin::ensure_origin(origin)?;

			Self::do_set_max_subnet_nodes(value)
		}

		#[pallet::call_index(34)]
		#[pallet::weight({0})]
		pub fn set_min_stake_balance(
			origin: OriginFor<T>, 
			value: u128
		) -> DispatchResult {
			T::SuperMajorityCollectiveOrigin::ensure_origin(origin)?;

			Self::do_set_min_stake_balance(value)
		}
	}

	impl<T: Config> Pallet<T> {
		/// Register subnet
		pub fn do_register_subnet(
			activator: T::AccountId,
			subnet_data: RegistrationSubnetData,
		) -> DispatchResult {
			// Ensure path is unique
			ensure!(
				!SubnetPaths::<T>::contains_key(subnet_data.clone().path),
				Error::<T>::SubnetExist
			);

			let epoch = Self::get_current_epoch_as_u32();

			ensure!(
				Self::can_subnet_register(epoch),
				Error::<T>::SubnetRegistrationCooldown
			);

			// --- Ensure total network memory isn't exceeded
			ensure!(
				TotalSubnetMemoryMB::<T>::get() + subnet_data.memory_mb <= MaxTotalSubnetMemoryMB::<T>::get(),
				Error::<T>::MaxTotalSubnetMemory
			);

			// Ensure max subnets not reached
			// Get total live subnets
			let total_subnets: u32 = (SubnetsData::<T>::iter().count()).try_into().unwrap();
			let max_subnets: u32 = MaxSubnets::<T>::get();
			ensure!(
				total_subnets < max_subnets,
				Error::<T>::MaxSubnets
			);

			// --- Ensure registration time period is allowed
			ensure!(
				subnet_data.registration_blocks >= MinSubnetRegistrationBlocks::<T>::get() && 
				subnet_data.registration_blocks <= MaxSubnetRegistrationBlocks::<T>::get(),
				Error::<T>::InvalidSubnetRegistrationBlocks
			);

			// --- Ensure memory under max
			ensure!(
				subnet_data.memory_mb <= MaxSubnetMemoryMB::<T>::get(),
				Error::<T>::MaxSubnetMemory
			);

			ensure!(
				subnet_data.entry_interval <= MaxSubnetEntryInterval::<T>::get(),
				Error::<T>::MaxSubnetEntryInterval
			);

			let block: u64 = Self::get_current_block_as_u64();
			let subnet_fee: u128 = Self::registration_cost(epoch);

			if subnet_fee > 0 {
				// unreserve from activator
				let subnet_fee_as_balance = Self::u128_to_balance(subnet_fee);

				ensure!(
					Self::can_remove_balance_from_coldkey_account(&activator, subnet_fee_as_balance.unwrap()),
					Error::<T>::NotEnoughBalanceToStake
				);
				
				// --- Burn fee
				ensure!(
					Self::remove_balance_from_coldkey_account(&activator, subnet_fee_as_balance.unwrap()) == true,
					Error::<T>::BalanceWithdrawalError
				);

				// TODO
				// Send portion to treasury
			}

			// Get total subnets ever
			let subnet_len: u32 = TotalSubnets::<T>::get();

			// Start the subnet_ids at 1
			let subnet_id = subnet_len + 1;
			
			let base_node_memory: u128 = BaseSubnetNodeMemoryMB::<T>::get();
	
			let min_subnet_nodes: u32 = Self::get_min_subnet_nodes(base_node_memory, subnet_data.memory_mb);
			let target_subnet_nodes: u32 = Self::get_target_subnet_nodes(min_subnet_nodes);
	
			let subnet_data = SubnetData {
				id: subnet_id,
				path: subnet_data.clone().path,
				min_nodes: min_subnet_nodes,
				target_nodes: target_subnet_nodes,
				memory_mb: subnet_data.memory_mb, 
				registration_blocks: subnet_data.clone().registration_blocks,
				initialized: block,
				activated: 0,
				entry_interval: 0,
			};

			// Increase total subnet memory
			TotalSubnetMemoryMB::<T>::mutate(|n: &mut u128| *n += subnet_data.memory_mb);
			// Store unique path
			SubnetPaths::<T>::insert(subnet_data.clone().path, subnet_id);
			// Store subnet data
			SubnetsData::<T>::insert(subnet_id, subnet_data.clone());
			// Increase total subnets. This is used for unique Subnet IDs
			TotalSubnets::<T>::mutate(|n: &mut u32| *n += 1);
			// Update latest registration epoch
			LastSubnetRegistrationEpoch::<T>::set(epoch);

			Self::deposit_event(Event::SubnetRegistered { 
				account_id: activator, 
				path: subnet_data.path, 
				subnet_id: subnet_id 
			});

			Ok(())
		}

		/// Activate subnet or remove registering subnet if doesn't meet requirements
		pub fn do_activate_subnet(subnet_id: u32) -> DispatchResult {
			let subnet = match SubnetsData::<T>::try_get(subnet_id) {
        Ok(subnet) => subnet,
        Err(()) => return Err(Error::<T>::InvalidSubnetId.into()),
			};

			ensure!(
				subnet.activated == 0,
				Error::<T>::SubnetActivatedAlready
			);

			let block: u64 = Self::get_current_block_as_u64();

			// --- Ensure the subnet has passed it's required period to begin consensus submissions
			// --- Ensure the subnet is within the enactment period
			ensure!(
				block > subnet.initialized + subnet.registration_blocks,
				Error::<T>::SubnetInitializing
			);

			// --- If subnet not activated yet and is outside the enactment period, remove subnet
			if block > subnet.initialized + subnet.registration_blocks + SubnetActivationEnactmentPeriod::<T>::get() {
				return Self::deactivate_subnet(
					subnet.path,
					SubnetRemovalReason::EnactmentPeriod,
				)
			}

			// --- Ensure minimum nodes are activated
			let epoch_length: u64 = T::EpochLength::get();
			let epoch: u64 = block / epoch_length;
			let subnet_node_ids: Vec<u32> = Self::get_classified_subnet_node_ids(subnet_id, &SubnetNodeClass::Validator, epoch);
      let subnet_nodes_count: u32 = subnet_node_ids.len() as u32;

			if subnet_nodes_count < subnet.min_nodes {
				return Self::deactivate_subnet(
					subnet.path,
					SubnetRemovalReason::MinSubnetNodes,
				)
			}

			// --- Ensure minimum delegate stake achieved 
			let subnet_delegate_stake_balance = TotalSubnetDelegateStakeBalance::<T>::get(subnet_id);
			let min_subnet_delegate_stake_balance = Self::get_min_subnet_delegate_stake_balance(subnet.min_nodes);

			// --- Ensure delegate stake balance is below minimum threshold required
			if subnet_delegate_stake_balance < min_subnet_delegate_stake_balance {
				return Self::deactivate_subnet(
					subnet.path,
					SubnetRemovalReason::MinSubnetDelegateStake,
				)
			}

			// --- Gauntlet passed

			// --- Activate subnet
			SubnetsData::<T>::try_mutate(
				subnet_id,
				|maybe_params| -> DispatchResult {
					let params = maybe_params.as_mut().ok_or(Error::<T>::InvalidSubnetId)?;
					params.activated = block;
					Ok(())
				}
			)?;

      Self::deposit_event(Event::SubnetActivated { subnet_id: subnet_id });
	
			Ok(())
		}

		pub fn deactivate_subnet(
			path: Vec<u8>,
			reason: SubnetRemovalReason,
		) -> DispatchResult {
			ensure!(
				SubnetPaths::<T>::contains_key(path.clone()),
				Error::<T>::SubnetNotExist
			);

			let subnet_id = SubnetPaths::<T>::get(path.clone()).unwrap();

			let subnet = match SubnetsData::<T>::try_get(subnet_id) {
        Ok(subnet) => subnet,
        Err(()) => return Err(Error::<T>::SubnetNotExist.into()),
			};

			// Remove unique path
			SubnetPaths::<T>::remove(path.clone());
			// Remove subnet data
			SubnetsData::<T>::remove(subnet_id);
			// Decrease total subnet memory
			TotalSubnetMemoryMB::<T>::mutate(|n: &mut u128| n.saturating_reduce(subnet.memory_mb));
			// Remove subnet entry ledger
			LastSubnetEntry::<T>::remove(subnet_id);

			// We don't subtract TotalSubnets since it's used for ids

			// Remove all subnet nodes data
			let _ = SubnetNodesData::<T>::clear_prefix(subnet_id, u32::MAX, None);
			let _ = TotalSubnetNodes::<T>::remove(subnet_id);
			let _ = SubnetNodeAccount::<T>::clear_prefix(subnet_id, u32::MAX, None);
			let _ = SubnetNodeUniqueParam::<T>::clear_prefix(subnet_id, u32::MAX, None);
			let _ = HotkeySubnetNodeId::<T>::clear_prefix(subnet_id, u32::MAX, None);
			let _ = SubnetNodeIdHotkey::<T>::clear_prefix(subnet_id, u32::MAX, None);

			// Remove all subnet consensus data
			let _ = SubnetPenaltyCount::<T>::remove(subnet_id);

			// Remove consensus data
			let _ = SubnetRewardsSubmission::<T>::clear_prefix(subnet_id, u32::MAX, None);

			// Remove proposals
			let _ = Proposals::<T>::clear_prefix(subnet_id, u32::MAX, None);
	
			Self::deposit_event(Event::SubnetDeactivated { subnet_id: subnet_id, reason: reason });

			Ok(())
		}

		pub fn do_remove_subnet_node(
			subnet_id: u32,
			subnet_node_id: u32,
		) -> DispatchResult {
			let block: u64 = Self::get_current_block_as_u64();

			// We don't check consensus steps here because a subnet nodes stake isn't included in calculating rewards 
			// that hasn't reached their consensus submission epoch yet
			Self::perform_remove_subnet_node(block, subnet_id, subnet_node_id);
			Ok(())
		}

		pub fn do_register_subnet_node(
			origin: OriginFor<T>,
			subnet_id: u32, 
			hotkey: T::AccountId,
			peer_id: PeerId, 
			delegate_reward_rate: u128,
			stake_to_be_added: u128,
			a: Option<BoundedVec<u8, DefaultSubnetNodeUniqueParamLimit>>,
			b: Option<BoundedVec<u8, DefaultSubnetNodeUniqueParamLimit>>,
			c: Option<BoundedVec<u8, DefaultSubnetNodeUniqueParamLimit>>,
		) -> DispatchResult {
			let coldkey: T::AccountId = ensure_signed(origin.clone())?;

			let subnet = match SubnetsData::<T>::try_get(subnet_id) {
        Ok(subnet) => subnet,
        Err(()) => return Err(Error::<T>::SubnetNotExist.into()),
			};

			let block: u64 = Self::get_current_block_as_u64();
			ensure!(
				block >= LastSubnetEntry::<T>::get(subnet_id) + subnet.entry_interval,
				Error::<T>::MaxSubnetEntryIntervalReached
			);

			// Ensure hotkey either has no owner or is the origins hotkey
			match HotkeyOwner::<T>::try_get(hotkey.clone()) {
				Ok(coldkey_owner) => {
					ensure!(
						coldkey_owner == coldkey,
						Error::<T>::KeyOwnerTaken
					)
				},
				// Has no owner
				Err(()) => (),
			};

			// --- Subnet nodes can only register if within registration period or if it's activated
			// --- Ensure the subnet outside of the enactment period or still registering
			ensure!(
				subnet.activated != 0 || subnet.activated == 0 && block <= subnet.initialized + subnet.registration_blocks,
				Error::<T>::SubnetMustBeRegisteringOrActivated
			);

			// Ensure max nodes isn't surpassed
			let total_subnet_nodes: u32 = TotalSubnetNodes::<T>::get(subnet_id);
			let max_subnet_nodes: u32 = MaxSubnetNodes::<T>::get();
			ensure!(
				total_subnet_nodes < max_subnet_nodes,
				Error::<T>::SubnetNodesMax
			);

			// Unique subnet_id -> AccountId
			// Ensure account doesn't already have a peer within subnet
			ensure!(
				!HotkeySubnetNodeId::<T>::contains_key(subnet_id, hotkey.clone()),
				Error::<T>::SubnetNodeExist
			);

			// Unique ``a``
			// [here]
			if a.is_some() {
				ensure!(
					!SubnetNodeUniqueParam::<T>::contains_key(subnet_id, a.clone().unwrap()),
					Error::<T>::SubnetNodeUniqueParamTaken
				);
				SubnetNodeUniqueParam::<T>::insert(subnet_id, a.clone().unwrap(), peer_id.clone());
			}

			// Unique subnet_id -> PeerId
			// Ensure peer ID doesn't already exist within subnet regardless of coldkey
			match SubnetNodeAccount::<T>::try_get(subnet_id, peer_id.clone()) {
				Ok(_) => return Err(Error::<T>::PeerIdExist.into()),
				Err(()) => (),
			};

			// Validate peer_id
			ensure!(
				Self::validate_peer_id(peer_id.clone()),
				Error::<T>::InvalidPeerId
			);

			// --- Ensure they have no stake on registration
			// If a subnet node deregisters, then they must fully unstake its stake balance to register again using that same balance
			ensure!(
				AccountSubnetStake::<T>::get(hotkey.clone(), subnet_id) == 0,
				Error::<T>::MustUnstakeToRegister
			);

			// ====================
			// Initiate stake logic
			// ====================
			Self::do_add_stake(
				origin.clone(), 
				subnet_id,
				hotkey.clone(),
				stake_to_be_added,
			).map_err(|e| e)?;

			// To ensure the AccountId that owns the PeerId, they must sign the PeerId for others to verify
			// This ensures others cannot claim to own a PeerId they are not the owner of
			// Self::validate_signature(&Encode::encode(&peer_id), &signature, &signer)?;
			let epoch_length: u64 = T::EpochLength::get();
			let epoch: u64 = block / epoch_length;

			// ========================
			// Insert peer into storage
			// ========================
			let classification: SubnetNodeClassification = SubnetNodeClassification {
				class: SubnetNodeClass::Registered,
				start_epoch: epoch,
			};

			let subnet_node: SubnetNode<T::AccountId> = SubnetNode {
				hotkey: hotkey.clone(),
				peer_id: peer_id.clone(),
				initialized: 0,
				classification: classification,
				delegate_reward_rate: delegate_reward_rate,
				last_delegate_reward_rate_update: block,
				a: a,
				b: b,
				c: c,
			};

			// --- Start the UIDs at zero
			let uid = TotalSubnetNodeUids::<T>::get(subnet_id);

			HotkeySubnetNodeId::<T>::insert(subnet_id, hotkey.clone(), uid);

			// Insert subnet node ID -> hotkey
			SubnetNodeIdHotkey::<T>::insert(subnet_id, uid, hotkey.clone());

			// Insert hotkey -> coldkey
			HotkeyOwner::<T>::insert(hotkey.clone(), coldkey.clone());
			
			// Insert SubnetNodesData with hotkey as key
			SubnetNodesData::<T>::insert(subnet_id, uid, subnet_node);

			// Insert subnet peer account to keep peer_ids unique within subnets
			SubnetNodeAccount::<T>::insert(subnet_id, peer_id.clone(), uid);

			let next_uid = uid + 1;
			TotalSubnetNodeUids::<T>::insert(subnet_id, next_uid);

			// Increase total subnet nodes
			TotalSubnetNodes::<T>::mutate(subnet_id, |n: &mut u32| *n += 1);

			LastSubnetEntry::<T>::insert(subnet_id, block);

			Self::deposit_event(
				Event::SubnetNodeRegistered { 
					subnet_id: subnet_id, 
					subnet_node_id: uid,
					coldkey: coldkey.clone(),
					hotkey: hotkey.clone(), 
					peer_id: peer_id.clone(),
				}
			);

			Ok(())
		}

		pub fn do_activate_subnet_node(
			origin: OriginFor<T>, 
			subnet_id: u32, 
			subnet_node_id: u32,
		) -> DispatchResult {
			let hotkey: T::AccountId = ensure_signed(origin)?;

			ensure!(
				HotkeySubnetNodeId::<T>::get(subnet_id, hotkey.clone()) == Some(subnet_node_id),
				Error::<T>::NotUidOwner
			);

			let epoch_length: u64 = T::EpochLength::get();
			let block: u64 = Self::get_current_block_as_u64();
			let epoch: u64 = block / epoch_length;

			let subnet = match SubnetsData::<T>::try_get(subnet_id) {
        Ok(subnet) => subnet,
        Err(()) => return Err(Error::<T>::SubnetNotExist.into()),
			};

			// --- Subnet nodes can only register if within registration period or if it's activated
			// --- Ensure the subnet outside of the enactment period or still registering
			ensure!(
				subnet.activated != 0 || subnet.activated == 0 && block <= subnet.initialized + subnet.registration_blocks,
				Error::<T>::SubnetMustBeRegisteringOrActivated
			);

			SubnetNodesData::<T>::try_mutate_exists(
				subnet_id,
				subnet_node_id,
				|maybe_params| -> DispatchResult {
					let params = maybe_params.as_mut().ok_or(Error::<T>::SubnetNodeExist)?;	
					ensure!(
						params.classification.class <= SubnetNodeClass::Registered,
            Error::<T>::SubnetNodeAlreadyActivated
					);

					// ensure!(
					// 	params.initialized == 0,
          //   Error::<T>::SubnetNodeAlreadyActivated
					// );
					// --- If subnet activated, activate starting at `Idle`
					let mut class = SubnetNodeClass::Idle;
					// --- Increase epoch by one to ensure node starts on a fresh epoch unless subnet is still registering
					let mut epoch_increase = 1;
					// --- If subnet in registration, activate starting at `Validator` to start off subnet consensus
					// --- Initial nodes before activation are entered as ``submittable`` nodes
					// They initiate the first consensus epoch and are responsible for increasing classifications
					// of other nodes that come in post activation
					if subnet.activated == 0 {
						class = SubnetNodeClass::Validator;
						epoch_increase -= 1;
					} else if params.classification.class == SubnetNodeClass::Deactivated {
						// --- If coming out od deactivation, start back at Validator on the following epoch
						class = SubnetNodeClass::Validator;
					}
					
					params.initialized = block;
					params.classification = SubnetNodeClassification {
						class: class,
						start_epoch: epoch + epoch_increase,
					};
					Ok(())
				}
			)?;
	
			Self::deposit_event(
				Event::SubnetNodeActivated { 
					subnet_id: subnet_id, 
					subnet_node_id: subnet_node_id, 
				}
			);

			Ok(())
		}

		pub fn do_deactivate_subnet_node(
			origin: OriginFor<T>, 
			subnet_id: u32, 
			subnet_node_id: u32,
		) -> DispatchResult {
			let key: T::AccountId = ensure_signed(origin)?;

			let (hotkey, coldkey) = match Self::get_hotkey_coldkey(subnet_id, subnet_node_id) {
				Some((hotkey, coldkey)) => {
					(hotkey, coldkey)
				}
				None => {
					return Err(Error::<T>::NotUidOwner.into())
				}
			};

			ensure!(
				key == hotkey.clone() || key == coldkey,
				Error::<T>::NotKeyOwner
			);

			let epoch_length: u64 = T::EpochLength::get();
			let block: u64 = Self::get_current_block_as_u64();
			let epoch: u64 = block / epoch_length;

			let subnet_node = match SubnetNodesData::<T>::try_get(subnet_id, subnet_node_id) {
				Ok(subnet_node) => {
					ensure!(
						subnet_node.classification.class >= SubnetNodeClass::Validator,
            Error::<T>::SubnetNodeNotActivated
					);
					subnet_node
				},
				Err(()) => return Err(Error::<T>::SubnetNotExist.into()),
			};	

			// --- Check if attested or validator, if so, add to actions to remove after rewards
      let use_ledger: bool = match SubnetRewardsSubmission::<T>::try_get(
        subnet_id, 
        epoch as u32
      ) {
        Ok(submission) => {
					let mut is_attestor = false;
					let attests = submission.attests;
					if attests.get(&subnet_node_id).is_some() {
						is_attestor = true
					}
					is_attestor
				},
        Err(()) => {
					let is_validator: bool = match SubnetRewardsValidator::<T>::try_get(subnet_id, epoch as u32) {
						Ok(validator_id) => {
							let mut is_validator = false;
							if subnet_node_id == validator_id {
								is_validator = true
							}
							is_validator
						},
						Err(()) => false,
					};
					is_validator
				},
      };

			if use_ledger {
				let mut deactivation_ledger = DeactivationLedger::<T>::get();

				deactivation_ledger.insert(
					SubnetNodeDeactivation {
						subnet_id: subnet_id,
						subnet_node_id: subnet_node_id,
					}	
				);

				DeactivationLedger::<T>::set(deactivation_ledger);

				Self::deposit_event(
					Event::SubnetNodeDeactivated { 
						subnet_id: subnet_id, 
						subnet_node_id: subnet_node_id, 
					}
				);
	
				return Ok(())
			}

			SubnetNodesData::<T>::try_mutate_exists(
				subnet_id,
				subnet_node_id,
				|maybe_params| -> DispatchResult {
					let params = maybe_params.as_mut().ok_or(Error::<T>::SubnetNodeExist)?;	
					ensure!(
						params.classification.class >= SubnetNodeClass::Validator,
            Error::<T>::SubnetNodeNotActivated
					);
					params.classification = SubnetNodeClassification {
						class: SubnetNodeClass::Deactivated,
						start_epoch: epoch, // update to current epoch
					};
					Ok(())
				}
			)?;

			Self::deposit_event(
				Event::SubnetNodeDeactivated { 
					subnet_id: subnet_id, 
					subnet_node_id: subnet_node_id, 
				}
			);

			Ok(())
		}

		/// Run deactivation ledger
		///
		/// * This is called by blockchain validator nodes.
		/// * This updates a subnet node class from Validator to Deactivated.
		/// * This iterates deactivations up to the `MaxDeactivations`. Any nodes remaining will be delayed
		///	  until the following epochs until their turn is met.
		/// 
		pub fn do_deactivation_ledger() -> Weight {
			let mut deactivation_ledger = DeactivationLedger::<T>::get();

			if deactivation_ledger.is_empty() {
				return Weight::zero()
			}
	
			// --- Get current subnet IDs
			let subnet_ids: BTreeSet<u32> = SubnetsData::<T>::iter_keys()
				.map(|id| id)
				.collect();

			let subnet_node_registration_epochs = SubnetNodeRegistrationEpochs::<T>::get();
			let max = MaxDeactivations::<T>::get();

			let epoch_length: u64 = T::EpochLength::get();
			let block: u64 = Self::get_current_block_as_u64();
			let epoch: u64 = block / epoch_length;

			let mut i: u32 = 0;
			for data in deactivation_ledger.clone().iter() {
				if i == max {
					break
				}
				
				let subnet_id = data.subnet_id;

				// --- If subnet and subnet node exists, otherwise pass and remove set
				if subnet_ids.get(&subnet_id) != None {
					let subnet_node_id = data.subnet_node_id;
					SubnetNodesData::<T>::try_mutate_exists(
						subnet_id,
						subnet_node_id,
						|maybe_params| -> DispatchResult {
							let params = maybe_params.as_mut().ok_or(Error::<T>::SubnetNodeExist)?;							
							params.initialized = block;
							params.classification = SubnetNodeClassification {
								class: SubnetNodeClass::Deactivated,
								start_epoch: epoch + 1,
							};
							Ok(())
						}
					);
				}

				// --- Remove from ledger
				// Removes item from ledger if succeeded or not (even if subnet was previously removed)
				deactivation_ledger.remove(data);

				i+=1;
			}
			DeactivationLedger::<T>::set(deactivation_ledger.clone());

			T::WeightInfo::do_deactivation_ledger(subnet_ids.len() as u32, i)
		}
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		/// Run block functions
		///
		/// # Flow
		///
		/// At the start of each epoch
		///
		/// 1. Reward subnet nodes from the previous epochs validators data
		/// 2. Run deactivation ledger
		/// 3. Do epoch preliminaries
		///		* Remove subnets if needed
		/// 	* Randomly choose subnet validators
		///
		/// # Arguments
		///
		/// * `block_number` - Current block number.
		/// 
		fn on_initialize(block_number: BlockNumberFor<T>) -> Weight {
			let block: u64 = Self::convert_block_as_u64(block_number);
			let epoch_length: u64 = T::EpochLength::get();

			// Reward subnet nodes
			if block >= epoch_length && block % epoch_length == 0 {
				let epoch: u64 = block / epoch_length;

				// Reward subnets for the previous epoch
				// Reward before shifting
				Self::reward_subnets(block, (epoch - 1) as u32);

				// return T::WeightInfo::on_initialize_reward_subnets();
				return Weight::from_parts(207_283_478_000, 22166406)
					.saturating_add(T::DbWeight::get().reads(18250_u64))
					.saturating_add(T::DbWeight::get().writes(12002_u64));

			} else if (block - 1) >= epoch_length && (block - 1) % epoch_length == 0 {
				// --- Execute deactivate ledger before choosing validators
				return Self::do_deactivation_ledger()
			} else if (block - 2) >= epoch_length && (block - 2) % epoch_length == 0 {
				// We save some weight by waiting one more block to choose validators
				// Run the block succeeding form consensus
				let epoch: u64 = block / epoch_length;

				// Choose validators for the current epoch
				Self::do_epoch_preliminaries(block, epoch as u32, epoch_length);

				return Weight::from_parts(207_283_478_000, 22166406)
					.saturating_add(T::DbWeight::get().reads(18250_u64))
					.saturating_add(T::DbWeight::get().writes(12002_u64));
			}

			// return T::WeightInfo::on_initialize()
			return Weight::from_parts(207_283_478_000, 22166406)
				.saturating_add(T::DbWeight::get().reads(18250_u64))
				.saturating_add(T::DbWeight::get().writes(12002_u64))
		}

		fn on_idle(block_number: BlockNumberFor<T>, remaining_weight: Weight) -> Weight {
			let block: u64 = Self::convert_block_as_u64(block_number);

			if remaining_weight.any_lt(T::DbWeight::get().reads(2)) {
				return Weight::from_parts(0, 0)
			}

			return Weight::from_parts(0, 0)

			// Self::do_on_idle(remaining_weight)
		}
	}

	impl<T: Config> Pallet<T> {
		pub(crate) fn do_on_idle(remaining_weight: Weight) -> Weight {
			// any weight that is unaccounted for
			let mut unaccounted_weight = Weight::from_parts(0, 0);

			unaccounted_weight
		}
	}

	#[pallet::genesis_config]
	#[derive(frame_support::DefaultNoBound)]
	pub struct GenesisConfig<T: Config> {
		pub subnet_path: Vec<u8>,
		pub memory_mb: u128,
		pub subnet_nodes: Vec<(T::AccountId, PeerId)>,
	}

	#[pallet::genesis_build]
	impl<T: Config> BuildGenesisConfig for GenesisConfig<T> {
		fn build(&self) {
			// MinSubnetRegistrationBlocks::<T>::put(50);

			if self.subnet_path.last().is_none() {
				return
			}
			
			let subnet_id = 1;

			let base_node_memory: u128 = BaseSubnetNodeMemoryMB::<T>::get();

			// --- Get min nodes based on default memory settings
			let real_min_subnet_nodes: u128 = self.memory_mb.clone() / base_node_memory;
			let mut min_subnet_nodes: u32 = MinSubnetNodes::<T>::get();
			if real_min_subnet_nodes as u32 > min_subnet_nodes {
				min_subnet_nodes = real_min_subnet_nodes as u32;
			}
				
			let target_subnet_nodes: u32 = (min_subnet_nodes as u128).saturating_mul(TargetSubnetNodesMultiplier::<T>::get()).saturating_div(1000000000) as u32 + min_subnet_nodes;

			let subnet_data = SubnetData {
				id: subnet_id,
				path: self.subnet_path.clone(),
				min_nodes: min_subnet_nodes,
				target_nodes: target_subnet_nodes,
				memory_mb: self.memory_mb.clone(),
				registration_blocks: MinSubnetRegistrationBlocks::<T>::get(),
				initialized: 1,
				activated: 0,
				entry_interval: 0,
			};

			// Increase total subnet memory
			TotalSubnetMemoryMB::<T>::mutate(|n: &mut u128| *n += subnet_data.memory_mb);			
			// Store unique path
			SubnetPaths::<T>::insert(self.subnet_path.clone(), subnet_id);
			// Store subnet data
			SubnetsData::<T>::insert(subnet_id, subnet_data.clone());
			// Increase total subnets count
			TotalSubnets::<T>::mutate(|n: &mut u32| *n += 1);

			LastSubnetRegistrationEpoch::<T>::set(1);

			// Increase delegate stake to allow activation of subnet model
			let min_stake_balance = MinStakeBalance::<T>::get();
			// --- Get minimum subnet stake balance
			let min_subnet_stake_balance = min_stake_balance * min_subnet_nodes as u128;
			// --- Get required delegate stake balance for a subnet to have to stay live
			let mut min_subnet_delegate_stake_balance = (min_subnet_stake_balance as u128).saturating_mul(MinSubnetDelegateStakePercentage::<T>::get()).saturating_div(1000000000);

			// --- Get absolute minimum required subnet delegate stake balance
			let min_subnet_delegate_stake = MinSubnetDelegateStake::<T>::get();
			// --- Return here if the absolute minimum required subnet delegate stake balance is greater
			//     than the calculated minimum requirement
			if min_subnet_delegate_stake > min_subnet_delegate_stake_balance {
				min_subnet_delegate_stake_balance = min_subnet_delegate_stake
			}	
			TotalSubnetDelegateStakeBalance::<T>::insert(subnet_id, min_subnet_delegate_stake_balance);
			
			// // --- Initialize subnet nodes
			// // Only initialize to test using subnet nodes
			// // If testing using subnet nodes in a subnet, comment out the ``for`` loop

			// let mut stake_amount: u128 = MinStakeBalance::<T>::get();
			

			//
			//
			//
			
			// let mut count = 0;
			// for (account_id, peer_id) in &self.subnet_nodes {
			// 	// Redundant
			// 	// Unique subnet_id -> PeerId
			// 	// Ensure peer ID doesn't already exist within subnet regardless of account_id
			// 	let peer_exists: bool = match SubnetNodeAccount::<T>::try_get(subnet_id, peer_id.clone()) {
			// 		Ok(_) => true,
			// 		Err(()) => false,
			// 	};

			// 	if peer_exists {
			// 		continue;
			// 	}

			// 	// ====================
			// 	// Initiate stake logic
			// 	// ====================
			// 	// T::Currency::withdraw(
			// 	// 	&account_id,
			// 	// 	stake_amount,
			// 	// 	WithdrawReasons::except(WithdrawReasons::TIP),
			// 	// 	ExistenceRequirement::KeepAlive,
			// 	// );

			// 	// -- increase account subnet staking balance
			// 	AccountSubnetStake::<T>::insert(
			// 		account_id,
			// 		subnet_id,
			// 		AccountSubnetStake::<T>::get(account_id, subnet_id).saturating_add(stake_amount),
			// 	);

			// 	// -- increase total subnet stake
			// 	TotalSubnetStake::<T>::mutate(subnet_id, |mut n| *n += stake_amount);

			// 	// -- increase total stake overall
			// 	TotalStake::<T>::mutate(|mut n| *n += stake_amount);

			// 	// To ensure the AccountId that owns the PeerId, they must sign the PeerId for others to verify
			// 	// This ensures others cannot claim to own a PeerId they are not the owner of
			// 	// Self::validate_signature(&Encode::encode(&peer_id), &signature, &signer)?;

			// 	// ========================
			// 	// Insert peer into storage
			// 	// ========================
			// 	let classification = SubnetNodeClassification {
			// 		class: SubnetNodeClass::Validator,
			// 		start_epoch: 0,
			// 	};

			// 	let bounded_peer_id: BoundedVec<u8, DefaultSubnetNodeUniqueParamLimit> = BoundedVec::try_from(peer_id.clone().0)
			// 		.expect("Vec is within bounds");

			// 	let subnet_node: SubnetNode<T::AccountId> = SubnetNode {
			// 		hotkey: account_id.clone(),
			// 		peer_id: peer_id.clone(),
			// 		initialized: 0,
			// 		classification: classification,
			// 		delegate_reward_rate: 0,
			// 		last_delegate_reward_rate_update: 0,	
			// 		a: Some(bounded_peer_id),
			// 		b: Some(BoundedVec::new()),
			// 		c: Some(BoundedVec::new()),
			// 	};
	
			// 	let uid = TotalSubnetNodeUids::<T>::get(subnet_id);

			// 	HotkeySubnetNodeId::<T>::insert(subnet_id, account_id.clone(), uid);
	
			// 	// Insert subnet node ID -> hotkey
			// 	SubnetNodeIdHotkey::<T>::insert(subnet_id, uid, account_id.clone());
	
			// 	// Insert hotkey -> coldkey
			// 	HotkeyOwner::<T>::insert(account_id.clone(), account_id.clone());
				
			// 	// Insert SubnetNodesData with hotkey as key
			// 	SubnetNodesData::<T>::insert(subnet_id, uid, subnet_node);
	
			// 	// Insert subnet peer account to keep peer_ids unique within subnets
			// 	SubnetNodeAccount::<T>::insert(subnet_id, peer_id.clone(), uid);
	
			// 	let next_uid = uid + 1;
			// 	TotalSubnetNodeUids::<T>::insert(subnet_id, next_uid);
	
			// 	// Increase total subnet nodes
			// 	TotalSubnetNodes::<T>::mutate(subnet_id, |n: &mut u32| *n += 1);

			// 	LastSubnetEntry::<T>::insert(subnet_id, 0);
	
			// 	count += 1;
			// }
		}
	}
}

// Staking logic from rewards pallet
impl<T: Config> IncreaseStakeVault for Pallet<T> {
	fn increase_stake_vault(amount: u128) -> DispatchResult {
		StakeVaultBalance::<T>::mutate(|n: &mut u128| *n += amount);
		Ok(())
	}
}
pub trait IncreaseStakeVault {
	fn increase_stake_vault(amount: u128) -> DispatchResult;
}