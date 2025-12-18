use cosmwasm_schema::cw_serde;
use cosmwasm_std::Uint256;

pub use crate::bitmaps::BitMap256;

/// Deck configurations: small enum, we let cw_serde provide Clone/PartialEq/Eq etc.
/// We only add `Copy` explicitly (cw_serde already provides `Clone` which `Copy` requires).
#[cw_serde]
#[derive(Copy)]
pub enum DeckConfig {
    Deck5Card,
    Deck30Card,
    Deck52Card,
}

impl DeckConfig {
    pub fn num_cards(&self) -> u32 {
        match self {
            Self::Deck5Card => 5,
            Self::Deck30Card => 30,
            Self::Deck52Card => 52,
        }
    }
}

/// BaseState is small; cw_serde provides Clone/PartialEq/Eq, add Copy explicitly.
#[cw_serde]
#[derive(Copy)]
pub enum BaseState {
    Uncreated,
    Created,
    Registration,
    Shuffle,
    Deal,
    Open,
    GameError,
    Complete,
}

#[cw_serde]
pub struct Card {
    pub x: Uint256,
    pub y: Uint256,
}

#[cw_serde]
pub struct CardDelta {
    pub delta0: Uint256,
    pub delta1: Uint256,
}

#[cw_serde]
pub struct Groth16Proof {
    pub a: [Uint256; 2],
    pub b: [[Uint256; 2]; 2],
    pub c: [Uint256; 2],
}

#[cw_serde]
pub struct CompressedDeck {
    pub config: DeckConfig,
    pub x0: Vec<Uint256>,
    pub x1: Vec<Uint256>,
    pub selector0: BitMap256,
    pub selector1: BitMap256,
}

impl CompressedDeck {
    pub fn len_matches(&self) -> bool {
        let len = self.config.num_cards() as usize;
        self.x0.len() == len && self.x1.len() == len
    }
}
