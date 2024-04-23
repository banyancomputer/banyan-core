import { CharSize, EccCurve, HashAlg, SymmAlg, SymmKeyLength } from './types';

// This library is highly opinionated towards ECC. If you want to use RSA, you'll need to change these values, and
// refactor the code to support RSA.
export const ECC_EXCHANGE_ALG = 'ECDH';
export const ECC_WRITE_ALG = 'ECDSA';

export const DEFAULT_ECC_CURVE = EccCurve.P_384;
export const DEFAULT_SALT_LENGTH = 16; // bytes -- idk why I have to specify it this way, but I do

export const DEFAULT_SYMM_ALG = SymmAlg.AES_GCM;
export const DEFAULT_SYMM_KEY_LENGTH = SymmKeyLength.B256;

export const DEFAULT_HASH_ALG = HashAlg.SHA_256;
export const DEFAULT_CHAR_SIZE = CharSize.B16;

export const DEFAULT_STORE_NAME = 'keystore';