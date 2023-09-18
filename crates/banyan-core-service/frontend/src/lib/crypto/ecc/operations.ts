import aes from '../aes/index';
import utils, {
  normalizeBase64ToBuf,
  normalizeUnicodeToBuf,
} from '../utils';
import {
  DEFAULT_CHAR_SIZE,
  DEFAULT_ECC_CURVE,
  DEFAULT_HASH_ALG,
  ECC_EXCHANGE_ALG,
  ECC_WRITE_ALG,
} from '../constants';
import hkdf from '../hkdf/index';
import {
  CharSize,
  Msg,
  PrivateKey,
  PublicKey,
  HashAlg,
  SymmKey,
  SymmKeyOpts,
  EccCurve,
  CipherText,
} from '../types';
import { webcrypto } from 'one-webcrypto';

/**
 * Sign a message with an ECSDSA private key
 * @param msg The message to sign
 * @param privateKey The private key to use for signing
 * @param charSize The character size to use for normalization
 * @param hashAlg The hash algorithm to use for signing
 * @returns The signature as an ArrayBuffer
 */
export async function sign(
  msg: Msg,
  privateKey: PrivateKey,
  charSize: CharSize = DEFAULT_CHAR_SIZE,
  hashAlg: HashAlg = DEFAULT_HASH_ALG
): Promise<ArrayBuffer> {
  return webcrypto.subtle.sign(
    { name: ECC_WRITE_ALG, hash: { name: hashAlg } },
    privateKey,
    normalizeUnicodeToBuf(msg, charSize)
  );
}

/**
 * Verify a message with an ECDSA public key
 * @param msg The message to verify
 * @param sig The signature to verify
 * @param publicKey The public key to use for verification
 * @param charSize The character size to use for normalization
 * @param hashAlg The hash algorithm to use for verification
 * @returns A promise that resolves to a boolean indicating whether the signature is valid
 */
export async function verify(
  msg: Msg,
  sig: Msg,
  publicKey: PublicKey,
  charSize: CharSize = DEFAULT_CHAR_SIZE,
  hashAlg: HashAlg = DEFAULT_HASH_ALG
): Promise<boolean> {
  return webcrypto.subtle.verify(
    { name: ECC_WRITE_ALG, hash: { name: hashAlg } },
    publicKey,
    normalizeBase64ToBuf(sig),
    normalizeUnicodeToBuf(msg, charSize)
  );
}

/**
 * Encrypt a message with a shared public key and your own private key
 * @param msg The message to encrypt
 * @param privateKey Your private key
 * @param publicKey The public key to encrypt with
 * @param derivationSalt The salt to use for key derivation
 * @param charSize The character size to use for normalization
 * @param opts The options for encryption
 * @throws {UnsupportedSymmCrypto} If the symmetric algorithm is not supported
 */
export async function encrypt(
  msg: Msg,
  privateKey: PrivateKey,
  publicKey: PublicKey,
  derivationSalt: ArrayBuffer,
  curve: EccCurve = DEFAULT_ECC_CURVE,
  hashAlg: HashAlg = DEFAULT_HASH_ALG,
  charSize: CharSize = DEFAULT_CHAR_SIZE,
  opts?: Partial<SymmKeyOpts>
): Promise<ArrayBuffer> {
  const cipherKey = await getSharedKey(
    privateKey,
    publicKey,
    derivationSalt,
    ['encrypt'],
    'shared-encryption-key',
    curve,
    hashAlg,
    opts
  );
  return aes.encryptBytes(
    normalizeUnicodeToBuf(msg, charSize),
    cipherKey,
    opts
  );
}

/**
 * Decrypt a message with a shared public key and your own private key
 * @param msg The message to decrypt
 * @param privateKey Your private key
 * @param publicKey The public key to decrypt with
 * @param derivationSalt The salt to use for key derivation
 * @param curve The curve to use for key derivation
 * @param charSize The character size to use for normalization
 * @param opts The options for decryption
 * @returns The decrypted message as a string
 * @throws {InvalidCipherTextLength} If the cipher text is not the correct length
 * @throws {UnsupportedSymmCrypto} If the symmetric algorithm is not supported
 */
export async function decrypt(
  msg: Msg,
  privateKey: PrivateKey,
  publicKey: PublicKey,
  derivationSalt: ArrayBuffer,
  curve: EccCurve = DEFAULT_ECC_CURVE,
  hashAlg: HashAlg = DEFAULT_HASH_ALG,
  opts?: Partial<SymmKeyOpts>
): Promise<ArrayBuffer> {
  const cipherKey = await getSharedKey(
    privateKey,
    publicKey,
    derivationSalt,
    ['decrypt'],
    'shared-encryption-key',
    curve,
    hashAlg,
    opts
  );
  return aes.decryptBytes(normalizeBase64ToBuf(msg), cipherKey, opts);
}

/* Key Derivation Helpers */

async function getSharedKey(
  privateKey: PrivateKey,
  publicKey: PublicKey,
  derivationSalt: ArrayBuffer,
  uses: KeyUsage[],
  keyInfo: string,
  curve: EccCurve = DEFAULT_ECC_CURVE,
  hashAlg: HashAlg = DEFAULT_HASH_ALG,
  opts?: Partial<SymmKeyOpts>
): Promise<SymmKey> {
  const bitLength = utils.eccCurveToBitLength(curve);
  return webcrypto.subtle
    .deriveBits(
      { name: ECC_EXCHANGE_ALG, public: publicKey },
      privateKey,
      bitLength
    )
    .then((bits) =>
      hkdf.deriveKey(bits, derivationSalt, keyInfo, hashAlg, uses, opts)
    );
}

export default {
  sign,
  verify,
  encrypt,
  decrypt,
};
