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

// import { getApplicationDataSource } from './index';
// import { DeviceApiKeyEntity as DeviceApiKey } from './models';

// export const DeviceApiKeyFactory = {
// 	build: async (attrs: Partial<DeviceApiKey> = {}) => {
// 		return getApplicationDataSource().then((manager) => {
// 			return manager.getRepository(DeviceApiKey).create(attrs);
// 		});
// 	},

// 	create: async (attrs: Partial<DeviceApiKey> = {}) => {
// 		const publicKey = await DeviceApiKeyFactory.build(attrs);
// 		return await getApplicationDataSource().then((manager) => {
// 			return manager.getRepository(DeviceApiKey).save(publicKey);
// 		});
// 	},

// 	readAllByAccountId: async (account_id: string) => {
// 		return getApplicationDataSource().then((manager) => {
// 			return manager.getRepository(DeviceApiKey).findBy({ account_id });
// 		});
// 	},

// 	readByEcdsaFingerprint: async (ecdsa_fingerprint: string) => {
// 		return getApplicationDataSource().then((manager) => {
// 			return manager
// 				.getRepository(DeviceApiKey)
// 				.findOneBy({ ecdsa_fingerprint });
// 		});
// 	},

// 	deleteByUserIdAndEcdsaFingerprint: async (
// 		account_id: string,
// 		ecdsa_fingerprint: string
// 	) => {
// 		return getApplicationDataSource().then((manager) => {
// 			return manager
// 				.createQueryBuilder()
// 				.delete()
// 				.from(DeviceApiKey)
// 				.where(
// 					'account_id = :user_id AND ecdsa_fingerprint = :ecdsa_fingerprint',
// 					{ account_id, ecdsa_fingerprint }
// 				)
// 				.execute();
// 		});
// 	},
// };
