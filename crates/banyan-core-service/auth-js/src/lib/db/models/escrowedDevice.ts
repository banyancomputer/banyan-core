import { Sequelize, DataTypes, ModelDefined, Model } from 'sequelize';
import { validateOrReject } from 'class-validator';
import { EscrowedDevice as EscrowedDeviceAttributes } from '@/lib/interfaces';
import { BadModelFormat } from './errors';
import { isPem, isPrettyFingerprint, prettyFingerprintApiKeyPem } from '@/lib/utils';

interface EscrowedDeviceInstance
	extends Model<EscrowedDeviceAttributes>,
		EscrowedDeviceAttributes {
	validate(): Promise<void>;
}

const EscrowedDeviceModel = (
	sequelize: Sequelize
): ModelDefined<EscrowedDeviceAttributes, {}> => {
	const EscrowedDevice = sequelize.define<EscrowedDeviceInstance>(
		'EscrowedDevice',
		{
			id: {
				type: DataTypes.UUID,
				defaultValue: DataTypes.UUIDV4,
				primaryKey: true,
			},
			accountId: {
				type: DataTypes.STRING,
				allowNull: false,
			},
            apiKeyPem: {
                type: DataTypes.STRING,
                allowNull: false,
            },
			encryptionKeyPem: {
				type: DataTypes.STRING,
				allowNull: false,
			},
			wrappedApiKey: {
				type: DataTypes.STRING,
				allowNull: false,
			},
			wrappedEncryptionKey: {
				type: DataTypes.STRING,
				allowNull: false,
			},
			passKeySalt: {
				type: DataTypes.STRING,
				allowNull: false,
			},
		},
		{
			createdAt: false, // don't need these
			updatedAt: false, // don't need these
			underscored: true, // use snake_case rather than camelCase for db read / write
			hooks: {
				beforeCreate: async (escrowedDevice: EscrowedDeviceInstance) => {
					await escrowedDevice.validate().catch((err) => {
						throw new BadModelFormat(err);
					});
				},
				beforeUpdate: async (escrowedDevice: EscrowedDeviceInstance) => {
					await escrowedDevice.validate().catch((err) => {
						throw new BadModelFormat(err);
					});
				},
			},
			tableName: 'escrowed_devices',
		}
	);

	EscrowedDevice.prototype.validate = async function () {
        if (!isPem(this.apiKeyPem)) {
            console.log('invalid api key pem');
            console.log(this.apiKeyPem);
            console.log(typeof this.apiKeyPem);
            console.log(isPem(this.apiKeyPem));
            throw new Error('invalid api key pem');
        }
		if (!isPem(this.encryptionKeyPem)) {
			throw new Error('invalid encryption key pem');
		}
		// Make sure all other fields are not null, base64 strings
		if (
			!this.wrappedApiKey ||
			!this.wrappedEncryptionKey ||
			!this.passKeySalt
		) {
			throw new Error(
				'invalid wrapped api key, wrapped encryption key, or pass key salt'
			);
		}
		await validateOrReject(this);
	};
	return EscrowedDevice;
};

export default EscrowedDeviceModel;
