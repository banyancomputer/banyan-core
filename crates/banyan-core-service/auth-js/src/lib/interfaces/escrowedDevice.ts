/**
 * @interface EscrowedDevice - A user's escrowed device key material for Signing requests to Banyan Services and Recovering Shares.
 * Primary key material for the user's account, used to authorize additional devices.
 * Should be generated locally on account creation. Escrowed to our Auth Service.
 * @property ecdsa_fingerprint - the fingerprint of the device's public ecdsa key -- compressed point EC public key fingerprint.
 * @property ecdh_fingerprint - the fingerprint of the device's public ecdh key -- compressed point EC public key fingerprint.
 * @property wrapped_ecdsa_pkcs8 - the wrapped private ecdsa key - generated using webCrytpo.subtle.wrapKey with pkcs8 format and exporint as base64
 * @property wrapped_ecdh_pkcs8 - the wrapped private ecdh key - generated using webCrytpo.subtle.wrapKey with pkcs8 format and exporint as base64
 * @property passkey_salt - the salt used to derive the user's pass key, when used in conjunction with the user's pass key.
 */
export interface EscrowedDevice {
	ecdsa_fingerprint: string;
	ecdh_fingerprint: string;
	wrapped_ecdsa_pkcs8: string;
	wrapped_ecdh_pkcs8: string;
	passkey_salt: string;
}

export const verifyEscrowedDevice = (
	escrowedDevice: EscrowedDevice
): boolean => {
	// TODO: More robust validation
	if (!escrowedDevice) {
		return false;
	}
	// Check that all fields are present
	if (!escrowedDevice.ecdsa_fingerprint) {
		return false;
	}
	if (!escrowedDevice.ecdh_fingerprint) {
		return false;
	}
	if (!escrowedDevice.wrapped_ecdsa_pkcs8) {
		return false;
	}
	if (!escrowedDevice.wrapped_ecdh_pkcs8) {
		return false;
	}
	if (!escrowedDevice.passkey_salt) {
		return false;
	}
	return true;
};
