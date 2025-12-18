#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_json_binary, Addr, Binary, CosmosMsg, Deps, DepsMut, Env, MessageInfo, Response, StdError,
    StdResult, Storage, Uint256, WasmMsg,
};
use cw2::set_contract_version;

use crate::bitmaps::BitMap256;
use crate::curve;
use crate::deck::{card_index_from_x1, shuffle_public_input};
use crate::error::ContractError;
use crate::msg::{
    AggregatedPkResponse, CardValueResponse, DecryptRecordResponse, DeckResponse, ExecuteMsg,
    GameInfoResponse, GameStateResponse, InstantiateMsg, NumCardsResponse, PlayerIndexResponse,
    QueryMsg,
};
use crate::state::{
    Config, GameInfo, ShuffleGameState, ACTIVE_GAMES, CONFIG, GAME_INFOS, GAME_STATES,
    NEXT_CALLBACK,
};
use crate::types::{
    BaseState, BitMap256 as BitMap, Card, CardDelta, CompressedDeck, DeckConfig, Groth16Proof,
};

const CONTRACT_NAME: &str = "crates.io:zk-shuffle";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let config = Config {
        decrypt_verifier: deps.api.addr_validate(&msg.decrypt_verifier)?,
        deck5_verifier: deps.api.addr_validate(&msg.deck5_verifier)?,
        deck30_verifier: deps.api.addr_validate(&msg.deck30_verifier)?,
        deck52_verifier: deps.api.addr_validate(&msg.deck52_verifier)?,
        next_game_id: 0,
    };
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new().add_attribute("action", "instantiate"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::CreateGame {
            num_players,
            deck_config,
        } => execute_create_game(deps, info, num_players, deck_config),
        ExecuteMsg::Register { game_id, callback } => {
            execute_register(deps, info, game_id, callback)
        }
        ExecuteMsg::PlayerRegister {
            game_id,
            signing_addr,
            pk_x,
            pk_y,
        } => execute_player_register(deps, info, game_id, signing_addr, pk_x, pk_y),
        ExecuteMsg::Shuffle { game_id, callback } => {
            execute_shuffle(deps, info, game_id, callback)
        }
        ExecuteMsg::PlayerShuffle { game_id, proof, deck } => {
            execute_player_shuffle(deps, info, game_id, proof, deck)
        }
        ExecuteMsg::DealCardsTo {
            game_id,
            cards,
            player_id,
            callback,
        } => execute_deal_cards_to(deps, info, game_id, cards, player_id, callback),
        ExecuteMsg::PlayerDealCards {
            game_id,
            proofs,
            decrypted_cards,
            init_deltas,
        } => execute_player_deal_cards(
            deps,
            info,
            game_id,
            proofs,
            decrypted_cards,
            init_deltas,
        ),
        ExecuteMsg::OpenCards {
            game_id,
            player_id,
            opening,
            callback,
        } => execute_open_cards(deps, info, game_id, player_id, opening, callback),
        ExecuteMsg::PlayerOpenCards {
            game_id,
            cards,
            proofs,
            decrypted_cards,
        } => execute_player_open_cards(deps, info, game_id, cards, proofs, decrypted_cards),
        ExecuteMsg::EndGame { game_id } => execute_end_game(deps, info, game_id),
        ExecuteMsg::Error { game_id, callback } => execute_error(deps, info, game_id, callback),
    }
}

fn execute_create_game(
    mut deps: DepsMut,
    info: MessageInfo,
    num_players: u8,
    deck_config: DeckConfig,
) -> Result<Response, ContractError> {
    if num_players == 0 {
        return Err(ContractError::InvalidPlayer);
    }

    let mut config = CONFIG.load(deps.storage)?;
    let game_id = config.next_game_id + 1;
    config.next_game_id = game_id;

    let encrypt_verifier = config.deck_verifier(deck_config);
    let num_cards = deck_config.num_cards() as u8;

    let game_info = GameInfo {
        deck_config,
        num_cards,
        num_players,
        encrypt_verifier,
    };

    let state = ShuffleGameState::new(deck_config, num_players);

    GAME_INFOS.save(deps.storage, game_id, &game_info)?;
    GAME_STATES.save(deps.storage, game_id, &state)?;
    ACTIVE_GAMES.save(deps.storage, game_id, &info.sender)?;
    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new()
        .add_attribute("action", "create_game")
        .add_attribute("game_id", game_id.to_string())
        .add_attribute("owner", info.sender))
}

