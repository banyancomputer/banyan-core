import { getApplicationDataSource } from './index';
import { DevicePublicKeyEntity as DevicePublicKey } from './entities';

export const DevicePublicKeyFactory = {
	build: async (attrs: Partial<DevicePublicKey> = {}) => {
		return getApplicationDataSource().then((manager) => {
			return manager.getRepository(DevicePublicKey).create(attrs);
		});
	},

	create: async (attrs: Partial<DevicePublicKey> = {}) => {
		const publicKey = await DevicePublicKeyFactory.build(attrs);
		return await getApplicationDataSource().then((manager) => {
			return manager.getRepository(DevicePublicKey).save(publicKey);
		});
	},

	readAllByUserId: async (user_id: string) => {
		return getApplicationDataSource().then((manager) => {
			return manager.getRepository(DevicePublicKey).findBy({ user_id });
		});
	},

	readByEcdsaFingerprint: async (ecdsa_fingerprint: string) => {
		return getApplicationDataSource().then((manager) => {
			return manager
				.getRepository(DevicePublicKey)
				.findOneBy({ ecdsa_fingerprint });
		});
	},

	deleteByUserIdAndEcdsaFingerprint: async (
		user_id: string,
		ecdsa_fingerprint: string
	) => {
		return getApplicationDataSource().then((manager) => {
			return manager
				.createQueryBuilder()
				.delete()
				.from(DevicePublicKey)
				.where(
					'user_id = :user_id AND ecdsa_fingerprint = :ecdsa_fingerprint',
					{ user_id, ecdsa_fingerprint }
				)
				.execute();
		});
	},
};
