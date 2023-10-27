import { DataTypes, Model, ModelDefined, Sequelize } from 'sequelize';
import { validateOrReject } from 'class-validator';

import { BadModelFormat } from './errors';
import { DeviceApiKey as DeviceApiKeyAttributes } from '@/lib/interfaces';
import { hexFingerprintApiKeyPem, isHexFingerprint, isPem } from '@/utils';

interface DeviceApiKeyInstance
    extends Model<DeviceApiKeyAttributes>,
    DeviceApiKeyAttributes {
    generateFingerprint(): Promise<void>;
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
            accountId: {
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
            createdAt: false, // don't need these
            updatedAt: false, // don't need these
            underscored: true, // use snake_case rather than camelCase for db read / write
            hooks: {
                beforeCreate: async(deviceApiKey: DeviceApiKeyInstance) => {
                    await deviceApiKey.generateFingerprint();
                    await deviceApiKey.validate().catch((err) => {
                        throw new BadModelFormat(err);
                    });
                },
                beforeUpdate: async(deviceApiKey: DeviceApiKeyInstance) => {
                    await deviceApiKey.validate().catch((err) => {
                        throw new BadModelFormat(err);
                    });
                },
            },
            tableName: 'device_api_keys',
        }
    );

    /** Generate the fingerprint from the PEM */
    DeviceApiKey.prototype.generateFingerprint = async function() {
        this.fingerprint = await hexFingerprintApiKeyPem(this.pem);
    };

    DeviceApiKey.prototype.validate = async function() {
        if (!isPem(this.pem)) {
            throw new Error('invalid pem');
        }
        if (!isHexFingerprint(this.fingerprint)) {
            throw new Error('invalid fingerprint');
        }
        const pemFingerprint = await hexFingerprintApiKeyPem(this.pem);
        if (!pemFingerprint || pemFingerprint !== this.fingerprint) {
            throw new Error('api key fingerprint does not match api key pem');
        }
        await validateOrReject(this);
    };

    return DeviceApiKey;
};

export default DeviceApiKeyModel;
