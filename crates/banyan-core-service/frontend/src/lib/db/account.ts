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
};
