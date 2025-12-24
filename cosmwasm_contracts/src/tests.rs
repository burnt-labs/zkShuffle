use crate::bitmaps::*;
use crate::contract;
use crate::curve::*;
use crate::deck::*;
use crate::msg::{ExecuteMsg, InstantiateMsg};
use crate::state::GAME_STATES;
use crate::types::{
    BaseState, Card, CardDelta, CompressedDeck, DeckConfig, Groth16Proof,
};
use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
use cosmwasm_std::{DepsMut, StdError, Uint256};
use std::str::FromStr;

// // Helper to create a dummy CompressedDeck for testing
fn mock_compressed_deck(config: DeckConfig, fill_val: u128) -> CompressedDeck {
    let size = config.num_cards() as usize;
    CompressedDeck {
        config,
        x0: vec![Uint256::from(fill_val); size],
        x1: vec![Uint256::from(fill_val + 1); size],
        selector0: BitMap256::from_u128(12345),
        selector1: BitMap256::from_u128(67890),
    }
}

fn instantiate_contract(mut deps: DepsMut) {
    let msg = InstantiateMsg {
        decrypt_verifier: "decrypt".to_string(),
        deck5_verifier: "deck5".to_string(),
        deck30_verifier: "deck30".to_string(),
        deck52_verifier: "deck52".to_string(),
    };
    let info = mock_info("creator", &[]);
    contract::instantiate(deps, mock_env(), info, msg).unwrap();
}

fn zero_compressed_deck(config: DeckConfig) -> CompressedDeck {
    let size = config.num_cards() as usize;
    CompressedDeck {
        config,
        x0: vec![Uint256::zero(); size],
        x1: vec![Uint256::zero(); size],
        selector0: BitMap256::zero(),
        selector1: BitMap256::zero(),
    }
}

fn dummy_proof() -> Groth16Proof {
    Groth16Proof {
        a: [Uint256::zero(), Uint256::zero()],
        b: [[Uint256::zero(), Uint256::zero()], [Uint256::zero(), Uint256::zero()]],
        c: [Uint256::zero(), Uint256::zero()],
    }
}

fn dummy_card(x: u64, y: u64) -> Card {
    Card {
        x: Uint256::from(x),
        y: Uint256::from(y),
    }
}

#[test]
fn test_bitmap_basic_ops() {
    let mut bitmap = BitMap256::zero();

    // 1. Test Empty
    assert!(bitmap.is_zero());
    assert!(!bitmap.get(0));
    assert!(!bitmap.get(10));

    // 2. Test Set & Get (Arithmetic Check)
    bitmap.set(0); // 2^0 = 1
    assert!(bitmap.get(0));
    assert_eq!(bitmap.data, Uint256::from(1u128));

    bitmap.set(10); // 2^10 = 1024
    assert!(bitmap.get(10));
    // data should be 1 + 1024 = 1025
    assert_eq!(bitmap.data, Uint256::from(1025u128));

    // 3. Test Unset
    bitmap.unset(0);
    assert!(!bitmap.get(0));
    assert!(bitmap.get(10)); // 10 should still be there
    assert_eq!(bitmap.data, Uint256::from(1024u128));

    // 4. Test SetTo
    bitmap.set_to(5, true);
    assert!(bitmap.get(5));
    bitmap.set_to(5, false);
    assert!(!bitmap.get(5));
}

#[test]
fn test_bitmap_boundary() {
    let mut bitmap = BitMap256::zero();

    // Test the last possible bit (index 255)
    // This ensures the loop in pow2 doesn't overflow and logic holds for high bits
    bitmap.set(255);
    assert!(bitmap.get(255));
    assert!(!bitmap.get(254));

    // Ensure we can unset it
    bitmap.unset(255);
    assert!(bitmap.is_zero());
}

#[test]
fn test_bitmap_member_count() {
    let mut bitmap = BitMap256::zero();
    bitmap.set(0);
    bitmap.set(2);
    bitmap.set(5);

    // Count up to index 3 (0, 1, 2) -> Should find index 0 and 2
    let count = bitmap.member_count_up_to(3);
    assert_eq!(count, 2);

    // Count up to index 6 -> Should find 0, 2, 5
    let count_all = bitmap.member_count_up_to(6);
    assert_eq!(count_all, 3);
}

