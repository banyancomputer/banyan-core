import { DataTypes, Model, ModelDefined, Sequelize } from 'sequelize';
import { validateOrReject } from 'class-validator';
import { BadModelFormat } from './errors';
import { EscrowedDevice as EscrowedDeviceAttributes } from '@app/lib/interfaces';
import { isPem } from '@app/utils';

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
            apiPublicKeyPem: {
                type: DataTypes.STRING,
                allowNull: false,
            },
            encryptionPublicKeyPem: {
                type: DataTypes.STRING,
                allowNull: false,
            },
            encryptedPrivateKeyMaterial: {
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
                beforeCreate: async(escrowedDevice: EscrowedDeviceInstance) => {
                    await escrowedDevice.validate().catch((err) => {
                        throw new BadModelFormat(err);
                    });
                },
                beforeUpdate: async(escrowedDevice: EscrowedDeviceInstance) => {
                    await escrowedDevice.validate().catch((err) => {
                        throw new BadModelFormat(err);
                    });
                },
            },
            tableName: 'escrowed_devices',
        }
    );

    EscrowedDevice.prototype.validate = async function() {
        if (!isPem(this.apiPublicKeyPem)) {
            throw new Error('invalid api public key pem');
        }
        if (!isPem(this.encryptionPublicKeyPem)) {
            throw new Error('invalid encryption public key pem');
        }
        // Make sure all other fields are not null, base64 strings
        if (
            !this.encryptedPrivateKeyMaterial ||
			!this.passKeySalt
        ) {
            throw new Error(
                'invalid encrypted private key material or pass key salt'
            );
        }
        await validateOrReject(this);
    };

    return EscrowedDevice;
};

export default EscrowedDeviceModel;
