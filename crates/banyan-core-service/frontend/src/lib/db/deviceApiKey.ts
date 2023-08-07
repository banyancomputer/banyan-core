import { DeviceApiKey as DeviceApiKeyAttributes } from '@/lib/interfaces';
import { DeviceApiKey } from './models';

export const DeviceApiKeyFactory = {
	create: async (attrs: Partial<DeviceApiKeyAttributes> = {}) => {
		return DeviceApiKey.create(attrs);
	},

	readAllByAccountId: async (accountId: string) => {
		return DeviceApiKey.findAll({
			where: { accountId },
		});
	},

	readByFingerprint: async (fingerprint: string) => {
		return DeviceApiKey.findOne({
			where: { fingerprint },
		});
	},

	deleteByAccountIdAndFingerprint: async (
		accountId: string,
		fingerprint: string
	) => {
		return DeviceApiKey.destroy({
			where: {
				accountId,
				fingerprint,
			},
		});
	},
};