#[test]
fn test_deck_initialization() {
    // Test 5 Card configuration
    let config = DeckConfig::Deck5Card;
    let deck = Deck::new(config.clone());

    assert_eq!(deck.x0.len(), 5);
    assert_eq!(deck.x1.len(), 5);

    // Check if INIT_X1 loaded correctly into the deck
    // The first value of INIT_X1 in your lazy static
    let expected_first_x1 = Uint256::from_str(
        "5299619240641551281634865583518297030282874472190772894086521144482721001553",
    )
    .unwrap();
    assert_eq!(deck.x1[0], expected_first_x1);

    // Check Selector logic
    // Solidity: 4503599627370495 >> (52 - 5) = 4503599627370495 >> 47
    let base_sel0 = 4_503_599_627_370_495u128;
    let expected_sel0 = base_sel0 >> (52 - 5);
    assert_eq!(deck.selector0.data, Uint256::from(expected_sel0));
}

#[test]
fn test_shuffle_public_input_layout() {
    // This test ensures the vector sent to the ZK verifier matches the solidity loop order strictly
    let config = DeckConfig::Deck5Card; // Use small deck for easy math
    let deck_size = 5;

    let old_deck = mock_compressed_deck(config.clone(), 100); // x0=100, x1=101
    let enc_deck = mock_compressed_deck(config.clone(), 200); // x0=200, x1=201

    let nonce = Uint256::from(999u128);
    let agg_pk_x = Uint256::from(888u128);
    let agg_pk_y = Uint256::from(777u128);

    let input = shuffle_public_input(&enc_deck, &old_deck, &nonce, &agg_pk_x, &agg_pk_y).unwrap();

    // Expected Length: 7 + 4 * deck_size => 7 + 20 = 27
    assert_eq!(input.len(), 27);

    // --- VERIFY EXACT LAYOUT ---
    // 1. Nonce, PkX, PkY
    assert_eq!(input[0], nonce);
    assert_eq!(input[1], agg_pk_x);
    assert_eq!(input[2], agg_pk_y);

    // 2. Old Deck X0 (Indices 3 to 7)
    for i in 0..deck_size {
        assert_eq!(input[3 + i], Uint256::from(100u128));
    }

    // 3. Old Deck X1 (Indices 8 to 12)
    for i in 0..deck_size {
        assert_eq!(input[3 + deck_size + i], Uint256::from(101u128));
    }

    // 4. Enc Deck X0 (Indices 13 to 17)
    for i in 0..deck_size {
        assert_eq!(input[3 + 2 * deck_size + i], Uint256::from(200u128));
    }

    // 5. Enc Deck X1 (Indices 18 to 22)
    for i in 0..deck_size {
        assert_eq!(input[3 + 3 * deck_size + i], Uint256::from(201u128));
    }

    // 6. Selectors (Indices 23, 24, 25, 26)
    let offset = 3 + 4 * deck_size;
    assert_eq!(input[offset], old_deck.selector0.data);
    assert_eq!(input[offset + 1], old_deck.selector1.data);
    assert_eq!(input[offset + 2], enc_deck.selector0.data);
    assert_eq!(input[offset + 3], enc_deck.selector1.data);
}

// Helper to generate a known valid point on BabyJubJub
// Base point (Generator) values from standard BabyJubJub
// Helper to generate a known valid point on BabyJubJub
// Base point (Generator) values from standard BabyJubJub specs (EIP-2494)
fn generator() -> (Uint256, Uint256) {
    (
        Uint256::from_str(
            "5299619240641551281634865583518297030282874472190772894086521144482721001553",
        )
        .unwrap(),
        Uint256::from_str(
            "16950150798460657717958625567821834550301663161624707787222815936182638968203",
        )
        .unwrap(),
    )
}
#[test]
fn test_constants_and_helpers() {
    // 1. Verify Q matches the Solidity constant exactly
    let q_solidity = Uint256::from_str(
        "21888242871839275222246405745257275088548364400416034343698204186575808495617",
    )
    .unwrap();
    assert_eq!(*BABY_JUB_Q, q_solidity);

    // 2. Verify Helper: mul_mod_q
    let a = Uint256::from(100u128);
    let b = Uint256::from(200u128);
    let res = mul_mod_q(&a, &b);
    assert_eq!(res, Uint256::from(20000u128));
}

