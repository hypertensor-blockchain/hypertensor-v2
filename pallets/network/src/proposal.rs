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
use sp_runtime::traits::TrailingZeroInput;

impl<T: Config> Pallet<T> {
  // TODO: Max vector string limit
  pub fn do_propose(
    hotkey: T::AccountId, 
    subnet_id: u32,
    subnet_node_id: u32,
    peer_id: PeerId,
    data: Vec<u8>,
  ) -> DispatchResult {
    let proposer_subnet_node_id = HotkeySubnetNodeId::<T>::get(subnet_id, &hotkey);
    ensure!(
      proposer_subnet_node_id == Some(subnet_node_id),
      Error::<T>::NotUidOwner
    );

    // --- Ensure subnet exists
    let subnet = match SubnetsData::<T>::try_get(subnet_id) {
      Ok(subnet) => subnet,
      Err(()) => return Err(Error::<T>::SubnetNotExist.into()),
    };

    let block: u32 = Self::get_current_block_as_u32();
    let epoch: u32 = block / T::EpochLength::get();

    // --- Ensure proposer account has peer and is validator class
    match SubnetNodesData::<T>::try_get(
      subnet_id, 
      subnet_node_id
    ) {
      Ok(subnet_node) => subnet_node.has_classification(&SubnetNodeClass::Validator, epoch),
      Err(()) => return Err(Error::<T>::SubnetNotExist.into()),
    };

    // Unique subnet_id -> PeerId
    // Ensure peer ID exists within subnet
    let defendant_subnet_node_id = match PeerIdSubnetNode::<T>::try_get(subnet_id, &peer_id) {
      Ok(defendant_subnet_node_id) => defendant_subnet_node_id,
      Err(()) => return Err(Error::<T>::PeerIdNotExist.into()),
    };

    // --- Disputed hotkey cannot be the proposer
    ensure!(
      defendant_subnet_node_id != proposer_subnet_node_id.unwrap(),
      Error::<T>::PlaintiffIsDefendant
    );

    // --- Ensure the minimum required subnet peers exist
    // --- Only submittable can vote on proposals
    // --- Get all eligible voters from this block
    let subnet_nodes: BTreeSet<u32> = Self::get_classified_subnet_node_ids(subnet_id, &SubnetNodeClass::Validator, epoch);
    let subnet_nodes_count = subnet_nodes.len();

    // There must always be the required minimum subnet peers for each vote
    // This ensure decentralization in order for proposals to be accepted 

    // safe unwrap after `contains_key`
    ensure!(
      subnet_nodes_count as u32 >= MinSubnetNodes::<T>::get(),
      Error::<T>::SubnetNodesMin
    );

    // --- Ensure min nodes for proposals
    ensure!(
      subnet_nodes_count as u32 >= ProposalMinSubnetNodes::<T>::get(),
      Error::<T>::SubnetNodesMin
    );

    ensure!(
      !Self::account_has_active_proposal_as_plaintiff(
        subnet_id, 
        proposer_subnet_node_id.unwrap(), 
        block,
      ),
      Error::<T>::NodeHasActiveProposal
    );

    ensure!(
      !Self::account_has_active_proposal_as_defendant(
        subnet_id, 
        defendant_subnet_node_id, 
        block,
      ),
      Error::<T>::NodeHasActiveProposal
    );

    let proposal_bid_amount: u128 = ProposalBidAmount::<T>::get();
    let proposal_bid_amount_as_balance = Self::u128_to_balance(proposal_bid_amount);

    let can_withdraw: bool = Self::can_remove_balance_from_coldkey_account(
      &hotkey,
      proposal_bid_amount_as_balance.unwrap(),
    );

    ensure!(
      can_withdraw,
      Error::<T>::NotEnoughBalanceToBid
    );

    // --- Withdraw bid amount from proposer accounts
    let _ = T::Currency::withdraw(
      &hotkey,
      proposal_bid_amount_as_balance.unwrap(),
      WithdrawReasons::except(WithdrawReasons::TRANSFER),
      ExistenceRequirement::KeepAlive,
    );

    let proposal_id = ProposalsCount::<T>::get();

    // TODO: Test adding quorum and consensus into the Proposal storage
    //       by using the amount of nodes in the subnet
    //       It's possible the quorum or consensus for smaller subnets may not be divisible
    Proposals::<T>::insert(
      subnet_id,
      proposal_id,
      ProposalParams {
        subnet_id: subnet_id,
        plaintiff_id: proposer_subnet_node_id.unwrap(),
        defendant_id: defendant_subnet_node_id,
        plaintiff_bond: proposal_bid_amount,
        defendant_bond: 0,
        eligible_voters: subnet_nodes,
        votes: VoteParams {
          yay: BTreeSet::new(),
          nay: BTreeSet::new(),
        },
        start_block: block,
        challenge_block: 0, // No challenge block initially
        plaintiff_data: data.clone(),
        defendant_data: Vec::new(),
        complete: false,
      }
    );

    ProposalsCount::<T>::put(proposal_id + 1);

    Self::deposit_event(
      Event::Proposal { 
        subnet_id: subnet_id, 
        proposal_id: proposal_id,
        epoch: epoch as u32,
        plaintiff: hotkey.clone(), 
        defendant: hotkey.clone(),
        plaintiff_data: data
      }
    );

    Ok(())
  }

