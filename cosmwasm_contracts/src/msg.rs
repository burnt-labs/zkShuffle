use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Uint256;

use crate::types::{BaseState, BitMap256, Card, CardDelta, CompressedDeck, Groth16Proof};

#[cw_serde]
pub struct InstantiateMsg {
    // No verifier addresses needed - using module-level verifiers
    // vkey_id: 3 for "shuffle_encrypt"
    // vkey_id: 2 for "decrypt"
}

#[cw_serde]
pub enum ExecuteMsg {
    CreateGame {
        num_players: u8,
        // Simplified: always use 52-card deck
    },
    Register {
        game_id: u64,
    },
    PlayerRegister {
        game_id: u64,
        signing_addr: String,
        pk_x: Uint256,
        pk_y: Uint256,
    },
    Shuffle {
        game_id: u64,
    },
    PlayerShuffle {
        game_id: u64,
        proof: Groth16Proof,
        deck: CompressedDeck,
    },
    DealCardsTo {
        game_id: u64,
        cards: BitMap256,
        player_id: u32,
    },
    PlayerDealCards {
        game_id: u64,
        proofs: Vec<Groth16Proof>,
        decrypted_cards: Vec<Card>,
        init_deltas: Vec<CardDelta>,
    },
    OpenCards {
        game_id: u64,
        player_id: u32,
        opening: u8,
    },
    PlayerOpenCards {
        game_id: u64,
        cards: BitMap256,
        proofs: Vec<Groth16Proof>,
        decrypted_cards: Vec<Card>,
    },
    EndGame {
        game_id: u64,
    },
    Error {
        game_id: u64,
    },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(GameInfoResponse)]
    GameInfo { game_id: u64 },
    #[returns(GameStateResponse)]
    GameState { game_id: u64 },
    #[returns(NumCardsResponse)]
    NumCards { game_id: u64 },
    #[returns(PlayerIndexResponse)]
    CurPlayerIndex { game_id: u64 },
    #[returns(DecryptRecordResponse)]
    DecryptRecord { game_id: u64, card_index: u32 },
    #[returns(AggregatedPkResponse)]
    AggregatedPk { game_id: u64 },
    #[returns(DeckResponse)]
    Deck { game_id: u64 },
    #[returns(PlayerIndexResponse)]
    PlayerIndex { game_id: u64, address: String },
    #[returns(CardValueResponse)]
    CardValue { game_id: u64, card_index: u32 },
}

#[cw_serde]
pub struct GameInfoResponse {
    pub num_cards: u8,
    pub num_players: u8,
    // No verifier address needed - using module-level
}

#[cw_serde]
pub struct GameStateResponse {
    pub state: BaseState,
    pub opening: u8,
    pub cur_player_index: u32,
    pub aggregate_pk_x: Uint256,
    pub aggregate_pk_y: Uint256,
    pub nonce: Uint256,
    pub player_addrs: Vec<String>,
    pub signing_addrs: Vec<String>,
    pub player_hand: Vec<u32>,
}

#[cw_serde]
pub struct NumCardsResponse {
    pub count: u32,
}

#[cw_serde]
pub struct PlayerIndexResponse {
    pub index: Option<u32>,
}

#[cw_serde]
pub struct DecryptRecordResponse {
    pub bitmap: BitMap256,
}

#[cw_serde]
pub struct AggregatedPkResponse {
    pub px: Uint256,
    pub py: Uint256,
}

#[cw_serde]
pub struct DeckResponse {
    pub x0: Vec<Uint256>,
    pub x1: Vec<Uint256>,
    pub y0: Vec<Uint256>,
    pub y1: Vec<Uint256>,
    pub selector0: BitMap256,
    pub selector1: BitMap256,
    pub cards_to_deal: BitMap256,
    pub player_to_deal: u32,
}

#[cw_serde]
pub struct CardValueResponse {
    pub value: Option<u32>,
}
