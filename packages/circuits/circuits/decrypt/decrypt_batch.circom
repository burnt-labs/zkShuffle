pragma circom 2.0.0;

include "../../node_modules/circomlib/circuits/bitify.circom";
include "../../node_modules/circomlib/circuits/escalarmulfix.circom";
include "../../node_modules/circomlib/circuits/escalarmulany.circom";
include "../../node_modules/circomlib/circuits/babyjub.circom";

// ElGamalDecryptShared: partial decryption using pre-decomposed secret key bits.
// Avoids redundant Num2Bits when decrypting multiple cards with the same sk.
//
// Computes: m = c1 - sk * c0
template ElGamalDecryptShared(numBits) {
    signal input c0[2];
    signal input c1[2];
    signal input skBits[numBits];
    signal output m[2];

    component scalarMul = EscalarMulAny(numBits);
    scalarMul.p[0] <== c0[0];
    scalarMul.p[1] <== c0[1];
    for (var i = 0; i < numBits; i++) {
        scalarMul.e[i] <== skBits[i];
    }

    component adder = BabyAdd();
    adder.x1 <== 0 - scalarMul.out[0]; // negate x for twisted Edwards negation
    adder.y1 <== scalarMul.out[1];
    adder.x2 <== c1[0];
    adder.y2 <== c1[1];
    m[0] <== adder.xout;
    m[1] <== adder.yout;
}

// DecryptBatch: proves partial decryption of `numCards` cards under a single secret key.
//
// One-time cost (shared):
//   - Num2Bits(251): decompose sk
//   - EscalarMulFix(251, Base8): verify pk == sk * G
//
// Per-card cost:
//   - EscalarMulAny(251): compute sk * c0[i]
//   - BabyAdd: compute c1[i] - sk * c0[i]
//
// Public inputs:  Y (4 * numCards values), pkP (2 values)
// Private inputs: skP (1 value)
// Outputs:        out (2 * numCards values)
template DecryptBatch(numCards) {
    var numBits = 251;
    var base[2] = [
        5299619240641551281634865583518297030282874472190772894086521144482721001553,
        16950150798460657717958625567821834550301663161624707787222815936182638968203
    ];

    signal input Y[numCards][4];   // encrypted cards: [c0.x, c0.y, c1.x, c1.y] per card
    signal input pkP[2];           // player's public key
    signal input skP;              // player's secret key (private)
    signal output out[numCards][2]; // decrypted points per card

    // --- Shared: decompose sk to bits (once) ---
    component bitDecomposition = Num2Bits(numBits);
    bitDecomposition.in <== skP;

    // --- Shared: verify pk == sk * G (once) ---
    component deriveKey = EscalarMulFix(numBits, base);
    for (var i = 0; i < numBits; i++) {
        deriveKey.e[i] <== bitDecomposition.out[i];
    }
    pkP[0] === deriveKey.out[0];
    pkP[1] === deriveKey.out[1];

    // --- Per-card: partial ElGamal decryption ---
    component decrypts[numCards];
    for (var i = 0; i < numCards; i++) {
        decrypts[i] = ElGamalDecryptShared(numBits);
        decrypts[i].c0[0] <== Y[i][0];
        decrypts[i].c0[1] <== Y[i][1];
        decrypts[i].c1[0] <== Y[i][2];
        decrypts[i].c1[1] <== Y[i][3];
        for (var j = 0; j < numBits; j++) {
            decrypts[i].skBits[j] <== bitDecomposition.out[j];
        }
        out[i][0] <== decrypts[i].m[0];
        out[i][1] <== decrypts[i].m[1];
    }
}