  pub fn do_attest_proposal(
    hotkey: T::AccountId, 
    subnet_id: u32,
    proposal_id: u32,
    data: Vec<u8>,
  ) -> DispatchResult {
    let proposal = match Proposals::<T>::try_get(subnet_id, proposal_id) {
      Ok(proposal) => proposal,
      Err(()) =>
        return Err(Error::<T>::ProposalInvalid.into()),
    };

    Self::deposit_event(
      Event::ProposalAttested{ 
        subnet_id: subnet_id, 
        proposal_id: proposal_id, 
        account_id: hotkey,
        attestor_data: data
      }
    );

    Ok(())
  }

  pub fn do_challenge_proposal(
    hotkey: T::AccountId, 
    subnet_id: u32,
    proposal_id: u32,
    data: Vec<u8>,
  ) -> DispatchResult {
    let proposal = match Proposals::<T>::try_get(subnet_id, proposal_id) {
      Ok(proposal) => proposal,
      Err(()) =>
        return Err(Error::<T>::ProposalInvalid.into()),
    };

    let subnet_node_id = HotkeySubnetNodeId::<T>::get(subnet_id, &hotkey);

    // --- Ensure defendant
    ensure!(
      subnet_node_id == Some(proposal.defendant_id),
      Error::<T>::NotDefendant
    );

    // --- Ensure incomplete
    ensure!(
      !proposal.complete,
      Error::<T>::ProposalComplete
    );
    
    let challenge_period = ChallengePeriod::<T>::get();
    let block: u32 = Self::get_current_block_as_u32();

    // --- Ensure challenge period is active
    ensure!(
      block < proposal.start_block + challenge_period,
      Error::<T>::ProposalChallengePeriodPassed
    );

    // --- Ensure unchallenged
    ensure!(
      proposal.challenge_block == 0,
      Error::<T>::ProposalChallenged
    );

    // --- Get plaintiffs bond to match
    // We get the plaintiff bond in case this amount is updated in between proposals
    let proposal_bid_amount_as_balance = Self::u128_to_balance(proposal.plaintiff_bond);

    let can_withdraw: bool = Self::can_remove_balance_from_coldkey_account(
      &hotkey,
      proposal_bid_amount_as_balance.unwrap(),
    );

    // --- Ensure can bond
    ensure!(
      can_withdraw,
      Error::<T>::NotEnoughBalanceToBid
    );

    // --- Withdraw bid amount from proposer accounts
    let _ = T::Currency::withdraw(
      &hotkey,
      proposal_bid_amount_as_balance.unwrap(),
      WithdrawReasons::except(WithdrawReasons::TRANSFER),
      ExistenceRequirement::KeepAlive,
    );

    let epoch: u32 = block / T::EpochLength::get();

    Proposals::<T>::mutate(
      subnet_id,
      proposal_id,
      |params: &mut ProposalParams| {
        params.defendant_data = data.clone();
        params.defendant_bond = proposal.plaintiff_bond;
        params.challenge_block = block;
      }
    );

    Self::deposit_event(
      Event::ProposalChallenged { 
        subnet_id: subnet_id, 
        proposal_id: proposal_id,
        defendant: hotkey, 
        defendant_data: data,
      }
    );

    Ok(())
  }