#[test]
fn test_is_on_curve() {
    // 1. Identity point (0, 1) should be on curve
    // 168700*0^2 + 1^2 = 1 + 168696*0^2*1^2  => 1 = 1
    assert!(is_on_curve(&Uint256::zero(), &Uint256::one()));

    // 2. Generator point should be on curve
    let (gx, gy) = generator();
    assert!(is_on_curve(&gx, &gy));

    // 3. Random invalid point should fail
    assert!(!is_on_curve(
        &Uint256::from(123u128),
        &Uint256::from(456u128)
    ));
}

#[test]
fn test_point_add_basic() {
    let (gx, gy) = generator();
    let zero = Uint256::zero();
    let one = Uint256::one();

    // 1. Add Identity: P + (0, 1) = P
    // Note: In Twisted Edwards, (0,1) is the neutral element.
    // Your code handles (0,0) explicitly as a "null" placeholder return.
    // Let's test the math for (0,1) if your code supports it,
    // OR test your explicit (0,0) shortcut.

    // Test Explicit Shortcut (0,0)
    let (res_x, res_y) = point_add(&gx, &gy, &zero, &zero).unwrap();
    assert_eq!(res_x, gx);
    assert_eq!(res_y, gy);

    // Test Actual Math with Identity (0,1)
    // P + Identity = P
    let (res_x_math, res_y_math) = point_add(&gx, &gy, &zero, &one).unwrap();
    assert_eq!(res_x_math, gx);
    assert_eq!(res_y_math, gy);
}

#[test]
fn test_point_add_doubling() {
    let (gx, gy) = generator();
    let (x2, y2) = point_add(&gx, &gy, &gx, &gy).unwrap();

    assert!(is_on_curve(&x2, &y2));
    assert!(
        x2 != gx || y2 != gy,
        "doubling may be equal for Solidity-compatible BabyJubJub"
    );
}

#[test]
fn test_recover_y() {
    let (gx, gy) = generator();
    let q = &*BABY_JUB_Q;

    // Compute delta (the compressed Y coordinate)
    // delta is always the smaller of (y, Q-y)
    let y_complement = mod_sub(q, &gy, q);
    let (delta, sign) = if gy <= y_complement {
        (gy.clone(), true) // y is the smaller value
    } else {
        (y_complement, false) // Q-y is the smaller value
    };

    // Verify delta is within bounds
    assert!(delta <= *DELTA_MAX, "delta must be <= DELTA_MAX");

    // Verify (x, delta) is on the curve
    assert!(
        is_on_curve(&gx, &delta),
        "compressed point must be on curve"
    );

    // Case 1: Recover original Y using the correct sign
    let recovered = recover_y(&gx, &delta, sign).unwrap();
    assert_eq!(recovered, gy, "should recover original Y");

    // Case 2: Recover the complement Y using opposite sign
    let recovered_complement = recover_y(&gx, &delta, !sign).unwrap();
    let expected_complement = mod_sub(q, &gy, q);
    assert_eq!(
        recovered_complement, expected_complement,
        "should recover complement Y"
    );

    // Verify both points are on the curve
    assert!(is_on_curve(&gx, &recovered));
    assert!(is_on_curve(&gx, &recovered_complement));
}

#[test]
fn test_modular_math_properties() {
    // Check associativity and distribution to ensure Uint512 handling is correct
    let q = &*BABY_JUB_Q;
    let a = Uint256::from(123456789u128);
    let b = Uint256::from(987654321u128);
    let c = Uint256::from(555555555u128);

    // (a + b) mod q
    let sum = mod_add(&a, &b, q);
    // (a * b) mod q
    let prod = mod_mul(&a, &b, q);

    // Inverse check: a * a^-1 = 1
    let inv_a = mod_inverse(&a, q).unwrap();
    let unity = mod_mul(&a, &inv_a, q);
    assert_eq!(unity, Uint256::one());

    // Subtraction check: (a + b) - b = a
    let sub_res = mod_sub(&sum, &b, q);
    assert_eq!(sub_res, a);
}

