import { webcrypto } from 'one-webcrypto';
import {
    base64ToArrBuf,
    fingerprintEcPublicKey,
    hexFingerprint,
    prettyFingerprint,
} from '@/lib/crypto/utils';
import { publicPemUnwrap } from './pem';

export const prettyFingerprintApiKeySpki = async(
    spki: string
): Promise<string> => await webcrypto.subtle
    .importKey(
        'spki',
        base64ToArrBuf(spki),
        {
            name: 'ECDSA',
            namedCurve: 'P-384',
        },
        true,
        ['verify']
    )
    .then((key) => fingerprintEcPublicKey(key))
    .then((fingerprintBytes) => prettyFingerprint(fingerprintBytes));

export const prettyFingerprintApiKeyPem = async(
    pem: string
): Promise<string> => {
    const publicSpki = publicPemUnwrap(pem);
    return await prettyFingerprintApiKeySpki(publicSpki);
};

export const hexFingerprintApiKeySpki = async(
    spki: string
): Promise<string> => await webcrypto.subtle
    .importKey(
        'spki',
        base64ToArrBuf(spki),
        {
            name: 'ECDSA',
            namedCurve: 'P-384',
        },
        true,
        ['verify']
    )
    .then((key) => fingerprintEcPublicKey(key))
    .then((fingerprintBytes) => hexFingerprint(fingerprintBytes));

export const hexFingerprintApiKeyPem = async(
    pem: string
): Promise<string> => {
    const publicSpki = publicPemUnwrap(pem);
    return await hexFingerprintApiKeySpki(publicSpki);
};
