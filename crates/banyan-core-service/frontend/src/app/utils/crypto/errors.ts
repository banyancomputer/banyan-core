import { KeyUse } from './types';

export const ECCNotEnabled = new Error('ECC is not enabled for this browser.');
export const UnsupportedSymmCrypto = new Error('Cryptosystem not supported. Please use AES-GCM');
export const InvalidKeyUse = new Error('Invalid key use. Please use \'exchange\' or \'write');
export const InvalidMaxValue = new Error('Max must be less than 256 and greater than 0');
export const InvalidIvLength = new Error('IV must be 16 bytes');
export const InvalidCipherTextLength = new Error('Cipher text must align on AES-GCM block (16 bytes) boundary');
export const InvalidCipherText = new Error('Invalid cipher text');

export function checkValidKeyUse(use: KeyUse): void {
    checkValid(use, [KeyUse.Exchange, KeyUse.Write], InvalidKeyUse);
}

function checkValid<T>(toCheck: T, opts: T[], error: Error): void {
    const match = opts.some(opt => opt === toCheck);
    if (!match) {
        throw error;
    }
}

export default {
    ECCNotEnabled,
    InvalidKeyUse,
    checkValidKeyUse,
    InvalidMaxValue,
    InvalidIvLength,
    InvalidCipherTextLength,
    InvalidCipherText,
};
