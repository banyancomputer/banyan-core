import { Sequelize, DataTypes, Model, ModelDefined } from 'sequelize';
import { validateOrReject } from 'class-validator';
import { DeviceApiKey as DeviceApiKeyAttributes } from '@/lib/interfaces';
import { FINGERPRINT_REGEX, PEM_REGEX } from '@/lib/utils';

interface DeviceApiKeyInstance
	extends Model<DeviceApiKeyAttributes>,
		DeviceApiKeyAttributes {
	validate(): Promise<void>;
}

const DeviceApiKeyModel = (
	sequelize: Sequelize
): ModelDefined<DeviceApiKeyAttributes, {}> => {
	const DeviceApiKey = sequelize.define<DeviceApiKeyInstance>(
		'deviceApiKey',
		{
			id: {
				type: DataTypes.UUID,
				defaultValue: DataTypes.UUIDV4,
				primaryKey: true,
			},
			account_id: {
				type: DataTypes.STRING,
				allowNull: false,
			},
			fingerprint: {
				type: DataTypes.STRING,
				allowNull: false,
				unique: true,
			},
			pem: {
				type: DataTypes.STRING,
				allowNull: false,
			},
		},
		{
			createdAt: false,
			updatedAt: false,
			hooks: {
				beforeCreate: async (deviceApiKey: DeviceApiKeyInstance) => {
					await deviceApiKey.validate();
				},
				beforeUpdate: async (deviceApiKey: DeviceApiKeyInstance) => {
					await deviceApiKey.validate();
				},
			},
			tableName: 'device_api_keys',
			indexes: [
				{
					fields: ['account_id'],
				},
				{
					fields: ['ecdsa_fingerprint'],
					unique: true,
				},
			],
		}
	);

	DeviceApiKey.prototype.validate = async function () {
		if (
			typeof this.pem !== 'string' ||
			!this.pem.match(PEM_REGEX)
		) {
			throw new Error('invalid pem');
		}
		if (
			typeof this.fingerprint !== 'string' ||
			!this.fingerprint.match(FINGERPRINT_REGEX)
		) {
			console.log(this.fingerprint);
			console.log(FINGERPRINT_REGEX);
			console.log(this.fingerprint.match(FINGERPRINT_REGEX));
			throw new Error('invalid fingerprint');
		}
		await validateOrReject(this);
	};

	return DeviceApiKey;
};

export default DeviceApiKeyModel;