fn execute_register(
    mut deps: DepsMut,
    info: MessageInfo,
    game_id: u64,
    callback: Option<Binary>,
) -> Result<Response, ContractError> {
    ensure_game_owner(deps.storage, game_id, &info.sender)?;
    let mut state = load_state(deps.storage, game_id)?;
    ensure_state(game_id, &state, BaseState::Created)?;

    state.state = BaseState::Registration;
    GAME_STATES.save(deps.storage, game_id, &state)?;
    store_callback(deps.storage, game_id, callback)?;

    Ok(Response::new()
        .add_attribute("action", "register")
        .add_attribute("game_id", game_id.to_string()))
}

fn execute_player_register(
    mut deps: DepsMut,
    info: MessageInfo,
    game_id: u64,
    signing_addr: String,
    pk_x: Uint256,
    pk_y: Uint256,
) -> Result<Response, ContractError> {
    let signing_addr = deps.api.addr_validate(&signing_addr)?;
    let game_info = load_info(deps.storage, game_id)?;
    let mut state = load_state(deps.storage, game_id)?;
    ensure_state(game_id, &state, BaseState::Registration)?;

    if state.player_addrs.len() as u8 >= game_info.num_players {
        return Err(ContractError::GameFull { game_id });
    }

    if !curve::is_on_curve(&pk_x, &pk_y) {
        return Err(ContractError::InvalidCardSelection);
    }

    let pid = state.player_addrs.len() as u32;
    state.player_addrs.push(info.sender.clone());
    state.signing_addrs.push(signing_addr);
    state.player_pk_x.push(pk_x.clone());
    state.player_pk_y.push(pk_y.clone());

    if pid == 0 {
        state.aggregate_pk_x = pk_x;
        state.aggregate_pk_y = pk_y;
    } else {
        let (x, y) =
            curve::point_add(&state.aggregate_pk_x, &state.aggregate_pk_y, &pk_x, &pk_y)?;
        state.aggregate_pk_x = x;
        state.aggregate_pk_y = y;
    }

    let mut resp = Response::new()
        .add_attribute("action", "player_register")
        .add_attribute("game_id", game_id.to_string())
        .add_attribute("player_index", pid.to_string());

    if state.player_addrs.len() as u8 == game_info.num_players {
        state.nonce = curve::mul_mod_q(&state.aggregate_pk_x, &state.aggregate_pk_y);
        GAME_STATES.save(deps.storage, game_id, &state)?;
        if let Some(msg) = take_callback_msg(deps.storage, game_id)? {
            resp = resp.add_message(msg);
        }
    } else {
        GAME_STATES.save(deps.storage, game_id, &state)?;
    }

    Ok(resp)
}

fn execute_shuffle(
    mut deps: DepsMut,
    info: MessageInfo,
    game_id: u64,
    callback: Option<Binary>,
) -> Result<Response, ContractError> {
    ensure_game_owner(deps.storage, game_id, &info.sender)?;
    let mut state = load_state(deps.storage, game_id)?;
    if state.cur_player_index != 0 {
        return Err(ContractError::NotPlayersTurn { game_id });
    }
    state.state = BaseState::Shuffle;
    GAME_STATES.save(deps.storage, game_id, &state)?;
    store_callback(deps.storage, game_id, callback)?;

    Ok(Response::new()
        .add_attribute("action", "shuffle")
        .add_attribute("game_id", game_id.to_string()))
}

fn execute_player_shuffle(
    mut deps: DepsMut,
    info: MessageInfo,
    game_id: u64,
    _proof: Groth16Proof,
    deck: CompressedDeck,
) -> Result<Response, ContractError> {
    let game_info = load_info(deps.storage, game_id)?;
    let mut state = load_state(deps.storage, game_id)?;
    ensure_state(game_id, &state, BaseState::Shuffle)?;
    ensure_player_turn(&state, &info.sender, game_id)?;

    let old_compressed = state.deck.compressed();
    shuffle_public_input(
        &deck,
        &old_compressed,
        &state.nonce,
        &state.aggregate_pk_x,
        &state.aggregate_pk_y,
    )?;
    state.deck.set_from_compressed(deck)?;

    let num_players = state.player_addrs.len() as u32;
    state.cur_player_index += 1;
    if state.cur_player_index >= num_players {
        state.cur_player_index = 0;
    }

    let mut resp = Response::new()
        .add_attribute("action", "player_shuffle")
        .add_attribute("game_id", game_id.to_string())
        .add_attribute("next_player", state.cur_player_index.to_string());

    GAME_STATES.save(deps.storage, game_id, &state)?;

    if state.cur_player_index == 0 {
        if let Some(msg) = take_callback_msg(deps.storage, game_id)? {
            resp = resp.add_message(msg);
        }
    }

    Ok(resp)
}

