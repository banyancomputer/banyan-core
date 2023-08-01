import { Account } from './models';
import { splitProviderId } from '../utils';

export const AccountFactory = {
	/**
	 * Get a user's account id from their provider id
	 * @param providerId -- string in the format of provider:providerAccountId
	 * @returns account id
	 */
	idFromProviderId: async (providerId: string) => {
		const [provider, providerAccountId] = splitProviderId(providerId);
		// Find the account in the database
		return Account.findOne({
			where: {
				provider,
				providerAccountId,
			},
		}).then((account) => {
			if (!account) {
				throw new Error('Account not found');
			}
			// @ts-ignore
			return account.id;
		});
	},

	readEscrowedDevice: async (account_id: string) => {
		return Account.findOne({
			where: {
				id: account_id,
			},
		}).then((account) => {
			if (!account) {
				throw new Error('Account not found');
			}

			const { escrowed_device_blob, encryption_key_pem, api_key_pem } = account as Partial<AccountAttributes>;
			if (!escrowed_device_blob || !encryption_key_pem || !api_key_pem) {
				return null;
			}
			return {
				escrowed_device: JSON.parse(escrowed_device_blob) as EscrowedDevice,
				encryption_key_pem,
				api_key_pem,
			};
		});
	},


	updateEscrowedDevice: async (
		account_id: string,
		escrowed_device: EscrowedDevice,
		api_key_pem: string,
		encryption_key_pem: string
	) => {
		const escrowed_device_blob = JSON.stringify(escrowed_device);
		await Account.update(
			{
				escrowed_device_blob,
				encryption_key_pem,
				api_key_pem,
			},
			{
				where: {
					id: account_id,
				},
			}
		);
	},
};
