import { DeviceApiKey, EscrowedKeyMaterial } from "@app/types";
import { APIClient } from "./http";
import { publicPemUnwrap } from "@app/utils";
import { b64UrlEncode } from "@app/utils/b64";

export class AuthClient extends APIClient {
    /**
    * Initialize (or update) an Account with an Escrowed Device Key Pair and Associated public keys
    * @param escrowed_device - the escrowed device key material to be associated with the user's account
    */
	public async escrowDevice(request: EscrowedKeyMaterial): Promise<void> {
        const response = await this.http.post(`${this.ROOT_PATH}/api/v1/auth/create_escrowed_device`, JSON.stringify({
			api_public_key_pem: request.apiPublicKeyPem,
			encryption_public_key_pem: request.encryptionPublicKeyPem,
			encrypted_private_key_material: request.encryptedPrivateKeyMaterial,
			pass_key_salt: request.passKeySalt
		}));

        if (!response.ok) {
            await this.handleError(response);
        }
	};

    /**
    * Register a device api key for a user
    * @param spki - the public key of the device's API key in PEM format
    * @return void
    */
	public async registerDeviceApiKey(pem: string): Promise<DeviceApiKey> {
		const spki = publicPemUnwrap(pem);
		const urlSpki = b64UrlEncode(spki);
        const response = await this.http.get(`${this.ROOT_PATH}/auth/device/register?spki=${urlSpki}`);

        if (!response.ok) {
            await this.handleError(response);
        }

        return await response.json();
	};

    /**
    * Get the device api keys for a user
    * @return the device api keys for a user
    */
	public async readDeviceApiKeys(): Promise<DeviceApiKey[]> {
        const response = await this.http.get(`${this.ROOT_PATH}/auth/device`);

        if (!response.ok) {
            await this.handleError(response);
        }

        return await response.json();
	};
}