import { EscrowedDevice, DeviceApiKey } from "../interfaces";

export interface GetEscrowedDevice {
    escrowed_device: EscrowedDevice;
    api_key_pem: string;
	encryption_key_pem: string;
}

export { type DeviceApiKey as RegisterDeviceApiKey }

export interface GetDeviceApiKeys {
    device_api_keys: DeviceApiKey[];
}
