import { DataTypes, Model, ModelDefined, Sequelize } from 'sequelize';
import { validateOrReject } from 'class-validator';
import { BadModelFormat } from './errors';
import { AllowedEmail as AllowedEmailAttributes } from '@/lib/interfaces';

interface AllowedEmailInstance
	extends Model<AllowedEmailAttributes>,
		AllowedEmailAttributes {
	addId(): Promise<void>;
	validate(): Promise<void>;
}

const AllowedEmailModel = (
	sequelize: Sequelize
): ModelDefined<AllowedEmailAttributes, {}> => {
	const AllowedEmail = sequelize.define<AllowedEmailInstance>(
		'AllowedEmail',
		{
			id: {
				type: DataTypes.UUID,
				defaultValue: DataTypes.UUIDV4,
				primaryKey: true,
			},
			email: {
				type: DataTypes.STRING,
				allowNull: false,
				unique: true,
				validate: {
					isEmail: true,
				},
			},
		},
		{
			createdAt: false, // don't need these
			updatedAt: false, // don't need these
			underscored: true, // use snake_case rather than camelCase for db read / write
			hooks: {
				beforeCreate: async (allowedEmail: AllowedEmailInstance) => {
					await allowedEmail.validate().catch((err) => {
						throw new BadModelFormat(err);
					});
				},
				beforeUpdate: async (allowedEmail: AllowedEmailInstance) => {
					await allowedEmail.validate().catch((err) => {
						throw new BadModelFormat(err);
					});
				},
			},
			tableName: 'allowed_emails',
		}
	);

	AllowedEmail.prototype.validate = async function () {
		await validateOrReject(this);
	};

	return AllowedEmail;
};

export default AllowedEmailModel;
