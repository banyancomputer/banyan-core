import {
	EscrowedDeviceKeyPair,
	DevicePublicKey,
} from '@/lib/db/entities/index';

/**
 * Respond with a user's escrowed key pair
 * @param escrowed_key - the attributes of the escrowed key to be created
 */
export interface EscrowDeviceKeyPair {
	escrowed_device_key_pair: EscrowedDeviceKeyPair;
}

/**
 * Attempt to register a Device Public Key with a User
 * @param device_public_key - the attributes of the public key to be created
 */
export interface RegisterDevicePublicKey {
	device_public_key: DevicePublicKey;
}
