import { webcrypto } from 'one-webcrypto';
import {
    base64ToArrBuf,
    fingerprintEcPublicKey,
    prettyFingerprint,
} from '@app/utils/crypto/utils';
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