fn execute_deal_cards_to(
    mut deps: DepsMut,
    info: MessageInfo,
    game_id: u64,
    cards: BitMap,
    player_id: u32,
    callback: Option<Binary>,
) -> Result<Response, ContractError> {
    ensure_game_owner(deps.storage, game_id, &info.sender)?;
    let game_info = load_info(deps.storage, game_id)?;
    let mut state = load_state(deps.storage, game_id)?;
    if state.cur_player_index != 0 {
        return Err(ContractError::NotPlayersTurn { game_id });
    }
    if player_id as u8 >= game_info.num_players {
        return Err(ContractError::InvalidPlayer);
    }

    state.state = BaseState::Deal;
    state.deck.cards_to_deal = cards;
    state.deck.player_to_deal = player_id;
    if player_id == 0 && game_info.num_players > 1 {
        state.cur_player_index = 1;
    } else {
        state.cur_player_index = 0;
    }

    GAME_STATES.save(deps.storage, game_id, &state)?;
    store_callback(deps.storage, game_id, callback)?;

    Ok(Response::new()
        .add_attribute("action", "deal_cards")
        .add_attribute("game_id", game_id.to_string())
        .add_attribute("target_player", player_id.to_string()))
}

fn execute_player_deal_cards(
    mut deps: DepsMut,
    info: MessageInfo,
    game_id: u64,
    proofs: Vec<Groth16Proof>,
    decrypted_cards: Vec<Card>,
    init_deltas: Vec<CardDelta>,
) -> Result<Response, ContractError> {
    let game_info = load_info(deps.storage, game_id)?;
    let mut state = load_state(deps.storage, game_id)?;
    ensure_state(game_id, &state, BaseState::Deal)?;
    ensure_player_turn(&state, &info.sender, game_id)?;

    let num_to_deal =
        state
            .deck
            .cards_to_deal
            .member_count_up_to(game_info.num_cards as u32);
    if proofs.len() as u32 != num_to_deal
        || decrypted_cards.len() as u32 != num_to_deal
        || init_deltas.len() as u32 != num_to_deal
    {
        return Err(ContractError::InvalidCardSelection);
    }

    let mut counter = 0usize;
    for cid in 0..(game_info.num_cards as usize) {
        if state.deck.cards_to_deal.get(cid as u32) {
            update_decrypted_card(
                &mut state,
                cid,
                &proofs[counter],
                &decrypted_cards[counter],
                &init_deltas[counter],
            )?;
            counter += 1;
            if counter == num_to_deal as usize {
                break;
            }
        }
    }

    let num_players = state.player_addrs.len() as u32;
    state.cur_player_index += 1;
    if state.cur_player_index == state.deck.player_to_deal {
        state.cur_player_index += 1;
    }
    if state.cur_player_index >= num_players {
        state.cur_player_index = 0;
        if let Some(entry) = state
            .player_hand
            .get_mut(state.deck.player_to_deal as usize)
        {
            *entry += num_to_deal;
        }
    }

    let mut resp = Response::new()
        .add_attribute("action", "player_deal")
        .add_attribute("game_id", game_id.to_string())
        .add_attribute("next_player", state.cur_player_index.to_string());

    let finished_round = state.cur_player_index == 0;
    GAME_STATES.save(deps.storage, game_id, &state)?;

    if finished_round {
        if let Some(msg) = take_callback_msg(deps.storage, game_id)? {
            resp = resp.add_message(msg);
        }
    }

    Ok(resp)
}

fn execute_open_cards(
    mut deps: DepsMut,
    info: MessageInfo,
    game_id: u64,
    player_id: u32,
    opening: u8,
    callback: Option<Binary>,
) -> Result<Response, ContractError> {
    ensure_game_owner(deps.storage, game_id, &info.sender)?;
    let game_info = load_info(deps.storage, game_id)?;
    let mut state = load_state(deps.storage, game_id)?;
    if player_id as u8 >= game_info.num_players {
        return Err(ContractError::InvalidPlayer);
    }
    if opening as u32 > state.player_hand[player_id as usize] {
        return Err(ContractError::InvalidCardSelection);
    }

    state.state = BaseState::Open;
    state.opening = opening;
    state.cur_player_index = player_id;
    store_callback(deps.storage, game_id, callback)?;
    GAME_STATES.save(deps.storage, game_id, &state)?;

    Ok(Response::new()
        .add_attribute("action", "open_cards")
        .add_attribute("game_id", game_id.to_string())
        .add_attribute("player", player_id.to_string()))
}