  pub fn do_vote(
    hotkey: T::AccountId, 
    subnet_id: u32,
    subnet_node_id: u32,
    proposal_id: u32,
    vote: VoteType
  ) -> DispatchResult {
    let voter_subnet_node_id = HotkeySubnetNodeId::<T>::get(subnet_id, &hotkey);
    ensure!(
      voter_subnet_node_id == Some(subnet_node_id),
      Error::<T>::NotUidOwner
    );

    let proposal = match Proposals::<T>::try_get(subnet_id, proposal_id) {
      Ok(proposal) => proposal,
      Err(()) =>
        return Err(Error::<T>::ProposalInvalid.into()),
    };

    let plaintiff_id = proposal.plaintiff_id;
    let defendant_id = proposal.defendant_id;

    // --- Ensure not plaintiff or defendant
    ensure!(
      subnet_node_id != plaintiff_id && subnet_node_id != defendant_id,
      Error::<T>::PartiesCannotVote
    );

    // --- Ensure account has peer
    // Proposal voters are calculated within ``do_proposal`` as ``eligible_voters`` so we check if they
    // are still nodes
    ensure!(
      SubnetNodesData::<T>::contains_key(subnet_id, subnet_node_id),
      Error::<T>::SubnetNodeNotExist
    );
    
    // --- Ensure challenged
    ensure!(
      proposal.challenge_block != 0,
      Error::<T>::ProposalUnchallenged
    );

    // --- Ensure incomplete
    ensure!(
      !proposal.complete,
      Error::<T>::ProposalComplete
    );
    
    let voting_period = VotingPeriod::<T>::get();
    let block: u32 = Self::get_current_block_as_u32();

    // --- Ensure voting period is active
    // Voting period starts after the challenge block
    ensure!(
      block < proposal.challenge_block + voting_period,
      Error::<T>::VotingPeriodInvalid
    );

    // --- Ensure is eligible to vote
    ensure!(
      proposal.eligible_voters.get(&subnet_node_id).is_some(),
      Error::<T>::NotEligible
    );

    let yays: BTreeSet<u32> = proposal.votes.yay;
    let nays: BTreeSet<u32> = proposal.votes.nay;

    // --- Ensure hasn't already voted
    ensure!(
      yays.get(&subnet_node_id) == None && nays.get(&subnet_node_id) == None,
      Error::<T>::AlreadyVoted
    );

    Proposals::<T>::mutate(
      subnet_id,
      proposal_id,
      |params: &mut ProposalParams| {
        if vote == VoteType::Yay {
          params.votes.yay.insert(subnet_node_id);
        } else {
          params.votes.nay.insert(subnet_node_id);
        };  
      }
    );
    
    Self::deposit_event(
      Event::ProposalVote { 
        subnet_id: subnet_id, 
        proposal_id: proposal_id,
        account_id: hotkey,
        vote: vote,
      }
    );

    Ok(())
  }

  pub fn do_cancel_proposal(
    hotkey: T::AccountId, 
    subnet_id: u32,
    subnet_node_id: u32,
    proposal_id: u32,
  ) -> DispatchResult {
    let proposal = match Proposals::<T>::try_get(subnet_id, proposal_id) {
      Ok(proposal) => proposal,
      Err(()) =>
        return Err(Error::<T>::ProposalInvalid.into()),
    };

    let canceler_subnet_node_id = HotkeySubnetNodeId::<T>::get(subnet_id, &hotkey);
    ensure!(
      subnet_node_id == canceler_subnet_node_id.unwrap(),
      Error::<T>::NotUidOwner
    );

    // --- Ensure plaintiff
    ensure!(
      subnet_node_id == proposal.plaintiff_id,
      Error::<T>::NotPlaintiff
    );
    
    // --- Ensure unchallenged
    ensure!(
      proposal.challenge_block == 0,
      Error::<T>::ProposalChallenged
    );

    // --- Ensure incomplete
    ensure!(
      !proposal.complete,
      Error::<T>::ProposalComplete
    );

    // --- Remove proposal
    Proposals::<T>::remove(subnet_id, proposal_id);

    let plaintiff_bond_as_balance = Self::u128_to_balance(proposal.plaintiff_bond);

    // Give plaintiff bond back
    T::Currency::deposit_creating(&hotkey, plaintiff_bond_as_balance.unwrap());

    Self::deposit_event(
      Event::ProposalCanceled { 
        subnet_id: subnet_id, 
        proposal_id: proposal_id,
      }
    );

    Ok(())
  }

