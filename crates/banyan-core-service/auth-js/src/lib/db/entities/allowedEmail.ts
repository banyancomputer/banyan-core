import {
	Entity,
	PrimaryGeneratedColumn,
	Column,
	Index,
	BeforeInsert,
	BeforeUpdate,
} from 'typeorm';
import { v4 as uuidv4 } from 'uuid';
import { IsEmail, validateOrReject } from 'class-validator';

/**
 * AllowedEntity - represents allow-listed alpha users
 * @property id - the generated id of the allowed user
 * @property email - the email of the allowed user
 */
@Entity({ name: 'allowed_emails' })
export class AllowedEmailEntity {
	@PrimaryGeneratedColumn('uuid')
	public id!: string;

	@Index({ unique: true })
	@IsEmail()
	@Column({ type: 'varchar' })
	public email!: string;

	@BeforeInsert()
	addId() {
		this.id = uuidv4();
	}

	@BeforeInsert()
	@BeforeUpdate()
	async validate() {
		await validateOrReject(this);
	}
}
