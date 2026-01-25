//! zkShuffle card game contract with ZK proof verification

#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_json_binary, Addr, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult, Uint256,
};

use crate::curve::point_add;
use crate::deck::shuffle_public_input;
use crate::error::ContractError;
use crate::msg::{
    ExecuteMsg, InstantiateMsg, QueryMsg, VerificationCountResponse, GameStateResponse,
};
use crate::state::{GameInfo, GAME_INFOS, GAME_STATES, VERIFICATION_STATE};
use crate::types::{BaseState, BitMap256, Card, CardDelta, CompressedDeck, Groth16Proof};
use crate::zkshuffle::{verify_decrypt_proof, verify_shuffle_proof};

const CONTRACT_NAME: &str = "crates.io:zk-shuffle";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

// Next game ID counter (stored in VERIFICATION_STATE for now)
const NEXT_GAME_ID_KEY: &[u8] = b"next_game_id";

// Macro for state validation
macro_rules! ensure_state {
    ($game_state:expr, $game_id:expr, $expected:expr) => {
        if $game_state.state != $expected {
            return Err(ContractError::InvalidState {
                game_id: $game_id,
                expected: $expected,
                actual: $game_state.state,
            });
        }
    };
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let state = VERIFICATION_STATE.load(deps.storage);
    if state.is_err() {
        VERIFICATION_STATE.save(deps.storage, &crate::state::VerificationState::new())?;
    }

    Ok(Response::new().add_attribute("action", "instantiate"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        // ========== Game Flow Messages ==========
        ExecuteMsg::CreateGame { num_players } => {
            execute_create_game(deps, env, info, num_players)
        }
        ExecuteMsg::Register { game_id, callback } => {
            execute_register(deps, env, info, game_id)
        }
        ExecuteMsg::PlayerRegister {
            game_id,
            signing_addr,
            pk_x,
            pk_y,
        } => execute_player_register(deps, env, info, game_id, signing_addr, pk_x, pk_y),
        ExecuteMsg::Shuffle { game_id, callback } => execute_shuffle(deps, env, info, game_id),
        ExecuteMsg::PlayerShuffle { game_id, proof, deck } => {
            execute_player_shuffle(deps, env, info, game_id, proof, deck)
        }
        ExecuteMsg::DealCardsTo {
            game_id,
            cards,
            player_id,
            callback,
        } => execute_deal_cards_to(deps, env, info, game_id, cards, player_id),
        ExecuteMsg::PlayerDealCards {
            game_id,
            proofs,
            decrypted_cards,
            init_deltas,
        } => execute_player_deal_cards(deps, env, info, game_id, proofs, decrypted_cards, init_deltas),
        ExecuteMsg::OpenCards {
            game_id,
            player_id,
            opening,
            callback,
        } => execute_open_cards(deps, env, info, game_id, player_id, opening),
        ExecuteMsg::PlayerOpenCards {
            game_id,
            cards,
            proofs,
            decrypted_cards,
        } => execute_player_open_cards(deps, env, info, game_id, cards, proofs, decrypted_cards),

        // ========== Minimal Proof Verification Messages ==========
        ExecuteMsg::VerifyShuffleProof { proof, public_inputs } => {
            execute_verify_shuffle_proof(deps, proof, public_inputs)
        }
        ExecuteMsg::VerifyDecryptProof { proof, public_inputs } => {
            execute_verify_decrypt_proof(deps, proof, public_inputs)
        }
    }
}

// ========== Game Flow Implementations ==========

fn execute_create_game(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    num_players: u32,
) -> Result<Response, ContractError> {
    if num_players == 0 || num_players > 10 {
        return Err(ContractError::Std(cosmwasm_std::StdError::generic_err(
            "num_players must be between 1 and 10",
        )));
    }

    // Get next game ID
    let mut state = VERIFICATION_STATE.load(deps.storage)?;
    let game_id = state.shuffle_verifications + state.decrypt_verifications + 1;
    state.shuffle_verifications = game_id;
    VERIFICATION_STATE.save(deps.storage, &state)?;

    // Create game info
    let game_info = GameInfo {
        num_players,
        owner: info.sender.clone(),
    };
    GAME_INFOS.save(deps.storage, game_id, &game_info)?;

    // Create game state
    let game_state = crate::state::ShuffleGameState::new(num_players);
    GAME_STATES.save(deps.storage, game_id, &game_state)?;

    Ok(Response::new()
        .add_attribute("action", "create_game")
        .add_attribute("game_id", game_id.to_string())
        .add_attribute("owner", info.sender))
}

fn execute_register(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    game_id: u64,
) -> Result<Response, ContractError> {
    let game_info = GAME_INFOS.load(deps.storage, game_id)?;
    if game_info.owner != info.sender {
        return Err(ContractError::Unauthorized);
    }

    let mut game_state = GAME_STATES.load(deps.storage, game_id)?;
    ensure_state!(game_state, game_id, BaseState::Created);

    game_state.state = BaseState::Registration;
    GAME_STATES.save(deps.storage, game_id, &game_state)?;

    Ok(Response::new()
        .add_attribute("action", "register")
        .add_attribute("game_id", game_id.to_string())
        .add_attribute("state", "Registration"))
}

fn execute_player_register(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    game_id: u64,
    signing_addr: String,
    pk_x: Uint256,
    pk_y: Uint256,
) -> Result<Response, ContractError> {
    let game_info = GAME_INFOS.load(deps.storage, game_id)?;
    let mut game_state = GAME_STATES.load(deps.storage, game_id)?;
    ensure_state!(game_state, game_id, BaseState::Registration);

    if game_state.player_addrs.len() >= game_info.num_players as usize {
        return Err(ContractError::GameFull { game_id });
    }

    // Check if player already registered
    if game_state.player_addrs.contains(&info.sender) {
        return Err(ContractError::Std(cosmwasm_std::StdError::generic_err(
            "player already registered",
        )));
    }

    // Add player
    let signing_addr = deps.api.addr_validate(&signing_addr)?;
    game_state.player_addrs.push(info.sender.clone());
    game_state.signing_addrs.push(signing_addr);
    game_state.player_pks.push(Card { x: pk_x, y: pk_y });

    // Aggregate public key
    let agg_pk = &game_state.agg_pk;
    let new_agg = point_add(&agg_pk.x, &agg_pk.y, &pk_x, &pk_y)?;
    game_state.agg_pk = Card {
        x: new_agg.0,
        y: new_agg.1,
    };

    GAME_STATES.save(deps.storage, game_id, &game_state)?;

    Ok(Response::new()
        .add_attribute("action", "player_register")
        .add_attribute("game_id", game_id.to_string())
        .add_attribute("player", info.sender)
        .add_attribute("player_count", game_state.player_addrs.len().to_string()))
}

fn execute_shuffle(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    game_id: u64,
) -> Result<Response, ContractError> {
    let game_info = GAME_INFOS.load(deps.storage, game_id)?;
    if game_info.owner != info.sender {
        return Err(ContractError::Unauthorized);
    }

    let mut game_state = GAME_STATES.load(deps.storage, game_id)?;
    ensure_state!(game_state, game_id, BaseState::Registration);

    if !game_state.is_full(game_info.num_players) {
        return Err(ContractError::Std(cosmwasm_std::StdError::generic_err(
            "not all players registered",
        )));
    }

    game_state.state = BaseState::Shuffle;
    game_state.cur_player_index = 0;
    GAME_STATES.save(deps.storage, game_id, &game_state)?;

    Ok(Response::new()
        .add_attribute("action", "shuffle")
        .add_attribute("game_id", game_id.to_string())
        .add_attribute("state", "Shuffle"))
}

fn execute_player_shuffle(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    game_id: u64,
    proof: Groth16Proof,
    deck: CompressedDeck,
) -> Result<Response, ContractError> {
    let game_info = GAME_INFOS.load(deps.storage, game_id)?;
    let mut game_state = GAME_STATES.load(deps.storage, game_id)?;
    ensure_state!(game_state, game_id, BaseState::Shuffle);

    // Check if it's this player's turn
    let current_player = game_state
        .current_player()
        .ok_or(ContractError::NotPlayersTurn { game_id })?;
    if current_player != &info.sender {
        return Err(ContractError::NotPlayersTurn { game_id });
    }

    // Get old deck for verification
    let old_deck = game_state.deck.compressed();

    // Build public inputs for verification
    let nonce = Uint256::from(game_state.cur_player_index);
    let public_inputs = shuffle_public_input(
        &deck,
        &old_deck,
        &nonce,
        &game_state.agg_pk.x,
        &game_state.agg_pk.y,
    )?;

    // Verify shuffle proof
    let proof_tuple = (proof.a.clone(), proof.b.clone(), proof.c.clone());
    let verified = verify_shuffle_proof(deps.as_ref(), &proof_tuple, &public_inputs, "shuffle_encrypt")?;
    if !verified {
        return Err(ContractError::InvalidProof);
    }

    // Update deck
    game_state.deck.set_from_compressed(deck)?;

    // Update verification counter
    let mut state = VERIFICATION_STATE.load(deps.storage)?;
    state.shuffle_verifications += 1;
    VERIFICATION_STATE.save(deps.storage, &state)?;

    // Advance to next player
    let back_to_start = game_state.advance_player();
    GAME_STATES.save(deps.storage, game_id, &game_state)?;

    Ok(Response::new()
        .add_attribute("action", "player_shuffle")
        .add_attribute("game_id", game_id.to_string())
        .add_attribute("player", info.sender)
        .add_attribute("cur_player_index", game_state.cur_player_index.to_string())
        .add_attribute("back_to_start", back_to_start.to_string()))
}

fn execute_deal_cards_to(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    game_id: u64,
    cards: Binary,
    player_id: u32,
) -> Result<Response, ContractError> {
    let game_info = GAME_INFOS.load(deps.storage, game_id)?;
    if game_info.owner != info.sender {
        return Err(ContractError::Unauthorized);
    }

    let mut game_state = GAME_STATES.load(deps.storage, game_id)?;
    ensure_state!(game_state, game_id, BaseState::Shuffle);

    // Parse cards bitmap
    let cards_bitmap: BitMap256 = cosmwasm_std::from_binary(&cards)?;

    // Validate player_id
    if player_id >= game_info.num_players {
        return Err(ContractError::InvalidPlayer);
    }

    game_state.state = BaseState::Deal;
    game_state.deck.cards_to_deal = cards_bitmap;
    game_state.deck.player_to_deal = player_id;
    game_state.cur_player_index = 0;
    GAME_STATES.save(deps.storage, game_id, &game_state)?;

    Ok(Response::new()
        .add_attribute("action", "deal_cards_to")
        .add_attribute("game_id", game_id.to_string())
        .add_attribute("player_id", player_id.to_string())
        .add_attribute("state", "Deal"))
}

fn execute_player_deal_cards(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    game_id: u64,
    proofs: Vec<Groth16Proof>,
    decrypted_cards: Vec<Card>,
    init_deltas: Vec<CardDelta>,
) -> Result<Response, ContractError> {
    let game_info = GAME_INFOS.load(deps.storage, game_id)?;
    let mut game_state = GAME_STATES.load(deps.storage, game_id)?;
    ensure_state!(game_state, game_id, BaseState::Deal);

    // Check if it's this player's turn (by signing address)
    let current_signer = game_state
        .signing_addrs
        .get(game_state.cur_player_index as usize)
        .ok_or(ContractError::NotPlayersTurn { game_id })?;
    if current_signer != &info.sender {
        return Err(ContractError::NotPlayersTurn { game_id });
    }

    // Verify decrypt proofs
    for (i, proof) in proofs.iter().enumerate() {
        let card = &decrypted_cards[i];
        let delta = &init_deltas[i];

        // Build public inputs for decrypt proof
        // Format: [Y1, Y2, Y3, Y4, pk.x, pk.y, card.x, card.y, delta.delta0, delta.delta1]
        let mut public_inputs = Vec::with_capacity(10);
        // Get first 4 cards from deck as Y values
        public_inputs.push(game_state.deck.x0[0].clone());
        public_inputs.push(game_state.deck.x0[1].clone());
        public_inputs.push(game_state.deck.x0[2].clone());
        public_inputs.push(game_state.deck.x0[3].clone());
        public_inputs.push(game_state.agg_pk.x.clone());
        public_inputs.push(game_state.agg_pk.y.clone());
        public_inputs.push(card.x.clone());
        public_inputs.push(card.y.clone());
        public_inputs.push(delta.delta0.clone());
        public_inputs.push(delta.delta1.clone());

        let proof_tuple = (proof.a.clone(), proof.b.clone(), proof.c.clone());
        let verified = verify_decrypt_proof(deps.as_ref(), &proof_tuple, &public_inputs, "decrypt")?;
        if !verified {
            return Err(ContractError::InvalidProof);
        }
    }

    // Update decrypt records
    let player_id = game_state.deck.player_to_deal as usize;
    for (i, card) in decrypted_cards.iter().enumerate() {
        game_state.deck.y0[i] = card.x.clone();
        game_state.deck.y1[i] = card.y.clone();
        game_state.deck.decrypt_record[i].set(player_id as u32);
    }

    game_state.player_hand[player_id] += decrypted_cards.len() as u32;

    // Update verification counter
    let mut state = VERIFICATION_STATE.load(deps.storage)?;
    state.decrypt_verifications += proofs.len() as u64;
    VERIFICATION_STATE.save(deps.storage, &state)?;

    // Advance to next player
    let back_to_start = game_state.advance_player();
    GAME_STATES.save(deps.storage, game_id, &game_state)?;

    Ok(Response::new()
        .add_attribute("action", "player_deal_cards")
        .add_attribute("game_id", game_id.to_string())
        .add_attribute("player", info.sender)
        .add_attribute("cards_decrypted", decrypted_cards.len().to_string())
        .add_attribute("back_to_start", back_to_start.to_string()))
}

fn execute_open_cards(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    game_id: u64,
    player_id: u32,
    opening: u32,
) -> Result<Response, ContractError> {
    let game_info = GAME_INFOS.load(deps.storage, game_id)?;
    if game_info.owner != info.sender {
        return Err(ContractError::Unauthorized);
    }

    let mut game_state = GAME_STATES.load(deps.storage, game_id)?;
    ensure_state!(game_state, game_id, BaseState::Deal);

    // Validate player_id
    if player_id >= game_info.num_players {
        return Err(ContractError::InvalidPlayer);
    }

    game_state.state = BaseState::Open;
    game_state.opening = opening;
    game_state.cur_player_index = player_id;
    GAME_STATES.save(deps.storage, game_id, &game_state)?;

    Ok(Response::new()
        .add_attribute("action", "open_cards")
        .add_attribute("game_id", game_id.to_string())
        .add_attribute("player_id", player_id.to_string())
        .add_attribute("opening", opening.to_string())
        .add_attribute("state", "Open"))
}

fn execute_player_open_cards(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    game_id: u64,
    cards: Binary,
    proofs: Vec<Groth16Proof>,
    decrypted_cards: Vec<Card>,
) -> Result<Response, ContractError> {
    let _game_info = GAME_INFOS.load(deps.storage, game_id)?;
    let mut game_state = GAME_STATES.load(deps.storage, game_id)?;
    ensure_state!(game_state, game_id, BaseState::Open);

    // Check if it's this player's turn
    let current_player = game_state
        .player_addrs
        .get(game_state.cur_player_index as usize)
        .ok_or(ContractError::NotPlayersTurn { game_id })?;
    if current_player != &info.sender {
        return Err(ContractError::NotPlayersTurn { game_id });
    }

    // Verify decrypt proofs for opened cards
    for (i, proof) in proofs.iter().enumerate() {
        let card = &decrypted_cards[i];

        // Build public inputs for decrypt proof (same as deal)
        let mut public_inputs = Vec::with_capacity(10);
        public_inputs.push(game_state.deck.x0[0].clone());
        public_inputs.push(game_state.deck.x0[1].clone());
        public_inputs.push(game_state.deck.x0[2].clone());
        public_inputs.push(game_state.deck.x0[3].clone());
        public_inputs.push(game_state.agg_pk.x.clone());
        public_inputs.push(game_state.agg_pk.y.clone());
        public_inputs.push(card.x.clone());
        public_inputs.push(card.y.clone());
        public_inputs.push(Uint256::zero()); // delta values for opening
        public_inputs.push(Uint256::zero());

        let proof_tuple = (proof.a.clone(), proof.b.clone(), proof.c.clone());
        let verified = verify_decrypt_proof(deps.as_ref(), &proof_tuple, &public_inputs, "decrypt")?;
        if !verified {
            return Err(ContractError::InvalidProof);
        }
    }

    // Update hand count
    game_state.player_hand[game_state.cur_player_index as usize] = 0;

    // Update verification counter
    let mut state = VERIFICATION_STATE.load(deps.storage)?;
    state.decrypt_verifications += proofs.len() as u64;
    VERIFICATION_STATE.save(deps.storage, &state)?;

    // Reset to beginning
    game_state.cur_player_index = 0;
    game_state.state = BaseState::Complete;
    GAME_STATES.save(deps.storage, game_id, &game_state)?;

    Ok(Response::new()
        .add_attribute("action", "player_open_cards")
        .add_attribute("game_id", game_id.to_string())
        .add_attribute("player", info.sender)
        .add_attribute("cards_opened", decrypted_cards.len().to_string())
        .add_attribute("state", "Complete"))
}

// ========== Minimal Proof Verification Implementations ==========

fn execute_verify_shuffle_proof(
    deps: DepsMut,
    proof: Groth16Proof,
    public_inputs: Vec<Uint256>,
) -> Result<Response, ContractError> {
    let proof_tuple = (proof.a, proof.b, proof.c);
    let verified = verify_shuffle_proof(deps.as_ref(), &proof_tuple, &public_inputs, "shuffle_encrypt")?;

    if !verified {
        return Err(ContractError::InvalidProof);
    }

    let mut state = VERIFICATION_STATE.load(deps.storage)?;
    state.shuffle_verifications += 1;
    VERIFICATION_STATE.save(deps.storage, &state)?;

    Ok(Response::new()
        .add_attribute("action", "verify_shuffle_proof")
        .add_attribute("result", "success")
        .add_attribute("total_shuffle_verifications", state.shuffle_verifications.to_string()))
}

fn execute_verify_decrypt_proof(
    deps: DepsMut,
    proof: Groth16Proof,
    public_inputs: Vec<Uint256>,
) -> Result<Response, ContractError> {
    let proof_tuple = (proof.a, proof.b, proof.c);
    let verified = verify_decrypt_proof(deps.as_ref(), &proof_tuple, &public_inputs, "decrypt")?;

    if !verified {
        return Err(ContractError::InvalidProof);
    }

    let mut state = VERIFICATION_STATE.load(deps.storage)?;
    state.decrypt_verifications += 1;
    VERIFICATION_STATE.save(deps.storage, &state)?;

    Ok(Response::new()
        .add_attribute("action", "verify_decrypt_proof")
        .add_attribute("result", "success")
        .add_attribute("total_decrypt_verifications", state.decrypt_verifications.to_string()))
}

// ========== Queries ==========

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::VerificationCount {} => to_json_binary(&query_verification_count(deps)?),
        QueryMsg::GameState { game_id } => to_json_binary(&query_game_state(deps, game_id)?),
    }
}

fn query_verification_count(deps: Deps) -> StdResult<VerificationCountResponse> {
    let state = VERIFICATION_STATE.load(deps.storage)?;
    Ok(VerificationCountResponse {
        shuffle_verifications: state.shuffle_verifications,
        decrypt_verifications: state.decrypt_verifications,
    })
}

fn query_game_state(deps: Deps, game_id: u64) -> StdResult<GameStateResponse> {
    let game_state = GAME_STATES.load(deps.storage, game_id)?;
    Ok(GameStateResponse {
        game_id,
        state: format!("{:?}", game_state.state),
        cur_player_index: game_state.cur_player_index,
        num_players: game_state.player_addrs.len(),
    })
}

use crate::state::ShuffleGameState;