  /// Finalize the proposal and come to a conclusion
  /// Either plaintiff or defendant win, or neither win if no consensus or quorum is met
  pub fn do_finalize_proposal(
    hotkey: T::AccountId, 
    subnet_id: u32,
    proposal_id: u32,
  ) -> DispatchResult {
    let proposal = match Proposals::<T>::try_get(subnet_id, proposal_id) {
      Ok(proposal) => proposal,
      Err(()) =>
        return Err(Error::<T>::ProposalInvalid.into()),
    };

    // --- Ensure challenged
    ensure!(
      proposal.challenge_block != 0,
      Error::<T>::ProposalUnchallenged
    );

    // --- Ensure incomplete
    ensure!(
      !proposal.complete,
      Error::<T>::ProposalComplete
    );
    
    let voting_period = VotingPeriod::<T>::get();
    let block: u32 = Self::get_current_block_as_u32();

    // --- Ensure voting period is completed
    ensure!(
      block > proposal.challenge_block + voting_period,
      Error::<T>::VotingPeriodInvalid
    );

    // TODO: include enactment period for executing proposals

    // --- Ensure quorum reached
    let yays_len: u128 = proposal.votes.yay.len() as u128;
    let nays_len: u128 = proposal.votes.nay.len() as u128;
    let voters_len: u128 = proposal.eligible_voters.len() as u128;
    let voting_percentage: u128 = Self::percent_div(yays_len + nays_len, voters_len);

    let yays_percentage: u128 = Self::percent_div(yays_len, voters_len);
    let nays_percentage: u128 = Self::percent_div(nays_len, voters_len);

    let plaintiff_bond_as_balance = Self::u128_to_balance(proposal.plaintiff_bond);
    let defendant_bond_as_balance = Self::u128_to_balance(proposal.defendant_bond);

    let quorum_reached: bool = voting_percentage >= ProposalQuorum::<T>::get();
    let consensus_threshold: u128 = ProposalConsensusThreshold::<T>::get();

    // --- Mark as complete
    Proposals::<T>::mutate(
      subnet_id,
      proposal_id,
      |params: &mut ProposalParams| {
        params.complete = true;
        params.plaintiff_bond = 0;
        params.defendant_bond = 0;
      }
    );

    // --- If quorum not reached and both voting options didn't succeed consensus then complete
    if !quorum_reached || 
      (yays_percentage < consensus_threshold && 
      nays_percentage < consensus_threshold && 
      quorum_reached)
    {
      // Give plaintiff and defendant bonds back
      SubnetNodeIdHotkey::<T>::try_get(subnet_id, proposal.plaintiff_id)
        .ok()
        .map(|hotkey| T::Currency::deposit_creating(&hotkey, plaintiff_bond_as_balance.unwrap()));

      SubnetNodeIdHotkey::<T>::try_get(subnet_id, proposal.defendant_id)
        .ok()
        .map(|hotkey| T::Currency::deposit_creating(&hotkey, defendant_bond_as_balance.unwrap()));
      return Ok(())
    }

    // --- At this point we know that one of the voting options are in consensus
    if yays_len > nays_len {
      // --- Plaintiff wins, return bond
      // --- Remove defendant
      // Self::perform_remove_subnet_node(block, subnet_id, proposal.defendant);
      // --- Return bond
      SubnetNodeIdHotkey::<T>::try_get(subnet_id, proposal.plaintiff_id)
        .ok()
        .map(|hotkey| T::Currency::deposit_creating(&hotkey, plaintiff_bond_as_balance.unwrap()));
      // --- Distribute bond to voters in consensus
      Self::distribute_bond(
        subnet_id,
        proposal.defendant_bond, 
        proposal.votes.yay,
        &proposal.plaintiff_id
      );
    } else {
      // --- Defendant wins, return bond
      SubnetNodeIdHotkey::<T>::try_get(subnet_id, proposal.defendant_id)
        .ok()
        .map(|hotkey| T::Currency::deposit_creating(&hotkey, defendant_bond_as_balance.unwrap()));
      // --- Distribute bond to voters in consensus
      Self::distribute_bond(
        subnet_id,
        proposal.plaintiff_bond, 
        proposal.votes.nay,
        &proposal.defendant_id
      );
    }

    Self::deposit_event(
      Event::ProposalFinalized{ 
        subnet_id: subnet_id, 
        proposal_id: proposal_id, 
      }
    );

    Ok(())
  }

