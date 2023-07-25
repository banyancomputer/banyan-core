import { getApplicationDataSource } from './index';
import { EscrowedKey } from './entities';

export const EscrowedKeyFactory = {
	build: async (attrs: Partial<EscrowedKey> = {}) => {
		return getApplicationDataSource().then((manager) => {
			return manager.getRepository(EscrowedKey).create(attrs);
		});
	},

	create: async (attrs: Partial<EscrowedKey> = {}) => {
		const escrowedKey = await EscrowedKeyFactory.build(attrs);
		return await getApplicationDataSource().then((manager) => {
			return manager.getRepository(EscrowedKey).save(escrowedKey);
		});
	},

	readByOwner: async (owner: string) => {
		return await getApplicationDataSource().then((manager) => {
			return manager.getRepository(EscrowedKey).findOneBy({
				owner: owner,
			});
		});
	},

	deleteByOwner: async (owner: string) => {
		return await getApplicationDataSource().then((manager) => {
			return manager.getRepository(EscrowedKey).delete({
				owner: owner,
			});
		});
	},
};
