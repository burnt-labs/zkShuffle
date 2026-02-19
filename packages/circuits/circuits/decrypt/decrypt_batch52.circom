pragma circom 2.0.0;

include "./decrypt_batch.circom";

component main {public [Y, pkP]} = DecryptBatch(52);