  pub fn distribute_bond(
    subnet_id: u32,
    bond: u128, 
    mut distributees: BTreeSet<u32>,
    winner_id: &u32
  ) {
    // --- Insert winner to distributees
    //     Parties cannot vote but receive distribution
    distributees.insert(*winner_id);
    let voters_len = distributees.len();
    let distribution_amount = bond.saturating_div(voters_len as u128);
    let distribution_amount_as_balance = Self::u128_to_balance(distribution_amount);
    // Redundant
    if !distribution_amount_as_balance.is_some() {
      return
    }

    let mut total_distributed: u128 = 0;
    // --- Distribute losers bond to consensus
    for subnet_node_id in distributees {
      total_distributed += distribution_amount;
      SubnetNodeIdHotkey::<T>::try_get(subnet_id, subnet_node_id)
        .ok()
        .map(|hotkey| T::Currency::deposit_creating(&hotkey, distribution_amount_as_balance.unwrap()));
    }

    // --- Take care of dust and send to winner
    if total_distributed < bond {
      let remaining_bond = bond - total_distributed;
      let remaining_bid_as_balance = Self::u128_to_balance(remaining_bond);
      if remaining_bid_as_balance.is_some() {
        SubnetNodeIdHotkey::<T>::try_get(subnet_id, winner_id)
          .ok()
          .map(|hotkey| T::Currency::deposit_creating(&hotkey, remaining_bid_as_balance.unwrap()));
      }
    }
  }

  fn account_has_active_proposal_as_plaintiff(
    subnet_id: u32, 
    subnet_node_id: u32, 
    block: u32,
  ) -> bool {
    let challenge_period = ChallengePeriod::<T>::get();
    let voting_period = VotingPeriod::<T>::get();

    let mut active_proposal: bool = false;

    for proposal in Proposals::<T>::iter_prefix_values(subnet_id) {
      let plaintiff_id: u32 = proposal.plaintiff_id;
      if plaintiff_id != subnet_node_id {
        continue;
      }

      // At this point we have a proposal that matches the plaintiff
      let proposal_block: u32 = proposal.start_block;
      let challenge_block: u32 = proposal.challenge_block;
      if challenge_block == 0 {
        // If time remaining for challenge
        if block < proposal.start_block + challenge_period {
          active_proposal = true;
          break;
        }
      } else {
        // If time remaining for vote
        if block < challenge_block + voting_period {
          active_proposal = true;
          break;
        }
      }
    }

    active_proposal
  }

  /// Does a subnet node have a proposal against them under the following conditions
  /// Proposal must not be completed to qualify or awaiting challenge
  fn account_has_active_proposal_as_defendant(
    subnet_id: u32, 
    subnet_node_id: u32, 
    block: u32,
  ) -> bool {
    let challenge_period = ChallengePeriod::<T>::get();
    let voting_period = VotingPeriod::<T>::get();

    let mut active_proposal: bool = false;

    // Proposals::<T>::iter_prefix_values(subnet_id)
    //   .find(|x| {
    //     .defendant == *hotkey
    //   })

    for proposal in Proposals::<T>::iter_prefix_values(subnet_id) {
      let defendant_id: u32 = proposal.defendant_id;
      if defendant_id != subnet_node_id {
        continue;
      }

      // At this point we have a proposal that matches the defendant
      let proposal_block: u32 = proposal.start_block;
      let challenge_block: u32 = proposal.challenge_block;
      if challenge_block == 0 {
        // If time remaining for challenge
        if block < proposal.start_block + challenge_period {
          active_proposal = true;
          break;
        }
      } else {
        // If time remaining for vote
        if block < challenge_block + voting_period {
          active_proposal = true;
          break;
        }
      }
    }

    active_proposal
  }

  fn remove_proposal(subnet_id: u32, proposal_id: u32) {

  }

  fn delete_completed_proposals() {

  }
}