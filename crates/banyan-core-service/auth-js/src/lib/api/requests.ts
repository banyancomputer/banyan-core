import { EscrowedDevice } from '@/lib/interfaces';

/**
 * Initialize a User's Account with an Escrowed Device Key Pair and Associated public keys
 * @param escrowed_device - the escrowed device key pair to be associated with the user's account
 * @param api_key_pem - the ecdsa public key to be associated with the user's account
 * @param encryption_pem - the ecdh public key to be associated with the user's account
 */
export interface EscrowDevice {
	escrowed_device: EscrowedDevice;
	api_key_pem: string;
	encryption_key_pem: string;
}

// /**
//  * Attempt to register a Device Public Key with a User
//  * @param device_public_key - the attributes of the public key to be created
//  */
// export interface RegisterDeviceApiKey {
//     device_public_key: Partial<DevicePublicKey>;
// }

// /**
//  * Authorize a Device Public Key with a User
//  */
// export interface AuthorizeDevicePublicKey {
//     // TODO: is id better for this?
//     device_public_key_ecdsa_fingerprint: string;
// }

// /**
//  * Deny or remove a Device Public Key from a User
//  */
// export interface DeleteDevicePublicKey {
//     // TODO: is id better for this?
//     device_public_key_ecdsa_fingerprint: string;
// }
