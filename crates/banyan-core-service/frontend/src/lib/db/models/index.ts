import { Sequelize } from 'sequelize';
import AllowedEmailModel from './allowedEmail';
import AccountModel from './account';
import DeviceApiKeyModel from './deviceApiKey';
import EscrowedDeviceModel from './escrowedDevice';

export const client = new Sequelize({
	dialect: 'sqlite',
	storage: process.env.DB_PATH,
	logging: process.env.NODE_ENV === 'development',
	sync: { force: false, alter: false },
});

const AllowedEmail = AllowedEmailModel(client);
const Account = AccountModel(client);
const EscrowedDevice = EscrowedDeviceModel(client);
const DeviceApiKey = DeviceApiKeyModel(client);
Account.hasOne(EscrowedDevice, {
	foreignKey: 'account_id', // Snake case
	sourceKey: 'id', // Snake case
	onDelete: 'CASCADE',
});
Account.hasMany(DeviceApiKey, {
	foreignKey: 'account_id', // Snake case
	sourceKey: 'id', // Snake case
	onDelete: 'CASCADE',
});

export const models = {
	AllowedEmail,
	EscrowedDevice,
	Account,
	DeviceApiKey,
};

export default client;
export * from './errors';
export { AllowedEmail, Account, DeviceApiKey, EscrowedDevice };
