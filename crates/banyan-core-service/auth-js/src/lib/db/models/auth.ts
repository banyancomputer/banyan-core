import { models } from '@auth/sequelize-adapter';
import { Sequelize, DataTypes, ModelDefined } from 'sequelize';

export interface AccountAttributes {
	id: string;
	escrowed_device_blob: string | null;
	encryption_key_pem: string | null;
	api_key_pem: string | null;
	userId: string;
	type: string;
	provider: string;
	providerAccountId: string;
	refresh_token: string | null;
	access_token: string | null;
	expires_at: number | null;
	token_type: string | null;
	scope: string | null;
	id_token: string | null;
	session_state: string | null;
}

const AccountModel = (
	sequelize: Sequelize
): ModelDefined<AccountAttributes, {}> => {
	const Account = sequelize.define('Account', {
		...models.Account,
		escrowed_device_blob: {
			type: DataTypes.STRING,
			allowNull: true,
		},
		encryption_key_pem: {
			type: DataTypes.STRING,
			allowNull: true,
		},
		api_key_pem: {
			type: DataTypes.STRING,
			allowNull: true,
		},
	},
	{
		createdAt: false,
		updatedAt: false,
	}
	);

	return Account;
};

export default AccountModel;
