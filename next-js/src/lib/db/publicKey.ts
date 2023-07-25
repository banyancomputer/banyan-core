import { getApplicationDataSource } from './index';
import { PublicKey } from './entities';

export const PublicKeyFactory = {
	build: async (attrs: Partial<PublicKey> = {}) => {
		return getApplicationDataSource().then((manager) => {
			return manager.getRepository(PublicKey).create(attrs);
		});
	},

	create: async (attrs: Partial<PublicKey> = {}) => {
		const publicKey = await PublicKeyFactory.build(attrs);
		return await getApplicationDataSource().then((manager) => {
			return manager.getRepository(PublicKey).save(publicKey);
		});
	},

	readAllByOwner: async (owner: string) => {
		return getApplicationDataSource().then((manager) => {
			return manager.getRepository(PublicKey).findBy({ owner });
		});
	},

	readByFingerprint: async (ecdsa_fingerprint: string) => {
		return getApplicationDataSource().then((manager) => {
			return manager.getRepository(PublicKey).findOneBy({ ecdsa_fingerprint });
		});
	},
};
