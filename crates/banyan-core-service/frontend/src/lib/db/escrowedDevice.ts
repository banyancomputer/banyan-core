import { EscrowedDevice } from './models';
import { EscrowedDevice as EscrowedDeviceAttributes } from '../interfaces';

export const EscrowedDeviceFactory = {
	create: async (attrs: Partial<EscrowedDeviceAttributes> = {}) => {
		return EscrowedDevice.create(attrs);
	},

	readByAccountId: async (accountId: string) => {
		return EscrowedDevice.findOne({
			where: { accountId },
		});
	},
};
