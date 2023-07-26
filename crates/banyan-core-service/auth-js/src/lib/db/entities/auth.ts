/**
 * Note: These are the default entities used by NextAuth + TypeORM. Kept here for reference.
 * See:
 *   @auth/typeorm-adapter v1.0.1
 *   https://github.com/nextauthjs/next-auth/blob/main/packages/adapter-typeorm/src/entities.ts
 */

import {
	Entity,
	PrimaryGeneratedColumn,
	Column,
	ManyToOne,
	OneToMany,
} from 'typeorm';
import transformer from './transformer';

@Entity({ name: 'users' })
export class UserEntity {
	@PrimaryGeneratedColumn('uuid')
	id!: string;

	@Column({ type: 'varchar', nullable: true })
	name!: string | null;

	@Column({ type: 'varchar', nullable: true, unique: true })
	email!: string | null;

	@Column({ type: 'varchar', nullable: true, transformer: transformer.date })
	emailVerified!: string | null;

	@Column({ type: 'varchar', nullable: true })
	image!: string | null;

	@OneToMany(() => SessionEntity, (session) => session.userId)
	sessions!: SessionEntity[];

	@OneToMany(() => AccountEntity, (account) => account.userId)
	accounts!: AccountEntity[];
}

@Entity({ name: 'accounts' })
export class AccountEntity {
	@PrimaryGeneratedColumn('uuid')
	id!: string;

	@Column({ type: 'uuid' })
	userId!: string;

	@Column()
	type!: string;

	@Column()
	provider!: string;

	@Column()
	providerAccountId!: string;

	@Column({ type: 'varchar', nullable: true })
	refresh_token!: string | null;

	@Column({ type: 'varchar', nullable: true })
	access_token!: string | null;

	@Column({
		nullable: true,
		type: 'bigint',
		transformer: transformer.bigint,
	})
	expires_at!: number | null;

	@Column({ type: 'varchar', nullable: true })
	token_type!: string | null;

	@Column({ type: 'varchar', nullable: true })
	scope!: string | null;

	@Column({ type: 'varchar', nullable: true })
	id_token!: string | null;

	@Column({ type: 'varchar', nullable: true })
	session_state!: string | null;

	@ManyToOne(() => UserEntity, (user) => user.accounts, {
		createForeignKeyConstraints: true,
	})
	user!: UserEntity;
}

@Entity({ name: 'sessions' })
export class SessionEntity {
	@PrimaryGeneratedColumn('uuid')
	id!: string;

	@Column({ unique: true })
	sessionToken!: string;

	@Column({ type: 'uuid' })
	userId!: string;

	@Column({ transformer: transformer.date })
	expires!: string;

	@ManyToOne(() => UserEntity, (user) => user.sessions)
	user!: UserEntity;
}

@Entity({ name: 'verification_tokens' })
export class VerificationTokenEntity {
	@PrimaryGeneratedColumn('uuid')
	id!: string;

	@Column()
	token!: string;

	@Column()
	identifier!: string;

	@Column({ transformer: transformer.date })
	expires!: string;
}
