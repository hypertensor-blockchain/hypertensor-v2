use jsonrpsee::{
	core::{RpcResult},
	proc_macros::rpc,
	types::{error::ErrorObject, ErrorObjectOwned},
};

use sp_blockchain::HeaderBackend;
use sp_runtime::traits::Block as BlockT;
use std::sync::Arc;

use sp_api::ProvideRuntimeApi;

pub use network_custom_rpc_runtime_api::NetworkRuntimeApi;
use frame_support::storage::bounded_vec::BoundedVec;
use pallet_network::DefaultSubnetNodeUniqueParamLimit;

#[rpc(client, server)]
pub trait NetworkCustomApi<BlockHash> {
	#[method(name = "network_getSubnetNodes")]
	fn get_subnet_nodes(&self, subnet_id: u32, at: Option<BlockHash>) -> RpcResult<Vec<u8>>;
	#[method(name = "network_getSubnetNodesIncluded")]
	fn get_subnet_nodes_included(&self, subnet_id: u32, at: Option<BlockHash>) -> RpcResult<Vec<u8>>;
	#[method(name = "network_getSubnetNodesSubmittable")]
	fn get_subnet_nodes_submittable(&self, subnet_id: u32, at: Option<BlockHash>) -> RpcResult<Vec<u8>>;
	#[method(name = "network_getSubnetNodesUnconfirmedCount")]
	fn get_subnet_nodes_subnet_unconfirmed_count(&self, subnet_id: u32, at: Option<BlockHash>) -> RpcResult<u32>;
	#[method(name = "network_getConsensusData")]
	fn get_consensus_data(&self, subnet_id: u32, epoch: u32, at: Option<BlockHash>) -> RpcResult<Vec<u8>>;
	#[method(name = "network_getMinimumSubnetNodes")]
	fn get_minimum_subnet_nodes(&self, memory_mb: u128, at: Option<BlockHash>) -> RpcResult<u32>;
	#[method(name = "network_getMinimumDelegateStake")]
	fn get_minimum_delegate_stake(&self, memory_mb: u128, at: Option<BlockHash>) -> RpcResult<u128>;
	#[method(name = "network_getSubnetNodeInfo")]
	fn get_subnet_node_info(&self, subnet_id: u32, at: Option<BlockHash>) -> RpcResult<Vec<u8>>;
	#[method(name = "network_isSubnetNodeByPeerId")]
	fn is_subnet_node_by_peer_id(&self, subnet_id: u32, peer_id: Vec<u8>, at: Option<BlockHash>) -> RpcResult<bool>;
	#[method(name = "network_areSubnetNodesByPeerId")]
	fn are_subnet_nodes_by_peer_id(&self, subnet_id: u32, peer_ids: Vec<Vec<u8>>, at: Option<BlockHash>) -> RpcResult<Vec<u8>>;
	#[method(name = "network_isSubnetNodeByA")]
	fn is_subnet_node_by_a(&self, subnet_id: u32, a: BoundedVec<u8, DefaultSubnetNodeUniqueParamLimit>, at: Option<BlockHash>) -> RpcResult<bool>;
}

/// A struct that implements the `NetworkCustomApi`.
pub struct NetworkCustom<C, Block> {
	// If you have more generics, no need to NetworkCustom<C, M, N, P, ...>
	// just use a tuple like NetworkCustom<C, (M, N, P, ...)>
	client: Arc<C>,
	_marker: std::marker::PhantomData<Block>,
}

impl<C, Block> NetworkCustom<C, Block> {
	/// Create new `NetworkCustom` instance with the given reference to the client.
	pub fn new(client: Arc<C>) -> Self {
		Self { 
      client, 
      _marker: Default::default() 
    }
	}
}

/// Error type of this RPC api.
pub enum Error {
	/// The call to runtime failed.
	RuntimeError(String),
}

impl From<Error> for ErrorObjectOwned {
	fn from(e: Error) -> Self {
			match e {
					Error::RuntimeError(e) => ErrorObject::owned(1, e, None::<()>),
			}
	}
}

impl From<Error> for i32 {
	fn from(e: Error) -> i32 {
		match e {
			Error::RuntimeError(_) => 1,
		}
	}
}