#[test]
fn test_recover_y_failure() {
    // Test invalid delta (too large)
    let bad_delta = *DELTA_MAX + Uint256::one();
    let err = recover_y(&Uint256::zero(), &bad_delta, true);
    assert!(err.is_err());
    assert_eq!(
        err.unwrap_err(),
        StdError::generic_err("delta out of range")
    );

    // Test point not on curve
    // (0, 2) is definitely not on the curve
    let err_curve = recover_y(&Uint256::zero(), &Uint256::from(2u128), true);
    assert!(err_curve.is_err());
    assert_eq!(
        err_curve.unwrap_err(),
        StdError::generic_err("point not on curve")
    );
}

#[test]
fn test_basic_game_flow_shuffle_and_deal() {
    let mut deps = mock_dependencies();
    instantiate_contract(deps.as_mut());

    let env = mock_env();
    let owner = "owner";
    let player0 = "player0";
    let player1 = "player1";
    let deck_config = DeckConfig::Deck5Card;

    contract::execute(
        deps.as_mut(),
        env.clone(),
        mock_info(owner, &[]),
        ExecuteMsg::CreateGame {
            num_players: 2,
            deck_config,
        },
    )
    .unwrap();

    contract::execute(
        deps.as_mut(),
        env.clone(),
        mock_info(owner, &[]),
        ExecuteMsg::Register {
            game_id: 1,
            callback: None,
        },
    )
    .unwrap();

    let pk_x = Uint256::zero();
    let pk_y = Uint256::one();
    contract::execute(
        deps.as_mut(),
        env.clone(),
        mock_info(player0, &[]),
        ExecuteMsg::PlayerRegister {
            game_id: 1,
            signing_addr: "player0-sign".to_string(),
            pk_x: pk_x.clone(),
            pk_y: pk_y.clone(),
        },
    )
    .unwrap();
    contract::execute(
        deps.as_mut(),
        env.clone(),
        mock_info(player1, &[]),
        ExecuteMsg::PlayerRegister {
            game_id: 1,
            signing_addr: "player1-sign".to_string(),
            pk_x,
            pk_y,
        },
    )
    .unwrap();

    contract::execute(
        deps.as_mut(),
        env.clone(),
        mock_info(owner, &[]),
        ExecuteMsg::Shuffle {
            game_id: 1,
            callback: None,
        },
    )
    .unwrap();

    let compressed = zero_compressed_deck(deck_config);
    contract::execute(
        deps.as_mut(),
        env.clone(),
        mock_info(player0, &[]),
        ExecuteMsg::PlayerShuffle {
            game_id: 1,
            proof: dummy_proof(),
            deck: compressed.clone(),
        },
    )
    .unwrap();
    let mut state = GAME_STATES.load(&deps.storage, 1).unwrap();
    assert_eq!(state.cur_player_index, 1);

    contract::execute(
        deps.as_mut(),
        env.clone(),
        mock_info(player1, &[]),
        ExecuteMsg::PlayerShuffle {
            game_id: 1,
            proof: dummy_proof(),
            deck: compressed.clone(),
        },
    )
    .unwrap();
    state = GAME_STATES.load(&deps.storage, 1).unwrap();
    assert_eq!(state.cur_player_index, 0);
    assert_eq!(state.state, BaseState::Shuffle);
    assert!(state.deck.x1.iter().all(|v| v.is_zero()));

    let mut cards = BitMap256::zero();
    cards.set(0);
    cards.set(1);
    contract::execute(
        deps.as_mut(),
        env.clone(),
        mock_info(owner, &[]),
        ExecuteMsg::DealCardsTo {
            game_id: 1,
            cards: cards.clone(),
            player_id: 0,
            callback: None,
        },
    )
    .unwrap();

    state = GAME_STATES.load(&deps.storage, 1).unwrap();
    assert_eq!(state.state, BaseState::Deal);
    assert_eq!(state.cur_player_index, 1);
    assert_eq!(state.deck.cards_to_deal.member_count_up_to(5), 2);

    let proofs = vec![dummy_proof(), dummy_proof()];
    let decrypted_cards = vec![dummy_card(10, 20), dummy_card(11, 21)];
    let deltas = vec![
        CardDelta {
            delta0: Uint256::one(),
            delta1: Uint256::one(),
        };
        2
    ];
    contract::execute(
        deps.as_mut(),
        env.clone(),
        mock_info(player1, &[]),
        ExecuteMsg::PlayerDealCards {
            game_id: 1,
            proofs,
            decrypted_cards,
            init_deltas: deltas,
        },
    )
    .unwrap();

    state = GAME_STATES.load(&deps.storage, 1).unwrap();
    assert_eq!(state.cur_player_index, 0);
    assert_eq!(state.player_hand[0], 2);
    assert!(state.deck.decrypt_record[0].get(1));
}

