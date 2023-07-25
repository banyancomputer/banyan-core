import {
	Entity,
	PrimaryGeneratedColumn,
	Column,
	Index,
	BeforeInsert,
	BeforeUpdate,
} from 'typeorm';
import { v4 as uuidv4 } from 'uuid';
import { validateOrReject } from 'class-validator';

// TODO: Implement proper validation with Class Validator
/**
 * EscrowedKey - represents a user's escrowed key for using web service
 * @property id - the pg generated id of the allowed user
 * @property owner - the uid of the user
 * @property ecdsa_pubkey_fingerprint - the fingerprint of the user's ecdsa public key
 * @property wrapped_ecdsa_privkey_pkcs8 - the wrapped private ecdsa key
 * @property wrapped_ecdh_privkey_pkcs8 - the wrapped private ecdh key
 * @property passkey_salt - the salt used to derive the user's pass key
 */
@Entity({ name: 'escrowedKey' })
export class EscrowedKey {
	@PrimaryGeneratedColumn('uuid')
	public id!: string;

	// The key owner's uid
	@Index({ unique: true }) // (For now) enforce that each user can only have one key
	@Column({ type: 'varchar' })
	public owner!: string;

	// The public ecdsa key's fingerprint
	@Column({ type: 'varchar' })
	public ecdsa_pubkey_fingerprint!: string;

	// The wrapped private ecdsa key
	@Column({ type: 'varchar' })
	public wrapped_ecdsa_pkcs8!: string;

	// The wrapped private ecdh key
	@Column({ type: 'varchar' })
	public wrapped_ecdh_pkcs8!: string;

	// The salt used to derive the user's pass key
	@Column({ type: 'varchar' })
	public passkey_salt!: string;

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

/**
 * Interface type for Frontend Javascript
 */
export interface IEscrowedKey {
	owner: string;
	ecdsa_pubkey_fingerprint: string;
	wrapped_ecdsa_pkcs8: string;
	wrapped_ecdh_pkcs8: string;
	passkey_salt: string;
}
