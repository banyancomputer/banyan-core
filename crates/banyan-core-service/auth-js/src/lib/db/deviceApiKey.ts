import { DeviceApiKey as DeviceApiKeyAttributes } from '@/lib/interfaces';
import { DeviceApiKey } from './models';

export const DeviceApiKeyFactory = {
	build: async (attrs: Partial<DeviceApiKeyAttributes> = {}) => {
		return DeviceApiKey.build(attrs);
	},

	create: async (attrs: Partial<DeviceApiKeyAttributes> = {}) => {
		return DeviceApiKey.create(attrs);
	},

	readAllByAccountId: async (account_id: string) => {
		return DeviceApiKey.findAll({
			where: { account_id },
		});
	},

	readByFingerprint: async (fingerprint: string) => {
		return DeviceApiKey.findOne({
			where: { fingerprint },
		});
	},

	deleteByAccountIdAndFingerprint: async (
		account_id: string,
		fingerprint: string
	) => {
		return DeviceApiKey.destroy({
			where: {
				account_id,
				fingerprint,
			},
		});
	},
};