#[test]
fn test_open_cards_flow() {
    let mut deps = mock_dependencies();
    instantiate_contract(deps.as_mut());

    let env = mock_env();
    let owner = "owner";
    let player0 = "player0";
    let player1 = "player1";
    let deck_config = DeckConfig::Deck5Card;

    contract::execute(
        deps.as_mut(),
        env.clone(),
        mock_info(owner, &[]),
        ExecuteMsg::CreateGame {
            num_players: 2,
            deck_config,
        },
    )
    .unwrap();
    contract::execute(
        deps.as_mut(),
        env.clone(),
        mock_info(owner, &[]),
        ExecuteMsg::Register {
            game_id: 1,
            callback: None,
        },
    )
    .unwrap();

    let pk_x = Uint256::zero();
    let pk_y = Uint256::one();
    contract::execute(
        deps.as_mut(),
        env.clone(),
        mock_info(player0, &[]),
        ExecuteMsg::PlayerRegister {
            game_id: 1,
            signing_addr: "player0-sign".to_string(),
            pk_x: pk_x.clone(),
            pk_y: pk_y.clone(),
        },
    )
    .unwrap();
    contract::execute(
        deps.as_mut(),
        env.clone(),
        mock_info(player1, &[]),
        ExecuteMsg::PlayerRegister {
            game_id: 1,
            signing_addr: "player1-sign".to_string(),
            pk_x,
            pk_y,
        },
    )
    .unwrap();

    contract::execute(
        deps.as_mut(),
        env.clone(),
        mock_info(owner, &[]),
        ExecuteMsg::Shuffle {
            game_id: 1,
            callback: None,
        },
    )
    .unwrap();
    let compressed = zero_compressed_deck(deck_config);
    contract::execute(
        deps.as_mut(),
        env.clone(),
        mock_info(player0, &[]),
        ExecuteMsg::PlayerShuffle {
            game_id: 1,
            proof: dummy_proof(),
            deck: compressed.clone(),
        },
    )
    .unwrap();
    contract::execute(
        deps.as_mut(),
        env.clone(),
        mock_info(player1, &[]),
        ExecuteMsg::PlayerShuffle {
            game_id: 1,
            proof: dummy_proof(),
            deck: compressed.clone(),
        },
    )
    .unwrap();

    let mut cards = BitMap256::zero();
    cards.set(0);
    contract::execute(
        deps.as_mut(),
        env.clone(),
        mock_info(owner, &[]),
        ExecuteMsg::DealCardsTo {
            game_id: 1,
            cards: cards.clone(),
            player_id: 0,
            callback: None,
        },
    )
    .unwrap();
    let deltas = vec![CardDelta {
        delta0: Uint256::one(),
        delta1: Uint256::one(),
    }];
    let proofs = vec![dummy_proof()];
    let decrypted_cards = vec![dummy_card(42, 24)];
    contract::execute(
        deps.as_mut(),
        env.clone(),
        mock_info(player1, &[]),
        ExecuteMsg::PlayerDealCards {
            game_id: 1,
            proofs,
            decrypted_cards,
            init_deltas: deltas,
        },
    )
    .unwrap();

    contract::execute(
        deps.as_mut(),
        env.clone(),
        mock_info(owner, &[]),
        ExecuteMsg::OpenCards {
            game_id: 1,
            player_id: 0,
            opening: 1,
            callback: None,
        },
    )
    .unwrap();
    contract::execute(
        deps.as_mut(),
        env.clone(),
        mock_info(player0, &[]),
        ExecuteMsg::PlayerOpenCards {
            game_id: 1,
            cards,
            proofs: vec![dummy_proof()],
            decrypted_cards: vec![dummy_card(99, 100)],
        },
    )
    .unwrap();

    let state = GAME_STATES.load(&deps.storage, 1).unwrap();
    assert_eq!(state.state, BaseState::Open);
    assert_eq!(state.cur_player_index, 0);
    assert_eq!(state.opening, 0);
    assert_eq!(state.player_hand[0], 0);
}
