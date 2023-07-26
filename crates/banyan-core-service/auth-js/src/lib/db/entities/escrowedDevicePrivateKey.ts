import {
	Entity,
	PrimaryGeneratedColumn,
	Column,
	BeforeInsert,
	BeforeUpdate,
	OneToOne,
	Index,
} from 'typeorm';
import { DevicePublicKeyEntity, DevicePublicKey } from './devicePublicKey';
import { v4 as uuidv4 } from 'uuid';
import { validateOrReject } from 'class-validator';

/**
 * EscrowedPrivateKey:
 * A user's escrowed device private key material for Signing requests to Banyan Services and Recovering Shares.
 * Primary key material for the user's account, used to authorize additional devices.
 * Should be generated locally on account creation.
 * Corresponding device public key acts as primary device public key for the user.
 *
 * @property id - the generated id of the record  -- unique id for the record
 * @property user_id - the id of the user who owns this key material. See user.ts for more information
 * @property device_public_key_ecdsa_fingerprint - the fingerprint of the device's public ecdsa key -- compressed point EC public key fingerprint.
 * @property wrapped_ecdsa_pkcs8 - the wrapped private ecdsa key - generated using webCrytpo.subtle.wrapKey with pkcs8 format and exporint as base64
 * @property wrapped_ecdh_pkcs8 - the wrapped private ecdh key - generated using webCrytpo.subtle.wrapKey with pkcs8 format and exporint as base64
 * @property passkey_salt - the salt used to derive the user's pass key.
 */
@Entity({ name: 'escrowed_device_private_keys' })
export class EscrowedDevicePrivateKeyEntity {
	@PrimaryGeneratedColumn('uuid')
	id!: string;

	@Index({ unique: true }) // Enforce that each user only has one escrowed private key pair
	@Column({ type: 'uuid' })
	user_id!: string;

	@Column({ type: 'varchar' })
	device_public_key_ecdsa_fingerprint!: string;

	@Column({ type: 'varchar' })
	wrapped_ecdsa_pkcs8!: string;

	@Column({ type: 'varchar' })
	wrapped_ecdh_pkcs8!: string;

	@Column({ type: 'varchar' })
	passkey_salt!: string;

	@BeforeInsert()
	addId() {
		this.id = uuidv4();
	}

	@BeforeInsert()
	@BeforeUpdate()
	async validate() {
		// TODO: better validation
		// Validate the record
		await validateOrReject(this);
	}
}

/**
 * Interface type for Frontend Javascript
 */
export interface EscrowedDevicePrivateKey {
	id: string;
	user_id: string;
	device_public_key_ecdsa_fingerprint: string;
	wrapped_ecdsa_pkcs8: string;
	wrapped_ecdh_pkcs8: string;
	passkey_salt: string;
}
