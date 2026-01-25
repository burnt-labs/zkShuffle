//! Minimal zkShuffle contract for testing proof verification at XION module level

use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Uint256;

use crate::types::{Groth16Proof};

#[cw_serde]
pub struct InstantiateMsg {
    // No verifier addresses needed - using module-level verifiers
    // vkey_id: 3 for "shuffle_encrypt"
    // vkey_id: 2 for "decrypt"
}

#[cw_serde]
pub enum ExecuteMsg {
    /// Verify a shuffle_encrypt proof
    VerifyShuffleProof {
        proof: Groth16Proof,
        public_inputs: Vec<Uint256>,
    },
    /// Verify a decrypt proof
    VerifyDecryptProof {
        proof: Groth16Proof,
        public_inputs: Vec<Uint256>,
    },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(VerificationCountResponse)]
    VerificationCount {},
}

#[cw_serde]
pub struct VerificationCountResponse {
    pub shuffle_verifications: u64,
    pub decrypt_verifications: u64,
}
