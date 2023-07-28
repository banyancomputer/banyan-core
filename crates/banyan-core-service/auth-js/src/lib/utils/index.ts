export * from './id';
export * from './pem';

export const FINGERPRINT_REGEX = new RegExp('^[0-9a-fA-F]{2}(?::[0-9a-fA-F]{2})*$');
export const PEM_REGEX = new RegExp('-----BEGIN PUBLIC KEY-----(.*?)-----END PUBLIC KEY-----');