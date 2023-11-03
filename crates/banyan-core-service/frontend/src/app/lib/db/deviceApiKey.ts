import { DeviceApiKey } from './models';
import { DeviceApiKey as DeviceApiKeyAttributes } from '@app/lib/interfaces';

export const DeviceApiKeyFactory = {
    create: async(attrs: Partial<DeviceApiKeyAttributes> = {}) => DeviceApiKey.create(attrs),

    readAllByAccountId: async(accountId: string) => DeviceApiKey.findAll({
        where: { accountId },
    }),

    readByFingerprint: async(fingerprint: string) => DeviceApiKey.findOne({
        where: { fingerprint },
    }),

    deleteByAccountIdAndFingerprint: async(
        accountId: string,
        fingerprint: string
    ) => DeviceApiKey.destroy({
        where: {
            accountId,
            fingerprint,
        },
    }),
};
