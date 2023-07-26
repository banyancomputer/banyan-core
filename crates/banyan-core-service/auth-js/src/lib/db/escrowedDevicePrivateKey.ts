import { getApplicationDataSource } from './index';
import { EscrowedDevicePrivateKeyEntity as EscrowedDevicePrivateKey } from './entities';

export const EscrowedDevicePrivateKeyFactory = {
	build: async (attrs: Partial<EscrowedDevicePrivateKey> = {}) => {
		return getApplicationDataSource().then((manager) => {
			return manager.getRepository(EscrowedDevicePrivateKey).create(attrs);
		});
	},

	create: async (attrs: Partial<EscrowedDevicePrivateKey> = {}) => {
		const escrowedKey = await EscrowedDevicePrivateKeyFactory.build(attrs);
		return await getApplicationDataSource().then((manager) => {
			return manager.getRepository(EscrowedDevicePrivateKey).save(escrowedKey);
		});
	},

	readByUserId: async (user_id: string) => {
		return await getApplicationDataSource().then((manager) => {
			return manager.getRepository(EscrowedDevicePrivateKey).findOneBy({
				user_id,
			});
		});
	},

	deleteByUserId: async (user_id: string) => {
		return await getApplicationDataSource().then((manager) => {
			return manager.getRepository(EscrowedDevicePrivateKey).delete({
				user_id,
			});
		});
	},
};
