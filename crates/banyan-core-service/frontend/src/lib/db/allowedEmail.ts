import { AllowedEmail } from './models';
import { AllowedEmail as AllowedEmailAttributes } from '@/lib/interfaces';

export const AllowedEmailFactory = {
    create: async(attrs: Partial<AllowedEmailAttributes> = {}) => AllowedEmail.create(attrs),

    readAll: async() => AllowedEmail.findAll(),

    readByEmail: async(email: string) => AllowedEmail.findOne({
        where: { email: email },
    }),

    deleteByEmail: async(email: string) => AllowedEmail.destroy({
        where: { email: email },
    }),
};
