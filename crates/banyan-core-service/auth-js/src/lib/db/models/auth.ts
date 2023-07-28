import { FINGERPRINT_REGEX, PEM_REGEX } from '@/lib/utils';
import { models } from '@auth/sequelize-adapter';
import { Sequelize, DataTypes, ModelDefined, Model } from 'sequelize';
import { validateOrReject } from 'class-validator';

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

interface AccountInstance 
	extends Model<AccountAttributes>,
		AccountAttributes {
	validate(): Promise<void>;
}

const AccountModel = (
	sequelize: Sequelize
): ModelDefined<AccountAttributes, {}> => {
	const Account = sequelize.define<AccountInstance>('Account', {
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

			hooks: {
				beforeCreate: async (account: AccountInstance) => {
					await account.validate();
				},
				beforeUpdate: async (account: AccountInstance) => {
					await account.validate();
				},
			},
			tableName: 'accounts',
		}
	);

	Account.prototype.validate = async function () {
		if (!this.api_key_pem && !this.encryption_key_pem) {
			return true;
		}
		if (
			typeof this.api_key_pem !== 'string' ||
			!this.api_key_pem.match(PEM_REGEX)
		) {
			throw new Error('invalid pem');
		}
		if (
			typeof this.encryption_key_pem !== 'string' ||
			!this.encryption_key_pem.match(PEM_REGEX)
		) {
			throw new Error('invalid fingerprint');
		}
		await validateOrReject(this);
	};
	return Account;
};

export default AccountModel;