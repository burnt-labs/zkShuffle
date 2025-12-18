use std::convert::TryFrom;
use std::str::FromStr;

use cosmwasm_std::{StdError, StdResult, Uint256, Uint512};
use once_cell::sync::Lazy;

const A_CONST: Uint256 = Uint256::from_u128(168700);
const D_CONST: Uint256 = Uint256::from_u128(168696);

pub static BABY_JUB_Q: Lazy<Uint256> = Lazy::new(|| {
    Uint256::from_str(
        "21888242871839275222246405745257275088548364400416034343698204186575808495617",
    )
    .unwrap()
});

pub static DELTA_MAX: Lazy<Uint256> = Lazy::new(|| {
    Uint256::from_str(
        "10944121435919637611123202872628637544274182200208017171849102093287904247808",
    )
    .unwrap()
});

pub fn is_on_curve(x: &Uint256, y: &Uint256) -> bool {
    let q = &*BABY_JUB_Q;
    let x_sq = mod_mul(x, x, q);
    let y_sq = mod_mul(y, y, q);
    let lhs = mod_add(&mod_mul(&A_CONST, &x_sq, q), &y_sq, q);
    let rhs = mod_add(
        &Uint256::one(),
        &mod_mul(&mod_mul(&D_CONST, &x_sq, q), &y_sq, q),
        q,
    );
    lhs == rhs
}

pub fn point_add(
    x1: &Uint256,
    y1: &Uint256,
    x2: &Uint256,
    y2: &Uint256,
) -> StdResult<(Uint256, Uint256)> {
    let q = &*BABY_JUB_Q;
    if x1.is_zero() && y1.is_zero() {
        return Ok((x2.clone(), y2.clone()));
    }
    if x2.is_zero() && y2.is_zero() {
        return Ok((x1.clone(), y1.clone()));
    }
    let x1x2 = mod_mul(x1, x2, q);
    let y1y2 = mod_mul(y1, y2, q);
    let dx1x2y1y2 = mod_mul(&D_CONST, &mod_mul(&x1x2, &y1y2, q), q);
    let x3_num = mod_add(&mod_mul(x1, y2, q), &mod_mul(y1, x2, q), q);
    let y3_num = mod_sub(&y1y2, &mod_mul(&A_CONST, &x1x2, q), q);
    let denom_x = mod_add(&Uint256::one(), &dx1x2y1y2, q);
    let denom_y = mod_sub(&Uint256::one(), &dx1x2y1y2, q);
    let inv_dx = mod_inverse(&denom_x, q)?;
    let inv_dy = mod_inverse(&denom_y, q)?;
    let x3 = mod_mul(&x3_num, &inv_dx, q);
    let y3 = mod_mul(&y3_num, &inv_dy, q);
    Ok((x3, y3))
}

pub fn recover_y(x: &Uint256, delta: &Uint256, sign: bool) -> StdResult<Uint256> {
    if delta > &*DELTA_MAX {
        return Err(StdError::generic_err("delta out of range"));
    }
    if !is_on_curve(x, delta) {
        return Err(StdError::generic_err("point not on curve"));
    }
    if sign {
        Ok(delta.clone())
    } else {
        Ok(mod_sub(&*BABY_JUB_Q, delta, &*BABY_JUB_Q))
    }
}

pub fn curve_q() -> &'static Uint256 {
    &*BABY_JUB_Q
}

pub fn mul_mod_q(a: &Uint256, b: &Uint256) -> Uint256 {
    mod_mul(a, b, &*BABY_JUB_Q)
}

fn mod_add(a: &Uint256, b: &Uint256, modulus: &Uint256) -> Uint256 {
    let sum = Uint512::from(*a) + Uint512::from(*b);
    reduce(sum, modulus)
}

fn mod_sub(a: &Uint256, b: &Uint256, modulus: &Uint256) -> Uint256 {
    if a >= b {
        let diff = Uint512::from(*a) - Uint512::from(*b);
        reduce(diff, modulus)
    } else {
        let diff = Uint512::from(*b) - Uint512::from(*a);
        let tmp = Uint512::from(*modulus) - diff;
        reduce(tmp, modulus)
    }
}

fn mod_mul(a: &Uint256, b: &Uint256, modulus: &Uint256) -> Uint256 {
    let product = Uint512::from(*a) * Uint512::from(*b);
    reduce(product, modulus)
}

fn mod_inverse(value: &Uint256, modulus: &Uint256) -> StdResult<Uint256> {
    if value.is_zero() {
        return Err(StdError::generic_err("inverse undefined"));
    }
    let exponent = *modulus - Uint256::from_u128(2);
    Ok(mod_pow(value.clone(), exponent, modulus))
}

fn is_odd(n: &Uint256) -> bool {
    // compute n % 2 by comparing n to floor(n/2)*2
    let half = n.checked_div(Uint256::from(2u128)).unwrap();
    let double = half.checked_mul(Uint256::from(2u128)).unwrap();
    n != &double
}

fn mod_pow(mut base: Uint256, mut exp: Uint256, modulus: &Uint256) -> Uint256 {
    let two = Uint256::from(2u128);
    let mut result = Uint256::one();

    // reduce base modulo modulus safely
    base = reduce(Uint512::from(base), modulus);

    while !exp.is_zero() {
        if is_odd(&exp) {
            result = mod_mul(&result, &base, modulus);
        }
        // exp >>= 1  -> exp = exp / 2
        exp = exp.checked_div(two).unwrap();
        base = mod_mul(&base, &base, modulus);
    }
    result
}

fn reduce(value: Uint512, modulus: &Uint256) -> Uint256 {
    let modulus512 = Uint512::from(*modulus);
    let reduced = value % modulus512;
    Uint256::try_from(reduced).unwrap()
}
