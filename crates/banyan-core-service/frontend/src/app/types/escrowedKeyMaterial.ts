/**
 * @interface EscrowedDevice - A user's escrowed device key material Banyan Services.
 * Primary key material for the user's account.
 * Should be generated locally on account creation. Escrowed to our Auth Service.
 * @property apiPublicKeyPem - the public key of the device's API key in PEM format
 * @property encryptionPublicKeyPem - the public key of the device's encryption key in PEM format
 * @property encryptedPrivateKeyMaterial - the encrypted private key material of the device's encryption key
 * @property passKeySalt - the salt used to derive the user's pass key, when used in conjunction with the user's pass key.
 */
export interface EscrowedKeyMaterial {
	apiPublicKeyPem: string;
	encryptionPublicKeyPem: string;
	encryptedPrivateKeyMaterial: string;
	passKeySalt: string;
}
