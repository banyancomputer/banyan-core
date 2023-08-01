import { webcrypto } from 'one-webcrypto';
import { publicPemUnwrap } from './pem';
import { base64ToArrBuf, fingerprintEcPublicKey, prettyFingerprint } from 'banyan-webcrypto-experiment/lib/utils';

export const prettyFingerprintApiKeyPem = async (pem: string): Promise<string> => {
	const publicSpki = publicPemUnwrap(pem);
	return await prettyFingerprintApiKeySpki(publicSpki);
};

export const prettyFingerprintApiKeySpki = async (spki: string): Promise<string> => {
	return await webcrypto.subtle
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
};