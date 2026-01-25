//! State for zkShuffle card game and proof verification tracking

use cosmwasm_schema::cw_serde;
use cosmwasm_std::Addr;
use cw_storage_plus::{Item, Map};

use crate::deck::Deck;
use crate::types::{BaseState, Card, DeckConfig};

/// Proof verification tracking state (minimal)
#[cw_serde]
pub struct VerificationState {
    pub shuffle_verifications: u64,
    pub decrypt_verifications: u64,
}

impl VerificationState {
    pub fn new() -> Self {
        Self {
            shuffle_verifications: 0,
            decrypt_verifications: 0,
        }
    }
}

pub const VERIFICATION_STATE: Item<VerificationState> = Item::new("verification_state");

/// Game info - static metadata set at creation
#[cw_serde]
pub struct GameInfo {
    pub num_players: u32,
    pub owner: Addr,
}

/// Shuffle game state - dynamic game data
#[cw_serde]
pub struct ShuffleGameState {
    /// Current phase of the game
    pub state: BaseState,
    /// Current player index for turn-based operations
    pub cur_player_index: u32,
    /// The deck of cards (encrypted/decrypted state) - always 52 cards
    pub deck: Deck,
    /// Player addresses (who receives cards)
    pub player_addrs: Vec<Addr>,
    /// Signing addresses (who can submit proofs for each player)
    pub signing_addrs: Vec<Addr>,
    /// Public keys (x, y) for each player
    pub player_pks: Vec<Card>,
    /// Aggregated public key for encryption
    pub agg_pk: Card,
    /// Number of cards in each player's hand
    pub player_hand: Vec<u32>,
    /// Current opening round (for reveal phase)
    pub opening: u32,
}

impl ShuffleGameState {
    pub fn new(num_players: u32) -> Self {
        Self {
            state: BaseState::Created,
            cur_player_index: 0,
            deck: Deck::new(DeckConfig::Deck52Card),
            player_addrs: Vec::with_capacity(num_players as usize),
            signing_addrs: Vec::with_capacity(num_players as usize),
            player_pks: Vec::with_capacity(num_players as usize),
            agg_pk: Card {
                x: cosmwasm_std::Uint256::zero(),
                y: cosmwasm_std::Uint256::one(), // Identity element
            },
            player_hand: vec![0; num_players as usize],
            opening: 0,
        }
    }

    /// Get the current player address
    pub fn current_player(&self) -> Option<&Addr> {
        self.player_addrs.get(self.cur_player_index as usize)
    }

    /// Move to next player's turn, return true if back to player 0
    pub fn advance_player(&mut self) -> bool {
        let num_players = self.player_addrs.len() as u32;
        if num_players == 0 {
            return false;
        }
        self.cur_player_index = (self.cur_player_index + 1) % num_players;
        self.cur_player_index == 0
    }

    /// Check if all players are registered
    pub fn is_full(&self, expected_num_players: u32) -> bool {
        self.player_addrs.len() as u32 == expected_num_players
    }
}

/// Storage items
pub const GAME_INFOS: Map<u64, GameInfo> = Map::new("game_infos");
pub const GAME_STATES: Map<u64, ShuffleGameState> = Map::new("game_states");
