use solana_client::rpc;
use std::error::Error;

pub trait Reader: AccountReader {
    fn balance(&self, addr: solana_sdk::pubkey::Pubkey) -> Result<u64, Box<dyn Error>>;
    fn slot_height(&self) -> Result<u64, Box<dyn Error>>;
    fn latest_blockhash(&self) -> Result<rpc::GetLatestBlockhashResult, Box<dyn Error>>;
    fn chain_id(&self) -> Result<String, Box<dyn Error>>;
    fn get_fee_for_message(&self, msg: &str) -> Result<u64, Box<dyn Error>>;
    fn get_latest_block(&self) -> Result<rpc::GetBlockResult, Box<dyn Error>>;
}

