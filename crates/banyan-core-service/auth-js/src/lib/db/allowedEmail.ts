import { AllowedEmail } from './models';
import { AllowedEmail as AllowedEmailAttributes } from '@/lib/interfaces';

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