impl<C, Block> NetworkCustomApiServer<<Block as BlockT>::Hash> for NetworkCustom<C, Block>
where
	Block: BlockT,
	C: ProvideRuntimeApi<Block> + HeaderBackend<Block> + Send + Sync + 'static,
	C::Api: NetworkRuntimeApi<Block>,
{
	fn get_subnet_nodes(&self, subnet_id: u32, at: Option<<Block as BlockT>::Hash>) -> RpcResult<Vec<u8>> {
		let api = self.client.runtime_api();
		let at = at.unwrap_or_else(|| self.client.info().best_hash);
		api.get_subnet_nodes(at, subnet_id).map_err(|e| {
			Error::RuntimeError(format!("Unable to get subnet nodes: {:?}", e)).into()
		})
	}
	fn get_subnet_nodes_included(&self, subnet_id: u32, at: Option<<Block as BlockT>::Hash>) -> RpcResult<Vec<u8>> {
		let api = self.client.runtime_api();
		let at = at.unwrap_or_else(|| self.client.info().best_hash);
		api.get_subnet_nodes_included(at, subnet_id).map_err(|e| {
			Error::RuntimeError(format!("Unable to get subnet nodes included: {:?}", e)).into()
		})
	}
	fn get_subnet_nodes_submittable(&self, subnet_id: u32, at: Option<<Block as BlockT>::Hash>) -> RpcResult<Vec<u8>> {
		let api = self.client.runtime_api();
		let at = at.unwrap_or_else(|| self.client.info().best_hash);
		api.get_subnet_nodes_submittable(at, subnet_id).map_err(|e| {
			Error::RuntimeError(format!("Unable to get subnet nodes submittable: {:?}", e)).into()
		})
	}
	fn get_subnet_nodes_subnet_unconfirmed_count(&self, subnet_id: u32, at: Option<<Block as BlockT>::Hash>) -> RpcResult<u32> {
		let api = self.client.runtime_api();
		let at = at.unwrap_or_else(|| self.client.info().best_hash);
		api.get_subnet_nodes_subnet_unconfirmed_count(at, subnet_id).map_err(|e| {
			Error::RuntimeError(format!("Unable to get subnet nodes unconfirmed: {:?}", e)).into()
		})
	}
	fn get_consensus_data(&self, subnet_id: u32, epoch: u32, at: Option<<Block as BlockT>::Hash>) -> RpcResult<Vec<u8>> {
		let api = self.client.runtime_api();
		let at = at.unwrap_or_else(|| self.client.info().best_hash);
		api.get_consensus_data(at, subnet_id, epoch).map_err(|e| {
			Error::RuntimeError(format!("Unable to get consensus data: {:?}", e)).into()
		})
	}
	fn get_minimum_subnet_nodes(&self, memory_mb: u128, at: Option<<Block as BlockT>::Hash>) -> RpcResult<u32> {
		let api = self.client.runtime_api();
		let at = at.unwrap_or_else(|| self.client.info().best_hash);
		api.get_minimum_subnet_nodes(at, memory_mb).map_err(|e| {
			Error::RuntimeError(format!("Unable to get minimum subnet nodes: {:?}", e)).into()
		})
	}
	fn get_minimum_delegate_stake(&self, memory_mb: u128, at: Option<<Block as BlockT>::Hash>) -> RpcResult<u128> {
		let api = self.client.runtime_api();
		let at = at.unwrap_or_else(|| self.client.info().best_hash);
		api.get_minimum_delegate_stake(at, memory_mb).map_err(|e| {
			Error::RuntimeError(format!("Unable to minimuum delegate stake: {:?}", e)).into()
		})
	}
	fn get_subnet_node_info(&self, subnet_id: u32, at: Option<<Block as BlockT>::Hash>) -> RpcResult<Vec<u8>> {
		let api = self.client.runtime_api();
		let at = at.unwrap_or_else(|| self.client.info().best_hash);
		api.get_subnet_node_info(at, subnet_id).map_err(|e| {
			Error::RuntimeError(format!("Unable to get subnet node info: {:?}", e)).into()
		})
	}
	fn is_subnet_node_by_peer_id(&self, subnet_id: u32, peer_id: Vec<u8>, at: Option<<Block as BlockT>::Hash>) -> RpcResult<bool> {
		let api = self.client.runtime_api();
		let at = at.unwrap_or_else(|| self.client.info().best_hash);
		api.is_subnet_node_by_peer_id(at, subnet_id, peer_id).map_err(|e| {
			Error::RuntimeError(format!("Unable to subnet node by peer ID: {:?}", e)).into()
		})
	}
	fn are_subnet_nodes_by_peer_id(&self, subnet_id: u32, peer_ids: Vec<Vec<u8>>, at: Option<<Block as BlockT>::Hash>) -> RpcResult<Vec<u8>> {
		let api = self.client.runtime_api();
		let at = at.unwrap_or_else(|| self.client.info().best_hash);
		api.are_subnet_nodes_by_peer_id(at, subnet_id, peer_ids).map_err(|e| {
			Error::RuntimeError(format!("Unable to get subnet nodes by peer IDs: {:?}", e)).into()
		})
	}
	fn is_subnet_node_by_a(&self, subnet_id: u32, a: BoundedVec<u8, DefaultSubnetNodeUniqueParamLimit>, at: Option<<Block as BlockT>::Hash>) -> RpcResult<bool> {
		let api = self.client.runtime_api();
		let at = at.unwrap_or_else(|| self.client.info().best_hash);
		api.is_subnet_node_by_a(at, subnet_id, a).map_err(|e| {
			Error::RuntimeError(format!("Unable to get subnet nodes by a parameter: {:?}", e)).into()
		})
	}
}