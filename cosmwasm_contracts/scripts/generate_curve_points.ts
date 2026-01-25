#!/usr/bin/env bun
/**
 * Generate points on the Baby Jubjub elliptic curve for testing zkShuffle contract.
 *
 * The Baby Jubjub curve equation is: A*x² + y² = 1 + D*x²*y²
 * Where:
 * - A = 168700
 * - D = 168696
 * - Q (field modulus) = 21888242871839275222246405745257275088548364400416034343698204186575808495617
 */

// Baby Jubjub curve parameters
const A = 168700n;
const D = 168696n;
const Q = 21888242871839275222246405745257275088548364400416034343698204186575808495617n;

// Standard Baby Jubjub generator point from the Rust implementation
// This is the base generator point for the Baby Jubjub curve
const GENERATOR: [bigint, bigint] = [
  5299619240641551281634865583518297030282874472190772894086521144482721001553n,
  16950150798460657717958625567821834550301663161624707787222815936182638968203n,
];

// Identity (neutral) element on Baby Jubjub curve
const IDENTITY: [bigint, bigint] = [0n, 1n];

// Modular arithmetic functions
function modAdd(a: bigint, b: bigint, q: bigint = Q): bigint {
  return ((a % q) + (b % q)) % q;
}

function modSub(a: bigint, b: bigint, q: bigint = Q): bigint {
  return ((a % q) - (b % q) + q) % q;
}

function modMul(a: bigint, b: bigint, q: bigint = Q): bigint {
  return ((a % q) * (b % q)) % q;
}

function modPow(a: bigint, exp: bigint, q: bigint = Q): bigint {
  let result = 1n;
  let base = a % q;
  while (exp > 0n) {
    if ((exp & 1n) === 1n) {
      result = modMul(result, base, q);
    }
    exp >>= 1n;
    base = modMul(base, base, q);
  }
  return result;
}

function modInv(a: bigint, q: bigint = Q): bigint {
  // Fermat's little theorem: a^(q-2) ≡ a^(-1) mod q
  return modPow(a, q - 2n, q);
}

function isOnCurve(x: bigint, y: bigint): boolean {
  const xSq = modMul(x, x);
  const ySq = modMul(y, y);

  // A*x² + y²
  const lhs = modAdd(modMul(A, xSq), ySq);

  // 1 + D*x²*y²
  const rhs = modAdd(1n, modMul(modMul(D, xSq), ySq));

  return lhs === rhs;
}

function pointAdd(x1: bigint, y1: bigint, x2: bigint, y2: bigint): [bigint, bigint] {
  // Handle identity elements
  if (x1 === 0n && y1 === 0n) return [x2, y2];
  if (x2 === 0n && y2 === 0n) return [x1, y1];

  const x1x2 = modMul(x1, x2);
  const y1y2 = modMul(y1, y2);
  const dx1x2y1y2 = modMul(D, modMul(x1x2, y1y2));

  const x3Num = modAdd(modMul(x1, y2), modMul(y1, x2));
  const y3Num = modSub(y1y2, modMul(A, x1x2));
  const denomX = modAdd(1n, dx1x2y1y2);
  const denomY = modSub(1n, dx1x2y1y2);

  const invDx = modInv(denomX);
  const invDy = modInv(denomY);

  const x3 = modMul(x3Num, invDx);
  const y3 = modMul(y3Num, invDy);

  return [x3, y3];
}

function scalarMul(k: bigint, x: bigint, y: bigint): [bigint, bigint] {
  let resultX = 0n;
  let resultY = 0n; // Point at infinity
  let currentX = x;
  let currentY = y;

  while (k > 0n) {
    if ((k & 1n) === 1n) {
      [resultX, resultY] = pointAdd(resultX, resultY, currentX, currentY);
    }
    [currentX, currentY] = pointAdd(currentX, currentY, currentX, currentY);
    k >>= 1n;
  }

  return [resultX, resultY];
}

function formatPoint(x: bigint, y: bigint): string {
  return JSON.stringify({ pk_x: x.toString(), pk_y: y.toString() });
}

function generateRandomPoint(): [bigint, bigint] {
  // Randomly choose between generator and identity
  // In practice, you'd use scalar multiplication of the generator
  return Math.random() > 0.5 ? [...GENERATOR] : [...IDENTITY];
}

function main() {
  const args = process.argv.slice(2);
  const numPoints = args.length > 0 ? parseInt(args[0]) : 5;

  console.log("Baby Jubjub Curve Point Generator");
  console.log("=".repeat(60));
  console.log(`Generating ${numPoints} points on the Baby Jubjub curve:\n`);

  // First, validate our base points
  console.log("Validating base points:");
  const [genX, genY] = GENERATOR;
  const [idX, idY] = IDENTITY;

  console.log(`  Generator: ${isOnCurve(genX, genY) ? "✓ VALID" : "✗ INVALID"}`);
  console.log(`  Identity: ${isOnCurve(idX, idY) ? "✓ VALID" : "✗ INVALID"}`);
  console.log();

  console.log(`Generating ${numPoints} points using scalar multiplication:\n`);

  const [seedX, seedY] = GENERATOR;

  for (let i = 0; i < numPoints; i++) {
    // Use different scalars to derive different points
    const scalar = BigInt(i + 2);
    const [derivedX, derivedY] = scalarMul(scalar, seedX, seedY);

    // Verify the point is on the curve
    if (!isOnCurve(derivedX, derivedY)) {
      console.error(`ERROR: Generated point (${derivedX}, ${derivedY}) is not on the curve!`);
      process.exit(1);
    }

    console.log(`Point ${i + 1} (scalar=${scalar}):`);
    console.log(`  x: ${derivedX}`);
    console.log(`  y: ${derivedY}`);
    console.log(`  JSON: ${formatPoint(derivedX, derivedY)}`);
    console.log();
  }

  console.log("All points validated and ready for testing!");
}

main();
