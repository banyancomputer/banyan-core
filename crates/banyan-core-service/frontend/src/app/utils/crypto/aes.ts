import { webcrypto } from 'one-webcrypto';
import utils from './utils';
import { DEFAULT_SALT_LENGTH, DEFAULT_SYMM_ALG } from './constants';
import {
    CipherText,
    Msg,
    SymmAlg,
    SymmKey,
    SymmKeyOpts,
} from './types';
import {
    InvalidCipherTextLength,
    InvalidIvLength,
    UnsupportedSymmCrypto,
} from './errors';

/**
 * Encrypt a message with a symmetric key
 * @param msg The message to encrypt
 * @param key The symmetric key to use for encryption
 * @param opts The options for encryption
 * @returns The CipherText (which is just an ArrayBuffer) of form [iv, '.', cipherBytes]
 */
export async function encryptBytes(
    msg: Msg,
    key: SymmKey,
    opts?: Partial<SymmKeyOpts>
): Promise<CipherText> {
    const data = utils.normalizeUtf16ToBuf(msg);
    const alg = opts?.alg || DEFAULT_SYMM_ALG;

    // Note: we only support AES-GCM here
    // If you want support for more symmetric key algorithms, add implementations here
    if (alg !== SymmAlg.AES_GCM) {
        throw UnsupportedSymmCrypto;
    }

    const iv = opts?.iv || utils.randomBuf(DEFAULT_SALT_LENGTH);
    const cipherBuf = await webcrypto.subtle.encrypt(
        {
            name: alg,
            iv,
        },
        key,
        data
    );

    return utils.joinCipherText(iv, cipherBuf);
}

/**
 * Decrypt a CipherText of form [iv, '.', cipherBytes] with a symmetric key
 * @param msg The message to decrypt
 * @param key The symmetric key to use for decryption
 * @param opts The options for decryption
 * returns The decrypted message within an ArrayBuffer
 * throws InvalidIvLength
 * throws InvalidCipherTextLength
 * throws UnsupportedSymmCrypto
 */
export async function decryptBytes(
    msg: Msg,
    key: SymmKey,
    opts?: Partial<SymmKeyOpts>
): Promise<ArrayBuffer> {
    const cipherText = utils.normalizeBase64ToBuf(msg);
    const alg = opts?.alg || DEFAULT_SYMM_ALG;

    // Note: we only support AES-GCM here
    // If you want support for more symmetric key algorithms, add implementations here

    if (alg !== SymmAlg.AES_GCM) {
        throw UnsupportedSymmCrypto;
    }

    const [iv, cipherBytes] = utils.splitCipherText(cipherText);
    // Check lengths
    if (iv.byteLength !== 16) {
        throw InvalidIvLength;
    } else if (cipherBytes.byteLength === 0) {
        throw InvalidCipherTextLength;
    }

    const msgBuff = await webcrypto.subtle.decrypt(
        {
            name: alg,
            iv,
        },
        key,
        cipherBytes
    );

    return msgBuff;
}

/*
 * Encrypt a message with a symmetric key
 * @param msg The message to encrypt
 * @param key The symmetric key to use for encryption
 * @param opts The options for encryption
 * @returns The CipherText (which is just an ArrayBuffer) of form [iv, '.', cipherBytes]
 * @throws UnsupportedSymmCrypto
 * @throws InvalidIvLength
 * @throws InvalidCipherTextLength
 */
export async function encrypt(
    msg: Msg,
    key: SymmKey,
    opts?: Partial<SymmKeyOpts>
): Promise<string> {
    const cipherText = await encryptBytes(msg, key, opts);

    return utils.arrBufToBase64(cipherText);
}

/**
 * Decrypt a CipherText of form [iv, '.', cipherBytes] with a symmetric key
 * @param msg The message to decrypt
 * @param key The symmetric key to use for decryption
 * @param opts The options for decryption
 * @returns The decrypted message
 * @throws UnsupportedSymmCrypto
 * @throws InvalidIvLength
 * @throws InvalidCipherTextLength
 */
export async function decrypt(
    msg: Msg,
    key: SymmKey,
    opts?: Partial<SymmKeyOpts>
): Promise<string> {
    const msgBytes = await decryptBytes(msg, key, opts);

    return utils.arrBufToStr(msgBytes, 16);
}

export default {
    encryptBytes,
    decryptBytes,
    encrypt,
    decrypt,
};