fn execute_player_open_cards(
    mut deps: DepsMut,
    info: MessageInfo,
    game_id: u64,
    cards: BitMap,
    proofs: Vec<Groth16Proof>,
    decrypted_cards: Vec<Card>,
) -> Result<Response, ContractError> {
    let game_info = load_info(deps.storage, game_id)?;
    let mut state = load_state(deps.storage, game_id)?;
    ensure_state(game_id, &state, BaseState::Open)?;
    ensure_player_turn(&state, &info.sender, game_id)?;

    let number_to_open = cards.member_count_up_to(game_info.num_cards as u32);
    if number_to_open != state.opening as u32 {
        return Err(ContractError::InvalidCardSelection);
    }
    if proofs.len() as u32 != number_to_open || decrypted_cards.len() as u32 != number_to_open {
        return Err(ContractError::InvalidCardSelection);
    }

    let dummy_delta = CardDelta {
        delta0: Uint256::zero(),
        delta1: Uint256::zero(),
    };
    let mut counter = 0usize;
    for cid in 0..(game_info.num_cards as usize) {
        if cards.get(cid as u32) {
            update_decrypted_card(
                &mut state,
                cid,
                &proofs[counter],
                &decrypted_cards[counter],
                &dummy_delta,
            )?;
            counter += 1;
            if counter == number_to_open as usize {
                break;
            }
        }
    }

    if let Some(hand) = state
        .player_hand
        .get_mut(state.cur_player_index as usize)
    {
        *hand -= number_to_open;
    }
    state.opening = 0;
    state.cur_player_index = 0;

    let mut resp = Response::new()
        .add_attribute("action", "player_open")
        .add_attribute("game_id", game_id.to_string());

    GAME_STATES.save(deps.storage, game_id, &state)?;

    if let Some(msg) = take_callback_msg(deps.storage, game_id)? {
        resp = resp.add_message(msg);
    }

    Ok(resp)
}

fn execute_end_game(
    mut deps: DepsMut,
    info: MessageInfo,
    game_id: u64,
) -> Result<Response, ContractError> {
    ensure_game_owner(deps.storage, game_id, &info.sender)?;
    let mut state = load_state(deps.storage, game_id)?;
    state.state = BaseState::Complete;
    GAME_STATES.save(deps.storage, game_id, &state)?;
    ACTIVE_GAMES.remove(deps.storage, game_id);
    NEXT_CALLBACK.remove(deps.storage, game_id);

    Ok(Response::new()
        .add_attribute("action", "end_game")
        .add_attribute("game_id", game_id.to_string()))
}

fn execute_error(
    mut deps: DepsMut,
    info: MessageInfo,
    game_id: u64,
    callback: Option<Binary>,
) -> Result<Response, ContractError> {
    ensure_game_owner(deps.storage, game_id, &info.sender)?;
    let mut state = load_state(deps.storage, game_id)?;
    state.state = BaseState::GameError;
    GAME_STATES.save(deps.storage, game_id, &state)?;
    store_callback(deps.storage, game_id, callback)?;

    let mut resp = Response::new()
        .add_attribute("action", "error")
        .add_attribute("game_id", game_id.to_string());

    if let Some(msg) = take_callback_msg(deps.storage, game_id)? {
        resp = resp.add_message(msg);
    }

    Ok(resp)
}

fn update_decrypted_card(
    state: &mut ShuffleGameState,
    card_index: usize,
    _proof: &Groth16Proof,
    decrypted_card: &Card,
    init_delta: &CardDelta,
) -> Result<(), ContractError> {
    if state.deck.decrypt_record[card_index].get(state.cur_player_index) {
        return Err(ContractError::AlreadyDecrypted);
    }

    if state.deck.decrypt_record[card_index].is_zero() {
        let selector0 = state.deck.selector0.get(card_index as u32);
        let selector1 = state.deck.selector1.get(card_index as u32);
        state.deck.y0[card_index] = curve::recover_y(
            &state.deck.x0[card_index],
            &init_delta.delta0,
            selector0,
        )?;
        state.deck.y1[card_index] = curve::recover_y(
            &state.deck.x1[card_index],
            &init_delta.delta1,
            selector1,
        )?;
    }

    state.deck.x1[card_index] = decrypted_card.x.clone();
    state.deck.y1[card_index] = decrypted_card.y.clone();
    state.deck.decrypt_record[card_index].set(state.cur_player_index);
    Ok(())
}

