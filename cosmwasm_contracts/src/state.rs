use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Binary, Uint256};
use cw_storage_plus::{Item, Map};

use crate::deck::Deck;
use crate::types::{BaseState, DeckConfig};

pub const INVALID_INDEX: u32 = 999_999;

#[cw_serde]
pub struct Config {
    pub decrypt_verifier: Addr,
    pub deck5_verifier: Addr,
    pub deck30_verifier: Addr,
    pub deck52_verifier: Addr,
    pub next_game_id: u64,
}

impl Config {
    pub fn deck_verifier(&self, deck: DeckConfig) -> Addr {
        match deck {
            DeckConfig::Deck5Card => self.deck5_verifier.clone(),
            DeckConfig::Deck30Card => self.deck30_verifier.clone(),
            DeckConfig::Deck52Card => self.deck52_verifier.clone(),
        }
    }
}

#[cw_serde]
pub struct GameInfo {
    pub deck_config: DeckConfig,
    pub num_cards: u8,
    pub num_players: u8,
    pub encrypt_verifier: Addr,
}

#[cw_serde]
pub struct ShuffleGameState {
    pub state: BaseState,
    pub opening: u8,
    pub cur_player_index: u32,
    pub aggregate_pk_x: Uint256,
    pub aggregate_pk_y: Uint256,
    pub nonce: Uint256,
    pub player_hand: Vec<u32>,
    pub player_addrs: Vec<Addr>,
    pub signing_addrs: Vec<Addr>,
    pub player_pk_x: Vec<Uint256>,
    pub player_pk_y: Vec<Uint256>,
    pub deck: Deck,
}

impl ShuffleGameState {
    pub fn new(config: DeckConfig, num_players: u8) -> Self {
        Self {
            state: BaseState::Created,
            opening: 0,
            cur_player_index: 0,
            aggregate_pk_x: Uint256::zero(),
            aggregate_pk_y: Uint256::zero(),
            nonce: Uint256::zero(),
            player_hand: vec![0; num_players as usize],
            player_addrs: vec![],
            signing_addrs: vec![],
            player_pk_x: vec![],
            player_pk_y: vec![],
            deck: Deck::new(config),
        }
    }

    pub fn num_registered(&self) -> usize {
        self.player_addrs.len()
    }
}

pub const CONFIG: Item<Config> = Item::new("config");
pub const GAME_INFOS: Map<u64, GameInfo> = Map::new("game_infos");
pub const GAME_STATES: Map<u64, ShuffleGameState> = Map::new("game_states");
pub const ACTIVE_GAMES: Map<u64, Addr> = Map::new("active_games");
pub const NEXT_CALLBACK: Map<u64, Binary> = Map::new("next_callback");
