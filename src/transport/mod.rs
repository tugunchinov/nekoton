use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use ton_block::MsgAddressInt;

use nekoton_abi::TransactionId;
use nekoton_utils::Clock;

use crate::core::models::ReliableBehavior;

use self::models::*;

#[cfg(feature = "gql_transport")]
pub mod gql;
#[cfg(feature = "jrpc_transport")]
pub mod jrpc;

pub mod models;
#[cfg(any(feature = "gql_transport", feature = "jrpc_transport",))]
mod utils;

#[async_trait]
pub trait Transport: Send + Sync {
    fn info(&self) -> TransportInfo;

    async fn send_message(&self, message: &ton_block::Message) -> Result<()>;

    async fn get_contract_state(&self, address: &MsgAddressInt) -> Result<RawContractState>;

    async fn get_transactions(
        &self,
        address: MsgAddressInt,
        from: TransactionId,
        count: u8,
    ) -> Result<Vec<RawTransaction>>;

    async fn get_transaction(&self, id: &ton_types::UInt256) -> Result<Option<RawTransaction>>;

    async fn get_latest_key_block(&self) -> Result<ton_block::Block>;

    // NOTE: clock is used for caching here
    async fn get_blockchain_config(
        &self,
        clock: &dyn Clock,
    ) -> Result<ton_executor::BlockchainConfig>;
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct TransportInfo {
    pub max_transactions_per_fetch: u8,
    pub reliable_behavior: ReliableBehavior,
    pub has_key_blocks: bool,
}
