import { models } from '@auth/sequelize-adapter';
import { Model, ModelDefined, Sequelize } from 'sequelize';
import { Account as AccountAttributes } from '@app/lib/interfaces';

interface AccountInstance extends Model<AccountAttributes>, AccountAttributes {
    validate(): Promise<void>;
}

const AccountModel = (
    sequelize: Sequelize
): ModelDefined<AccountAttributes, {}> => {
    const Account = sequelize.define<AccountInstance>(
        'Account',
        {
            ...models.Account,
        },
        {
            createdAt: false,
            updatedAt: false,
            tableName: 'accounts',
        }
    );

    return Account;
};

export default AccountModel;
