import { EscrowedDevice as EscrowedDeviceAttributes } from '../interfaces';
import { EscrowedDevice } from './models';

export const EscrowedDeviceFactory = {
	create: async (attrs: Partial<EscrowedDeviceAttributes> = {}) =>
		EscrowedDevice.create(attrs),

	readByAccountId: async (accountId: string) =>
		EscrowedDevice.findOne({
			where: { accountId },
		}),
};
