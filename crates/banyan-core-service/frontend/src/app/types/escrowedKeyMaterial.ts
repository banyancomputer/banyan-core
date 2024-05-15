/**
 * @interface EscrowedDevice - A user's escrowed device key material Banyan Services.
 * Primary key material for the user's account.
 * Should be generated locally on account creation. Escrowed to our Auth Service.
 * @property publicKey - the public API user key of the device in PEM format
 * @property passKeySalt - the salt used to derive the user's pass key, when used in conjunction with the user's pass key.
 */
export interface EscrowedKeyMaterial {
	publicKey: string;
	encryptedPrivateKeyMaterial: string;
	passKeySalt: string;
}