fn ensure_game_owner(
    storage: &mut dyn Storage,
    game_id: u64,
    sender: &Addr,
) -> Result<(), ContractError> {
    let owner = ACTIVE_GAMES
        .may_load(storage, game_id)?
        .ok_or(ContractError::GameNotFound { game_id })?;
    if &owner != sender {
        return Err(ContractError::Unauthorized);
    }
    Ok(())
}

fn ensure_player_turn(
    state: &ShuffleGameState,
    sender: &Addr,
    game_id: u64,
) -> Result<(), ContractError> {
    let idx = state.cur_player_index as usize;
    let is_player = state
        .player_addrs
        .get(idx)
        .map(|addr| addr == sender)
        .unwrap_or(false);
    let is_signer = state
        .signing_addrs
        .get(idx)
        .map(|addr| addr == sender)
        .unwrap_or(false);
    if !(is_player || is_signer) {
        return Err(ContractError::NotPlayersTurn { game_id });
    }
    Ok(())
}

fn ensure_state(
    game_id: u64,
    state: &ShuffleGameState,
    expected: BaseState,
) -> Result<(), ContractError> {
    if state.state != expected {
        return Err(ContractError::InvalidState {
            game_id,
            expected,
            actual: state.state,
        });
    }
    Ok(())
}

fn load_info(storage: &mut dyn Storage, game_id: u64) -> Result<GameInfo, ContractError> {
    GAME_INFOS
        .may_load(storage, game_id)?
        .ok_or(ContractError::GameNotFound { game_id })
}

fn load_state(storage: &mut dyn Storage, game_id: u64) -> Result<ShuffleGameState, ContractError> {
    GAME_STATES
        .may_load(storage, game_id)?
        .ok_or(ContractError::GameNotFound { game_id })
}

fn store_callback(
    storage: &mut dyn Storage,
    game_id: u64,
    callback: Option<Binary>,
) -> StdResult<()> {
    if let Some(msg) = callback {
        NEXT_CALLBACK.save(storage, game_id, &msg)
    } else {
        NEXT_CALLBACK.remove(storage, game_id);
        Ok(())
    }
}

fn take_callback_msg(storage: &mut dyn Storage, game_id: u64) -> Result<Option<CosmosMsg>, ContractError> {
    let callback = NEXT_CALLBACK.may_load(storage, game_id)?;
    if let Some(msg) = callback {
        let target = ACTIVE_GAMES
            .may_load(storage, game_id)?
            .ok_or(ContractError::GameNotFound { game_id })?;
        NEXT_CALLBACK.remove(storage, game_id);
        let cosmos = CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: target.to_string(),
            msg,
            funds: vec![],
        });
        Ok(Some(cosmos))
    } else {
        Ok(None)
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GameInfo { game_id } => to_json_binary(&query_game_info(deps, game_id)?),
        QueryMsg::GameState { game_id } => to_json_binary(&query_game_state(deps, game_id)?),
        QueryMsg::NumCards { game_id } => to_json_binary(&query_num_cards(deps, game_id)?),
        QueryMsg::CurPlayerIndex { game_id } => {
            to_json_binary(&query_cur_player_index(deps, game_id)?)
        }
        QueryMsg::DecryptRecord { game_id, card_index } => {
            to_json_binary(&query_decrypt_record(deps, game_id, card_index)?)
        }
        QueryMsg::AggregatedPk { game_id } => {
            to_json_binary(&query_aggregated_pk(deps, game_id)?)
        }
        QueryMsg::Deck { game_id } => to_json_binary(&query_deck(deps, game_id)?),
        QueryMsg::PlayerIndex { game_id, address } => {
            to_json_binary(&query_player_index(deps, game_id, address)?)
        }
        QueryMsg::CardValue { game_id, card_index } => {
            to_json_binary(&query_card_value(deps, game_id, card_index)?)
        }
    }
}

