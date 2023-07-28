import { Sequelize, DataTypes, Model, ModelDefined } from 'sequelize';
import { v4 as uuidv4 } from 'uuid';
import { validateOrReject } from 'class-validator';

export interface AllowedEmailAttributes {
	id: string;
	email: string;
}

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
			createdAt: false,
			updatedAt: false,

			hooks: {
				beforeCreate: async (allowedEmail: AllowedEmailInstance) => {
					await allowedEmail.addId();
					await allowedEmail.validate();
				},
				beforeUpdate: async (allowedEmail: AllowedEmailInstance) => {
					await allowedEmail.validate();
				},
			},
			tableName: 'allowed_emails',
		}
	);

	AllowedEmail.prototype.addId = async function () {
		this.id = uuidv4();
	};

	AllowedEmail.prototype.validate = async function () {
		await validateOrReject(this);
	};

	return AllowedEmail;
};

export default AllowedEmailModel;
