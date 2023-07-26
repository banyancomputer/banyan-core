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

/**
 * DevicePublicKey:
 * The public key material for a given user's device.
 * Used for authenticating against Banyan Services and Recieving Shares.
 * @property id - the generated id of the public key record  -- unique id for the record
 * @property user_id - the id of the user who owns this key material.
//  * @property authorized - whether or not this key is authorized for use with the user's account 
 * @property ecdsa_fingerprint - the fingerprint of the device's public ecdsa key -- compressed point EC public key fingerprint. This is a unique identifier for the public key!
 * @property ecdsa_spki_pem - the spki formatted PEM of the user's ecdsa public key
 * @property ecdh_spki_pem - the spki formatted PEM of the user's ecdh public key
 */
@Entity({ name: 'device_public_keys' })
export class DevicePublicKeyEntity {
	@PrimaryGeneratedColumn('uuid')
	id!: string;

	@Column({ type: 'uuid' })
	user_id!: string;

	// TODO: What do we do with this?
	// @Column({ type: 'boolean', default: false })
	// authorized!: boolean;

	@Index({ unique: true }) // Enforce that each public key has a unique fingerprint
	@Column({ type: 'varchar' })
	ecdsa_fingerprint!: string;

	@Column({ type: 'varchar' })
	ecdsa_spki_pem!: string;

	@Column({ type: 'varchar' })
	ecdh_spki_pem!: string;

	@BeforeInsert()
	addId() {
		this.id = uuidv4();
	}

	@BeforeInsert()
	@BeforeUpdate()
	async validate() {
		// TODO: better validation
		await validateOrReject(this);
	}
}

/**
 * IPublicKey - interface for PublicKey for Frontend Javascript
 */
export interface DevicePublicKey {
	id: string;
	user_id: string;
	// authorized: boolean;
	ecdsa_fingerprint: string;
	ecdsa_spki_pem: string;
	ecdh_spki_pem: string;
}
