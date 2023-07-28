import { EscrowedDevice, DeviceApiKey } from '../interfaces';

export interface GetEscrowedDevice {
	escrowed_device: EscrowedDevice;
	api_key_pem: string;
	encryption_key_pem: string;
}

export interface GetDeviceApiKeys {
	device_api_keys: DeviceApiKeyAttributes[];
}
