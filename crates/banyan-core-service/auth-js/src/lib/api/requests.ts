import { EscrowedDevicePrivateKey, DevicePublicKey } from '@/lib/db/entities';

/**
 * Escrow a user's key pair
 * @param public_key - the attributes of the public key to be created
 * @param escrowed_key - the attributes of the escrowed key to be created
 */
export interface EscrowDeviceKeyPair {
    device_public_key: Partial<DevicePublicKey>;
    escrowed_device_private_key: Partial<EscrowedDevicePrivateKey>;
}

/**
 * Attempt to register a Device Public Key with a User
 * @param device_public_key - the attributes of the public key to be created
 */
export interface RegisterDevicePublicKey {
    device_public_key: Partial<DevicePublicKey>;
}

// /**
//  * Authorize a Device Public Key with a User
//  */
// export interface AuthorizeDevicePublicKey {
//     // TODO: is id better for this?
//     device_public_key_ecdsa_fingerprint: string;
// }

/**
 * Deny or remove a Device Public Key from a User
 */
export interface DeleteDevicePublicKey {
    // TODO: is id better for this?
    device_public_key_ecdsa_fingerprint: string;
}
