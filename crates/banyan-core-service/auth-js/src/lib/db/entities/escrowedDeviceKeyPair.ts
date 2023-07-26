import { DevicePublicKey } from './devicePublicKey';
import { EscrowedDevicePrivateKey } from './escrowedDevicePrivateKey';

export interface EscrowedDeviceKeyPair {
	device_public_key: DevicePublicKey;
	escrowed_device_private_key: EscrowedDevicePrivateKey;
}
