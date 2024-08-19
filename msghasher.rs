use std::collections::HashMap;
use std::error::Error;
use std::fmt;

use ethers::core::types::H256;
use ethers::abi::{Abi, Token};
use ethers::utils::keccak256;

pub struct MessageHasherV1;

impl MessageHasherV1 {
    pub fn new() -> Self {
        MessageHasherV1
    }

    pub fn hash(&self, msg: cciptypes::Message) -> Result<[u8; 32], Box<dyn Error>> {
        let mut ramp_token_amounts = Vec::new();
        for rta in msg.token_amounts {
            ramp_token_amounts.push(message_hasher::InternalRampTokenAmount {
                source_pool_address: rta.source_pool_address,
                dest_token_address: rta.dest_token_address,
                extra_data: rta.extra_data,
                amount: rta.amount.as_u128(),
            });
        }
        let encoded_ramp_token_amounts = abi_encode("encodeTokenAmountsHashPreimage", ramp_token_amounts)?;

        let meta_data_hash_input = abi_encode(
            "encodeMetadataHashPreimage",
            ANY_2_EVM_MESSAGE_HASH,
            msg.header.source_chain_selector as u64,
            msg.header.dest_chain_selector as u64,
            msg.header.on_ramp.as_bytes(),
        )?;

        let gas_limit = decode_extra_args_v1_v2(&msg.extra_args)?;

        let fixed_size_fields_encoded = abi_encode(
            "encodeFixedSizeFieldsHashPreimage",
            msg.header.message_id,
            msg.sender.as_bytes(),
            msg.receiver,
            msg.header.sequence_number as u64,
            gas_limit,
            msg.header.nonce,
        )?;

        let packed_values = abi_encode(
            "encodeFinalHashPreimage",
            LEAF_DOMAIN_SEPARATOR,
            keccak256(&meta_data_hash_input),
            keccak256(&fixed_size_fields_encoded),
            keccak256(&msg.data),
            keccak256(&encoded_ramp_token_amounts),
        )?;

        Ok(keccak256(&packed_values).into())
    }
}

fn abi_encode(method: &str, values: impl Into<Vec<Token>>) -> Result<Vec<u8>, Box<dyn Error>> {
    let res = message_hasher_abi().pack(method, values)?;
    // trim the method selector.
    Ok(res[4..].to_vec())
}

// Interface compliance check
impl cciptypes::MessageHasher for MessageHasherV1 {}

