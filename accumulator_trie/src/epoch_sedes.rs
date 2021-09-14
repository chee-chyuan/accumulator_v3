use ssz_derive::{Decode, Encode};
use tree_hash_derive::TreeHash;
use ethereum_types::{H256, U256};

#[derive(Clone, Copy, Encode, Decode, Debug, PartialEq, TreeHash, Default)]
pub struct EpochSede {
    block_hash: H256,
    total_difficulty: U256,
}

impl EpochSede {
    pub fn new(total_difficulty: U256, block_hash: H256) -> EpochSede {
        EpochSede {
            block_hash,
            total_difficulty,
        }
    }
}