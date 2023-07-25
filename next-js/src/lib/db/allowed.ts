import { getApplicationDataSource } from './index';
import { Allowed } from './entities/allowed';

export const AllowedFactory = {
	build: async (attrs: Partial<Allowed> = {}) => {
		return getApplicationDataSource().then((manager) => {
			return manager.getRepository(Allowed).create(attrs);
		});
	},

	create: async (attrs: Partial<Allowed> = {}) => {
		const allowed = await AllowedFactory.build(attrs);
		return await getApplicationDataSource().then((manager) => {
			return manager.getRepository(Allowed).save(allowed);
		});
	},

	readAll: async () => {
		return await getApplicationDataSource().then((manager) => {
			return manager.getRepository(Allowed).find();
		});
	},

	readByEmail: async (email: string) => {
		return await getApplicationDataSource().then((manager) => {
			return manager.getRepository(Allowed).findOneBy({
				email: email,
			});
		});
	},

	deleteByEmail: async (email: string) => {
		return await getApplicationDataSource().then((manager) => {
			return manager.getRepository(Allowed).delete({
				email: email,
			});
		});
	},
};
