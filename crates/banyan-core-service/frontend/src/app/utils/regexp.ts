const PRETTY_FINGERPRINT_REGEX = /^[0-9a-fA-F]{2}(?::[0-9a-fA-F]{2})*$/;
const HEX_FINGERPRINT_REGEX = /^(?:[0-9a-fA-F]{2})*$/;
const PUBLIC_PEM_REGEX =
	/^-----BEGIN ([A-Z ]+)-----\r?\n([A-Za-z0-9+/=\r\n]+)\r?\n-----END \1-----$/;

export const isPem = (pem: string): boolean => PUBLIC_PEM_REGEX.test(pem);

export const isPrettyFingerprint = (fingerprint: string): boolean => PRETTY_FINGERPRINT_REGEX.test(fingerprint);
export const isHexFingerprint = (fingerprint: string): boolean => HEX_FINGERPRINT_REGEX.test(fingerprint);