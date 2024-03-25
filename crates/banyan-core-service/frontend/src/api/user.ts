import { EscrowedKeyMaterial } from '@/app/types';
import { APIClient } from './http';
import { User } from '@/entities/user';

export class UserClient extends APIClient {
    public async getCurrentUser(): Promise<User> {
        const response = await this.http.get(`${this.ROOT_PATH}/api/v1/users/current`);

        if (!response.ok) {
            await this.handleError(response);
        }

        const rawUser = await response.json();

        return {
            id: rawUser.id,
            email: rawUser.email,
            displayName: rawUser.display_name,
            locale: rawUser.locale,
            monthlyEggress: rawUser.monthly_egress,
            profileImage: rawUser.profile_image,
            acceptedTosAt: rawUser.accepted_tos_at,
			accountTaxClass: rawUser.account_tax_class,
            subscriptionId: rawUser.subscription_id,
            subscriptionValidUntil: rawUser.subscription_valid_until,
        };
    };

    public async getEscrowedKeyMaterial(): Promise<EscrowedKeyMaterial> {
        const response = await this.http.get(`${this.ROOT_PATH}/api/v1/users/escrowed_device`);

        if (!response.ok) {
            await this.handleError(response);
        }

        const escrowedDevice =  await response.json();

        return {
			apiPublicKeyPem: escrowedDevice.api_public_key_pem,
			encryptionPublicKeyPem: escrowedDevice.encryption_public_key_pem,
			encryptedPrivateKeyMaterial: escrowedDevice.encrypted_private_key_material,
			passKeySalt: escrowedDevice.pass_key_salt
		};
    };
};
