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
 * PublicKey - represents a user's public key information for a device
 * @property id - the pg generated id of the allowed user
 * @property owner - the uid of the user
 * @property ecdsa_fingerprint - the fingerprint of the user's ecdsa public key
 * @property ecdsa_spki - the spki of the user's ecdsa public key
 */
@Entity({ name: 'publicKey' })
export class PublicKey {
	@PrimaryGeneratedColumn('uuid')
	public id!: string;

	// The fingerprint of the
	@Index({ unique: true }) // (For now) enforce that each user can only have one key
	@Column({ type: 'varchar' })
	public ecdsa_fingerprint!: string;

	// The key owner's uid
	@Column({ type: 'varchar' })
	public owner!: string;

	// The public ecdsa key's spki
	@Column({ type: 'varchar' })
	public ecdsa_spki!: string;

	// The public ecdh key's spki
	@Column({ type: 'varchar' })
	public ecdh_spki!: string;

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
 * IPublicKey - interface for PublicKey for Frontend Javascript
 */
export interface IPublicKey {
	ecdsa_fingerprint: string;
	owner: string;
	ecdsa_spki: string;
	ecdh_spki: string;
}
