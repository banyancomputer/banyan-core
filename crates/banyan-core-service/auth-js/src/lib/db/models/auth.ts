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

// import {
// 	Entity,
// 	PrimaryGeneratedColumn,
// 	Column,
// 	ManyToOne,
// 	OneToMany,
// } from 'typeorm';
// import transformer from './transformer';

// /**
//  * Note: These are the default entities used by NextAuth + TypeORM. Kept here for reference.
//  * 		 Changes are explicitly noted.
//  * See:
//  *   @auth/typeorm-adapter v1.0.1
//  *   https://github.com/nextauthjs/next-auth/blob/main/packages/adapter-typeorm/src/entities.ts
//  */

// // Note: Not sure how UserEntity is acutally used in the context of our Application logic.
// //       The fields may be useful for frontend customization, but it's not clear how to use the ID as
// //       a foreign key in other entities. For now, rely on providerAccountId as an account identifier for
// //       the user's application data.
// @Entity({ name: 'users' })
// export class UserEntity {
// 	@PrimaryGeneratedColumn('uuid')
// 	id!: string;

// 	@Column({ type: 'varchar', nullable: true })
// 	name!: string | null;

// 	@Column({ type: 'varchar', nullable: true, unique: true })
// 	email!: string | null;

// 	@Column({ type: 'varchar', nullable: true, transformer: transformer.date })
// 	emailVerified!: string | null;

// 	@Column({ type: 'varchar', nullable: true })
// 	image!: string | null;

// 	@OneToMany(() => SessionEntity, (session) => session.userId)
// 	sessions!: SessionEntity[];

// 	@OneToMany(() => AccountEntity, (account) => account.userId)
// 	accounts!: AccountEntity[];
// }

// @Entity({ name: 'accounts' })
// export class AccountEntity {
// 	@PrimaryGeneratedColumn('uuid')
// 	id!: string;

// 	/// Note: Added field
// 	///       User's private key material escrowed to our Auth Service
// 	@Column({ type: 'varchar', nullable: true })
// 	escrowed_device_blob!: string;

// 	/// Note: Added field
// 	///       User's public key material for escrowed ecdh private key material
// 	///		  Needs to be separate from escrow_key_blob to make pulling the public key material easier
// 	@Column({ type: 'varchar', nullable: true })
// 	ecdh_spki_pem!: string;

// 	/// Note: Added field
// 	///	      User's public key material for escrowed ecdsa private key material.
// 	/// 	  Not strictly necessary to include here, but makes recovery easier.
// 	@Column({ type: 'varchar', nullable: true })
// 	ecdsa_spki_pem!: string;

// 	@Column({ type: 'uuid' })
// 	userId!: string;

// 	@Column()
// 	type!: string;

// 	@Column()
// 	provider!: string;

// 	@Column()
// 	providerAccountId!: string;

// 	@Column({ type: 'varchar', nullable: true })
// 	refresh_token!: string | null;

// 	@Column({ type: 'varchar', nullable: true })
// 	access_token!: string | null;

// 	@Column({
// 		nullable: true,
// 		type: 'bigint',
// 		transformer: transformer.bigint,
// 	})
// 	expires_at!: number | null;

// 	@Column({ type: 'varchar', nullable: true })
// 	token_type!: string | null;

// 	@Column({ type: 'varchar', nullable: true })
// 	scope!: string | null;

// 	@Column({ type: 'varchar', nullable: true })
// 	id_token!: string | null;

// 	@Column({ type: 'varchar', nullable: true })
// 	session_state!: string | null;

// 	@ManyToOne(() => UserEntity, (user) => user.accounts, {
// 		createForeignKeyConstraints: true,
// 	})
// 	user!: UserEntity;
// }

// @Entity({ name: 'sessions' })
// export class SessionEntity {
// 	@PrimaryGeneratedColumn('uuid')
// 	id!: string;

// 	@Column({ unique: true })
// 	sessionToken!: string;

// 	@Column({ type: 'uuid' })
// 	userId!: string;

// 	@Column({ transformer: transformer.date })
// 	expires!: string;

// 	@ManyToOne(() => UserEntity, (user) => user.sessions)
// 	user!: UserEntity;
// }

// @Entity({ name: 'verification_tokens' })
// export class VerificationTokenEntity {
// 	@PrimaryGeneratedColumn('uuid')
// 	id!: string;

// 	@Column()
// 	token!: string;

// 	@Column()
// 	identifier!: string;

// 	@Column({ transformer: transformer.date })
// 	expires!: string;
// }
