import { EscrowedKeyMaterial } from '@app/types/escrowedKeyMaterial';
import { APIClient } from './http';

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
            pass_key_salt: request.passKeySalt,
        }));

        if (!response.ok) {
            await this.handleError(response);
        }
    };
}
