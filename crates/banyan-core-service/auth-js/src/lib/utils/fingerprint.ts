import { webcrypto } from 'one-webcrypto';
import { publicPemUnwrap } from './pem';
import { base64ToArrBuf, fingerprintEcPublicKey, prettyFingerprint } from 'banyan-webcrypto-experiment/lib/utils';

export const prettyFingerprintApiKeyPem = async (pem: string): Promise<string> => {
	const publicSpki = publicPemUnwrap(pem);
	return await prettyFingerprintApiKeySpki(publicSpki);
};

export const prettyFingerprintApiKeySpki = async (spki: string): Promise<string> => {
	// convert the spki to an ArrayBuffer
	const binaryString = atob(spki);
  const buffer = new Uint8Array(binaryString.length);
  for (let i = 0; i < binaryString.length; i++) {
    buffer[i] = binaryString.charCodeAt(i);
  }

  console.log('buffer: ', buffer);
  
  
	
	return await webcrypto.subtle
		.importKey(
			'spki',
			buffer.buffer,
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