fn query_game_info(deps: Deps, game_id: u64) -> StdResult<GameInfoResponse> {
    let info = GAME_INFOS.load(deps.storage, game_id)?;
    Ok(GameInfoResponse {
        num_cards: info.num_cards,
        num_players: info.num_players,
        encrypt_verifier: info.encrypt_verifier.to_string(),
        deck_config: info.deck_config,
    })
}

fn query_game_state(deps: Deps, game_id: u64) -> StdResult<GameStateResponse> {
    let info = GAME_INFOS.load(deps.storage, game_id)?;
    let state = GAME_STATES.load(deps.storage, game_id)?;
    Ok(GameStateResponse {
        state: state.state,
        opening: state.opening,
        cur_player_index: state.cur_player_index,
        aggregate_pk_x: state.aggregate_pk_x,
        aggregate_pk_y: state.aggregate_pk_y,
        nonce: state.nonce,
        player_addrs: state.player_addrs.iter().map(|a| a.to_string()).collect(),
        signing_addrs: state.signing_addrs.iter().map(|a| a.to_string()).collect(),
        deck_config: info.deck_config,
        player_hand: state.player_hand.clone(),
    })
}

fn query_num_cards(deps: Deps, game_id: u64) -> StdResult<NumCardsResponse> {
    let info = GAME_INFOS.load(deps.storage, game_id)?;
    Ok(NumCardsResponse {
        count: info.num_cards as u32,
    })
}

fn query_cur_player_index(deps: Deps, game_id: u64) -> StdResult<PlayerIndexResponse> {
    let state = GAME_STATES.load(deps.storage, game_id)?;
    Ok(PlayerIndexResponse {
        index: Some(state.cur_player_index),
    })
}

fn query_decrypt_record(
    deps: Deps,
    game_id: u64,
    card_index: u32,
) -> StdResult<DecryptRecordResponse> {
    let state = GAME_STATES.load(deps.storage, game_id)?;
    let idx = card_index as usize;
    if idx >= state.deck.decrypt_record.len() {
        return Err(StdError::generic_err("invalid card index"));
    }
    Ok(DecryptRecordResponse {
        bitmap: state.deck.decrypt_record[idx].clone(),
    })
}

fn query_aggregated_pk(deps: Deps, game_id: u64) -> StdResult<AggregatedPkResponse> {
    let state = GAME_STATES.load(deps.storage, game_id)?;
    Ok(AggregatedPkResponse {
        px: state.aggregate_pk_x,
        py: state.aggregate_pk_y,
    })
}

fn query_deck(deps: Deps, game_id: u64) -> StdResult<DeckResponse> {
    let state = GAME_STATES.load(deps.storage, game_id)?;
    Ok(DeckResponse {
        x0: state.deck.x0.clone(),
        x1: state.deck.x1.clone(),
        y0: state.deck.y0.clone(),
        y1: state.deck.y1.clone(),
        selector0: state.deck.selector0.clone(),
        selector1: state.deck.selector1.clone(),
        cards_to_deal: state.deck.cards_to_deal.clone(),
        player_to_deal: state.deck.player_to_deal,
    })
}

fn query_player_index(
    deps: Deps,
    game_id: u64,
    address: String,
) -> StdResult<PlayerIndexResponse> {
    let addr = deps.api.addr_validate(&address)?;
    let state = GAME_STATES.load(deps.storage, game_id)?;
    let mut index = None;
    for (idx, player_addr) in state.player_addrs.iter().enumerate() {
        if player_addr == &addr {
            index = Some(idx as u32);
            break;
        }
    }
    if index.is_none() {
        for (idx, signing) in state.signing_addrs.iter().enumerate() {
            if signing == &addr {
                index = Some(idx as u32);
                break;
            }
        }
    }
    Ok(PlayerIndexResponse { index })
}

fn query_card_value(
    deps: Deps,
    game_id: u64,
    card_index: u32,
) -> StdResult<CardValueResponse> {
    let info = GAME_INFOS.load(deps.storage, game_id)?;
    let state = GAME_STATES.load(deps.storage, game_id)?;
    let idx = card_index as usize;
    if idx >= state.deck.x1.len() {
        return Ok(CardValueResponse { value: None });
    }
    let decrypted = state.deck.decrypt_record[idx]
        .member_count_up_to(info.num_players as u32)
        == info.num_players as u32;
    if !decrypted {
        return Ok(CardValueResponse { value: None });
    }

    let value = card_index_from_x1(&state.deck.x1[idx], info.deck_config);
    Ok(CardValueResponse { value })
}
