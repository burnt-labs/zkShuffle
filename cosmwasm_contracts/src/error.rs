use cosmwasm_std::StdError;
use thiserror::Error;

use crate::types::BaseState;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("unauthorized")]
    Unauthorized,

    #[error("game {game_id} not found")]
    GameNotFound { game_id: u64 },

    #[error("game {game_id} already full")]
    GameFull { game_id: u64 },

    #[error("invalid state for game {game_id}: expected {expected:?}, actual {actual:?}")]
    InvalidState {
        game_id: u64,
        expected: BaseState,
        actual: BaseState,
    },

    #[error("not this player's turn for game {game_id}")]
    NotPlayersTurn { game_id: u64 },

    #[error("invalid player id")]
    InvalidPlayer,

    #[error("invalid card index")]
    InvalidCardIndex,

    #[error("insufficient cards specified")]
    InvalidCardSelection,

    #[error("card already decrypted by this player")]
    AlreadyDecrypted,

    #[error("callback message missing")]
    MissingCallback,

    #[error("operation not supported")]
    NotSupported,
}
