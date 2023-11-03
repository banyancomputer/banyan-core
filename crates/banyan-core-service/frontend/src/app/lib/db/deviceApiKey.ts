import { DeviceApiKey } from './models';
import { DeviceApiKey as DeviceApiKeyAttributes } from '@app/lib/interfaces';

export const DeviceApiKeyFactory = {
    create: async(attrs: Partial<DeviceApiKeyAttributes> = {}) => DeviceApiKey.create(attrs),

    readAllByAccountId: async(userId: string) => DeviceApiKey.findAll({
        where: { userId },
    }),

    readByFingerprint: async(fingerprint: string) => DeviceApiKey.findOne({
        where: { fingerprint },
    }),

    deleteByAccountIdAndFingerprint: async(
        userId: string,
        fingerprint: string
    ) => DeviceApiKey.destroy({
        where: {
            userId,
            fingerprint,
        },
    }),
};
