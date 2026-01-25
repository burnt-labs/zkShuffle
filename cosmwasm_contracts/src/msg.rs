//! zkShuffle card game messages

use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Binary, Uint256};

use crate::types::{Card, CardDelta, CompressedDeck, Groth16Proof};

#[cw_serde]
pub struct InstantiateMsg {
    // No verifier addresses needed - using module-level verifiers
    // vkey_id: 3 for "shuffle_encrypt"
    // vkey_id: 2 for "decrypt"
}

#[cw_serde]
pub enum ExecuteMsg {
    // ========== Game Flow Messages ==========

    /// Create a new game (always 52 cards)
    CreateGame {
        num_players: u32,
    },

    /// Start the registration phase (owner only)
    Register {
        game_id: u64,
        callback: Option<Binary>,
    },

    /// Player registers with their public key
    PlayerRegister {
        game_id: u64,
        signing_addr: String,
        pk_x: Uint256,
        pk_y: Uint256,
    },

    /// Start the shuffle phase (owner only)
    Shuffle {
        game_id: u64,
        callback: Option<Binary>,
    },

    /// Player shuffles the deck with a ZK proof
    PlayerShuffle {
        game_id: u64,
        proof: Groth16Proof,
        deck: CompressedDeck,
    },

    /// Deal specific cards to a player (owner only)
    DealCardsTo {
        game_id: u64,
        cards: Binary, // BitMap256 serialized
        player_id: u32,
        callback: Option<Binary>,
    },

    /// Player decrypts dealt cards with ZK proofs
    PlayerDealCards {
        game_id: u64,
        proofs: Vec<Groth16Proof>,
        decrypted_cards: Vec<Card>,
        init_deltas: Vec<CardDelta>,
    },

    /// Start the opening/reveal phase (owner only)
    OpenCards {
        game_id: u64,
        player_id: u32,
        opening: u32,
        callback: Option<Binary>,
    },

    /// Player opens/reveals their cards
    PlayerOpenCards {
        game_id: u64,
        cards: Binary, // BitMap256 serialized
        proofs: Vec<Groth16Proof>,
        decrypted_cards: Vec<Card>,
    },

    // ========== Minimal Proof Verification Messages (backward compatibility) ==========

    /// Verify a shuffle_encrypt proof directly (minimal mode)
    VerifyShuffleProof {
        proof: Groth16Proof,
        public_inputs: Vec<Uint256>,
    },

    /// Verify a decrypt proof directly (minimal mode)
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

    #[returns(GameStateResponse)]
    GameState { game_id: u64 },
}

#[cw_serde]
pub struct VerificationCountResponse {
    pub shuffle_verifications: u64,
    pub decrypt_verifications: u64,
}

#[cw_serde]
pub struct GameStateResponse {
    pub game_id: u64,
    pub state: String,
    pub cur_player_index: u32,
    pub num_players: usize,
}
