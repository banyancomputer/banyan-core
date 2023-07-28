import { Sequelize, DataTypes, Model, ModelDefined } from 'sequelize';
import { v4 as uuidv4 } from 'uuid';
import { validateOrReject } from 'class-validator';
import { DeviceApiKey as DeviceApiKeyAttributes } from '@/lib/interfaces'

interface DeviceApiKeyInstance
	extends Model<DeviceApiKeyAttributes>,
		DeviceApiKeyAttributes {
	addId(): Promise<void>;
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
					await deviceApiKey.addId();
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

	DeviceApiKey.prototype.addId = async function () {
		this.id = uuidv4();
	};

	DeviceApiKey.prototype.validate = async function () {
		// TODO: better validation for PEM
		await validateOrReject(this);
	};

	return DeviceApiKey;
};

export default DeviceApiKeyModel;