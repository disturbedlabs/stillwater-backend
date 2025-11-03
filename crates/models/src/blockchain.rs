use alloy::providers::{Provider, ProviderBuilder, RootProvider};
use alloy::transports::http::{Client, Http};
use anyhow::Result;

/// Blockchain service for interacting with Ethereum and Uniswap v4
pub struct BlockchainService {
    provider: RootProvider<Http<Client>>,
}

impl BlockchainService {
    /// Create a new blockchain service with the given RPC URL
    pub fn new(rpc_url: &str) -> Result<Self> {
        let provider = ProviderBuilder::new()
            .on_http(rpc_url.parse()?);

        Ok(Self { provider })
    }

    /// Get the current provider
    pub fn provider(&self) -> &RootProvider<Http<Client>> {
        &self.provider
    }

    /// Get the current block number
    pub async fn get_block_number(&self) -> Result<u64> {
        let block_number = self.provider.get_block_number().await?;
        Ok(block_number)
    }
}

impl Clone for BlockchainService {
    fn clone(&self) -> Self {
        Self {
            provider: self.provider.clone(),
        }
    }
}
