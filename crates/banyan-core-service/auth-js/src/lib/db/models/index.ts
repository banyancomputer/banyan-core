import AllowedEmailModel from './allowedEmail';
import AccountModel from './auth';
import DeviceApiKeyModel from './deviceApiKey';

import { Sequelize } from 'sequelize';

export const client = new Sequelize({
	dialect: 'sqlite',
	storage: process.env.DB_PATH,
	sync: { force: false, alter: false },
});

const AllowedEmail = AllowedEmailModel(client);
const Account = AccountModel(client);
const DeviceApiKey = DeviceApiKeyModel(client);
Account.hasMany(DeviceApiKey, {
	foreignKey: 'account_id',
	sourceKey: 'id',
	onDelete: 'CASCADE',
});


export const models = {
	AllowedEmail,
	Account,
	DeviceApiKey,
};

export default client;
export { AllowedEmail, Account, DeviceApiKey };
