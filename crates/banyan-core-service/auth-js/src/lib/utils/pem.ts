export const publicPemWrap = (spki: string) => {
	// Wrap the public key in a pem
	const pemHeader = '-----BEGIN PUBLIC KEY-----\n';
	const pemFooter = '\n-----END PUBLIC KEY-----';
	const pem = pemHeader + spki + pemFooter;
	return pem;
};

export const publicPemUnwrap = (pem: string) => {
	// Unwrap the public key from a pem
	const pemHeader = '-----BEGIN PUBLIC KEY-----\n';
	const pemFooter = '\n-----END PUBLIC KEY-----';
	const spki = pem.replace(pemHeader, '').replace(pemFooter, '');
	return spki;
};
