import { AllowedEmail } from './models';
import { AllowedEmailAttributes } from './models/allowedEmail';

export const AllowedEmailFactory = {
	create: async (attrs: Partial<AllowedEmailAttributes> = {}) => {
		return AllowedEmail.create(attrs);
	},

	readAll: async () => {
		return AllowedEmail.findAll();
	},

	readByEmail: async (email: string) => {
		return AllowedEmail.findOne({
			where: { email: email },
		});
	},

	deleteByEmail: async (email: string) => {
		return AllowedEmail.destroy({
			where: { email: email },
		});
	},
};

// import { getApplicationDataSource } from './index';
// import { AllowedEmailEntity as AllowedEmail } from './models/allowedEmail';

// export const AllowedEmailFactory = {
// 	build: async (attrs: Partial<AllowedEmail> = {}) => {
// 		return getApplicationDataSource().then((manager) => {
// 			return manager.getRepository(AllowedEmail).create(attrs);
// 		});
// 	},

// 	create: async (attrs: Partial<AllowedEmail> = {}) => {
// 		const allowed = await AllowedEmailFactory.build(attrs);
// 		return await getApplicationDataSource().then((manager) => {
// 			return manager.getRepository(AllowedEmail).save(allowed);
// 		});
// 	},

// 	readAll: async () => {
// 		return await getApplicationDataSource().then((manager) => {
// 			return manager.getRepository(AllowedEmail).find();
// 		});
// 	},

// 	readByEmail: async (email: string) => {
// 		return await getApplicationDataSource().then((manager) => {
// 			return manager.getRepository(AllowedEmail).findOneBy({
// 				email: email,
// 			});
// 		});
// 	},

// 	deleteByEmail: async (email: string) => {
// 		return await getApplicationDataSource().then((manager) => {
// 			return manager.getRepository(AllowedEmail).delete({
// 				email: email,
// 			});
// 		});
// 	},
// };
