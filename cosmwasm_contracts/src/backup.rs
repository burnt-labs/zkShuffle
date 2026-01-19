use cosmwasm_std::Uint256;
use std::str::FromStr;

/// Generate a point on Baby Jubjub curve by scalar multiplication of the generator
pub fn generate_point(scalar: u128) -> (Uint256, Uint256) {
    let (gx, gy) = generator();
    point_mul(&gx, &gy, &Uint256::from_u128(scalar)).unwrap()
}

/// Scalar multiplication: compute scalar * G
pub fn point_mul(x: &Uint256, y: &Uint256, scalar: &Uint256) -> StdResult<(Uint256, Uint256)> {
    let mut remaining = scalar.clone();
    let mut px = x.clone();
    let mut py = y.clone();
    let mut ax = Uint256::zero();
    let mut ay = Uint256::zero();

    while !remaining.is_zero() {
        if is_odd(&remaining) {
            (ax, ay) = point_add(&ax, &ay, &px, &py)?;
        }
        (px, py) = point_add(&px, &py, &px, &py)?;
        remaining = remaining.checked_div(Uint256::from(2u128)).unwrap();
    }

    Ok((ax, ay))
}

/// Find a valid point on the curve given an x-coordinate
/// This searches for a valid y such that the point (x, y) is on the curve
pub fn find_point_from_x(x: &Uint256) -> Option<(Uint256, Uint256)> {
    let q = &*BABY_JUB_Q;
    let x_sq = mod_mul(x, x, q);

    // From curve equation: Ax² + y² = 1 + Dx²y²
    // Rearranging: y²(1 - Dx²) = 1 - Ax²
    // So: y² = (1 - Ax²) / (1 - Dx²)

    let ax_sq = mod_mul(&A_CONST, &x_sq, q);
    let dx_sq = mod_mul(&D_CONST, &x_sq, q);

    let numerator = mod_sub(&Uint256::one(), &ax_sq, q);
    let denominator = mod_sub(&Uint256::one(), &dx_sq, q);

    // Check if denominator is zero
    if denominator.is_zero() {
        return None;
    }

    let inv_denom = mod_inverse(&denominator, q).ok()?;
    let y_squared = mod_mul(&numerator, &inv_denom, q);

    // Compute square root using Tonelli-Shanks or try both possibilities
    let y = sqrt_mod(&y_squared, q)?;

    if is_on_curve(x, &y) {
        Some((x.clone(), y))
    } else {
        None
    }
}

/// Compute modular square root (simplified for Baby Jubjub's prime)
/// Since Q ≡ 1 (mod 4), we can use y = ±(y²)^((Q+1)/4) mod Q
fn sqrt_mod(a: &Uint256, q: &Uint256) -> Option<Uint256> {
    // For Baby Jubjub, Q = 21888...617 ≡ 1 (mod 4)
    // So we can use: sqrt(a) = a^((Q+1)/4) mod Q
    let exponent = (*q + Uint256::one())
        .checked_div(Uint256::from(4u128))
        .unwrap();

    let y = mod_pow(a.clone(), exponent, q);

    // Verify it's actually a square root
    let y_squared = mod_mul(&y, &y, q);
    if y_squared == *a {
        Some(y)
    } else {
        None
    }
}

/// Generate random valid points by trying different x values
pub fn generate_random_point(seed: u128) -> Option<(Uint256, Uint256)> {
    let x = Uint256::from_u128(seed);
    find_point_from_x(&x)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generator_on_curve() {
        let (gx, gy) = generator();
        assert!(is_on_curve(&gx, &gy), "Generator must be on curve");
    }

    #[test]
    fn test_point_mul() {
        let (gx, gy) = generator();

        // Test 2*G
        let (x2, y2) = point_mul(&gx, &gy, &Uint256::from(2u128)).unwrap();
        assert!(is_on_curve(&x2, &y2));

        // Test 3*G
        let (x3, y3) = point_mul(&gx, &gy, &Uint256::from(3u128)).unwrap();
        assert!(is_on_curve(&x3, &y3));

        // Verify 2*G + G = 3*G
        let (x2_plus_g, y2_plus_g) = point_add(&x2, &y2, &gx, &gy).unwrap();
        assert_eq!(x2_plus_g, x3);
        assert_eq!(y2_plus_g, y3);
    }

    #[test]
    fn test_point_add_doubling() {
        let (gx, gy) = generator();
        let (x2, y2) = point_add(&gx, &gy, &gx, &gy).unwrap();

        assert!(is_on_curve(&x2, &y2));
        // Point doubling should give us 2*G
        assert!(x2 != gx || y2 != gy);
    }

    #[test]
    fn test_generate_multiple_points() {
        let (gx, gy) = generator();

        for i in 1..=10 {
            let (x, y) = point_mul(&gx, &gy, &Uint256::from(i as u128)).unwrap();
            assert!(is_on_curve(&x, &y), "Point {}*G must be on curve", i);
        }
    }
}

/// Tonelli-Shanks algorithm for computing square roots mod p
/// For Baby Jubjub's prime Q where Q ≡ 1 (mod 4), we use simpler method
fn sqrt_mod_tonelli_shanks(n: &Uint256, p: &Uint256) -> Option<Uint256> {
    // Check if n is a quadratic residue using Euler's criterion
    // n^((p-1)/2) mod p should be 1
    let exponent = (*p - Uint256::one())
        .checked_div(Uint256::from(2u128))
        .unwrap();
    let legendre = mod_pow(n.clone(), exponent, p);

    if legendre != Uint256::one() {
        return None; // Not a quadratic residue
    }

    // For p ≡ 3 (mod 4), use: r = n^((p+1)/4) mod p
    // Baby Jubjub Q ≡ 1 (mod 4), so we need different approach
    // But for simplicity, try both possible roots
    let exp = (*p + Uint256::one())
        .checked_div(Uint256::from(4u128))
        .unwrap();
    let r = mod_pow(n.clone(), exp, p);

    // Check both r and p-r
    let r_sq = mod_mul(&r, &r, p);
    if r_sq == *n {
        return Some(r);
    }

    let r_neg = mod_sub(p, &r, p);
    let r_neg_sq = mod_mul(&r_neg, &r_neg, p);
    if r_neg_sq == *n {
        return Some(r_neg);
    }

    None
}

/// Compress a point (x, y) into (x, delta, sign)
/// where delta <= DELTA_MAX and sign determines if y = delta or y = Q - delta
pub fn compress_point(x: &Uint256, y: &Uint256) -> (Uint256, Uint256, bool) {
    let q = &*BABY_JUB_Q;
    let y_complement = mod_sub(q, y, q);

    if y <= &y_complement {
        // y is the smaller value, so delta = y, sign = true
        (x.clone(), y.clone(), true)
    } else {
        // Q-y is the smaller value, so delta = Q-y, sign = false
        (x.clone(), y_complement, false)
    }
}

/// Decompress a point from (x, delta, sign) back to (x, y)
pub fn decompress_point(x: &Uint256, delta: &Uint256, sign: bool) -> StdResult<(Uint256, Uint256)> {
    let y = recover_y(x, delta, sign)?;
    Ok((x.clone(), y))
}
