import type { SignerWithAddress } from '@nomiclabs/hardhat-ethers/dist/src/signer-with-address';
import { assert } from '@zk-shuffle/proof/src/shuffle/utilities';
import { elgamalDecrypt, elgamalEncrypt } from '@zk-shuffle/proof/src/shuffle/plaintext';
import { ethers } from 'hardhat';
import { resolve } from 'path';
import { readFileSync } from 'fs';
import { build_circuit } from '../utils/utils';
const buildBabyjub = require('circomlibjs').buildBabyjub;
const snarkjs = require('snarkjs');

const NUM_CARDS = 2;

describe('Batch decrypt circuit tests', function () {

    before(async function () {
        const signers: SignerWithAddress[] = await ethers.getSigners();
        this.signers = signers;
        this.admin = signers[0];
        console.log("Building decryptBatch2 circuit...");
        await build_circuit("circuits/tests", "decryptBatch2");
    });

    it('Batch decrypt of 2 cards produces correct results', async function () {
        const babyjub = await buildBabyjub();

        const wasmFile = resolve(__dirname, '../wasm/decryptBatch2.wasm');
        const zkeyFile = resolve(__dirname, '../zkey/decryptBatch2.zkey');
        const vkey = await snarkjs.zKey.exportVerificationKey(
            new Uint8Array(Buffer.from(readFileSync(zkeyFile)))
        );

        // Setup: secret key, public key
        const sk = 7n;
        const pk = babyjub.mulPointEscalar(babyjub.Base8, sk);
        const pkStr = [babyjub.F.toString(pk[0]), babyjub.F.toString(pk[1])];

        // Encrypt 2 cards with different message scalars
        const messageScalars = [3n, 17n];
        const randomness = [42n, 99n];
        const Y: string[][] = [];
        const expectedDecryptions: any[] = [];

        for (let i = 0; i < NUM_CARDS; i++) {
            const ic0 = babyjub.mulPointEscalar(babyjub.Base8, 0); // identity
            const ic1 = babyjub.mulPointEscalar(babyjub.Base8, messageScalars[i]);
            const encryption = elgamalEncrypt(babyjub, ic0, ic1, randomness[i], pk);
            const c0 = encryption[0];
            const c1 = encryption[1];

            Y.push([
                babyjub.F.toString(c0[0]),
                babyjub.F.toString(c0[1]),
                babyjub.F.toString(c1[0]),
                babyjub.F.toString(c1[1]),
            ]);

            // Compute expected decryption in plaintext
            const decrypted = elgamalDecrypt(babyjub, c0, c1, sk);
            expectedDecryptions.push(decrypted);

            // Verify plaintext decryption recovers the original message
            assert(
                babyjub.F.toString(decrypted[0]) === babyjub.F.toString(ic1[0]),
                `Plaintext decryption failed for card ${i}`
            );
            assert(
                babyjub.F.toString(decrypted[1]) === babyjub.F.toString(ic1[1]),
                `Plaintext decryption failed for card ${i}`
            );
        }

        // Generate batch proof
        const proveOutput = await snarkjs.groth16.fullProve(
            { Y, pkP: pkStr, skP: sk.toString() },
            wasmFile,
            zkeyFile,
        );

        // Verify proof
        assert(
            await snarkjs.groth16.verify(vkey, proveOutput.publicSignals, proveOutput.proof),
            'Off-chain batch decrypt verification failed.'
        );

        // Verify outputs match expected decryptions
        // Public signals layout: [out[0][0], out[0][1], out[1][0], out[1][1], Y[0][0..3], Y[1][0..3], pkP[0], pkP[1]]
        for (let i = 0; i < NUM_CARDS; i++) {
            const outX = proveOutput.publicSignals[i * 2];
            const outY = proveOutput.publicSignals[i * 2 + 1];
            assert(
                outX === babyjub.F.toString(expectedDecryptions[i][0]),
                `Output X mismatch for card ${i}`
            );
            assert(
                outY === babyjub.F.toString(expectedDecryptions[i][1]),
                `Output Y mismatch for card ${i}`
            );
        }

        console.log("Batch decrypt proof verified successfully for", NUM_CARDS, "cards.");
    });

    it('Batch decrypt works with multi-party partial decryption', async function () {
        const babyjub = await buildBabyjub();

        const wasmFile = resolve(__dirname, '../wasm/decryptBatch2.wasm');
        const zkeyFile = resolve(__dirname, '../zkey/decryptBatch2.zkey');
        const vkey = await snarkjs.zKey.exportVerificationKey(
            new Uint8Array(Buffer.from(readFileSync(zkeyFile)))
        );

        // Two players with separate keys
        const sk1 = 11n;
        const sk2 = 23n;
        const pk1 = babyjub.mulPointEscalar(babyjub.Base8, sk1);
        const pk2 = babyjub.mulPointEscalar(babyjub.Base8, sk2);
        const aggregatePk = babyjub.addPoint(pk1, pk2);

        // Encrypt 2 cards under the aggregate key
        const messageScalars = [5n, 10n];
        const randomness = [31n, 77n];
        const cards: { c0: any, c1: any, ic1: any }[] = [];

        for (let i = 0; i < NUM_CARDS; i++) {
            const ic0 = babyjub.mulPointEscalar(babyjub.Base8, 0);
            const ic1 = babyjub.mulPointEscalar(babyjub.Base8, messageScalars[i]);
            const encryption = elgamalEncrypt(babyjub, ic0, ic1, randomness[i], aggregatePk);
            cards.push({ c0: encryption[0], c1: encryption[1], ic1 });
        }

        // Player 1 batch-decrypts (partial)
        const Y1: string[][] = [];
        for (let i = 0; i < NUM_CARDS; i++) {
            Y1.push([
                babyjub.F.toString(cards[i].c0[0]),
                babyjub.F.toString(cards[i].c0[1]),
                babyjub.F.toString(cards[i].c1[0]),
                babyjub.F.toString(cards[i].c1[1]),
            ]);
        }

        const proof1 = await snarkjs.groth16.fullProve(
            {
                Y: Y1,
                pkP: [babyjub.F.toString(pk1[0]), babyjub.F.toString(pk1[1])],
                skP: sk1.toString(),
            },
            wasmFile,
            zkeyFile,
        );
        assert(
            await snarkjs.groth16.verify(vkey, proof1.publicSignals, proof1.proof),
            'Player 1 batch proof verification failed.'
        );

        // Extract partially decrypted c1 values from Player 1's output
        // c0 stays the same, c1 is updated to the partial decryption result
        const Y2: string[][] = [];
        for (let i = 0; i < NUM_CARDS; i++) {
            const partialC1X = proof1.publicSignals[i * 2];
            const partialC1Y = proof1.publicSignals[i * 2 + 1];
            Y2.push([
                babyjub.F.toString(cards[i].c0[0]), // c0 unchanged
                babyjub.F.toString(cards[i].c0[1]),
                partialC1X,                          // c1 updated
                partialC1Y,
            ]);
        }

        // Player 2 batch-decrypts (final)
        const proof2 = await snarkjs.groth16.fullProve(
            {
                Y: Y2,
                pkP: [babyjub.F.toString(pk2[0]), babyjub.F.toString(pk2[1])],
                skP: sk2.toString(),
            },
            wasmFile,
            zkeyFile,
        );
        assert(
            await snarkjs.groth16.verify(vkey, proof2.publicSignals, proof2.proof),
            'Player 2 batch proof verification failed.'
        );

        // After both players have decrypted, the output should be the original message points
        for (let i = 0; i < NUM_CARDS; i++) {
            const finalX = proof2.publicSignals[i * 2];
            const finalY = proof2.publicSignals[i * 2 + 1];
            assert(
                finalX === babyjub.F.toString(cards[i].ic1[0]),
                `Final decryption X mismatch for card ${i}`
            );
            assert(
                finalY === babyjub.F.toString(cards[i].ic1[1]),
                `Final decryption Y mismatch for card ${i}`
            );
        }

        console.log("Multi-party batch decrypt verified: 2 players, 2 cards.");
    });
});
