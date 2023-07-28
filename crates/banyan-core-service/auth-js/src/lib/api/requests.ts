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
