import { NextApiRequest, NextApiResponse } from 'next';
import { getServerSession } from 'next-auth/next';
import { authOptions } from '../[...nextauth]';
import {
	EscrowedDevicePrivateKeyFactory,
	DevicePublicKeyFactory,
} from '@/lib/db';
import * as apiRequests from '@/lib/api/requests';
import * as apiResponses from '@/lib/api/responses';
import { Session } from 'next-auth';

export default async (req: NextApiRequest, res: NextApiResponse) => {
	// Get the user's session
	// TODO: Fix this ts-ignore s.t. we can type check session
	// @ts-ignore
	const session: Session = await getServerSession(req, res, authOptions);
	if (!session) {
		res.status(401).send({}); // Unauthorized
	}

	console.log('session', session);
	console.log('session.userId', session.userId);
	console.log('request method', req.method);

	// Handle POST request
	if (req.method === 'POST') {
		let { device_public_key, escrowed_device_private_key } =
			req.body as apiRequests.EscrowDeviceKeyPair;

		// If the ecdsa fingerprints are not the same, return an error
		if (
			device_public_key.ecdsa_fingerprint !==
			escrowed_device_private_key.device_public_key_ecdsa_fingerprint
		) {
			res.status(400).send('bad request'); // Bad Request
			return;
		}

		// Auto Set the user_id to the current user for both the public key and the escrowed key
		// Set the authorized flag to true for the public key
		device_public_key = { ...device_public_key, user_id: session.userId }; // , authorized: true };
		escrowed_device_private_key = {
			...escrowed_device_private_key,
			user_id: session.userId,
		};

		try {
			escrowed_device_private_key =
				await EscrowedDevicePrivateKeyFactory.create(
					escrowed_device_private_key
				);
		} catch (e) {
			console.log('Error creating escrowed key: ', e);
			res
				.status(500)
				.send(
					'internal server error: could not create escrowed device private key'
				); // Bad Request
			return;
		}

		try {
			device_public_key = await DevicePublicKeyFactory.create(
				device_public_key
			);
		} catch (e) {
			console.log('Error creating device public key: ', e);
			res
				.status(500)
				.send('internal server error: could not create device public key'); // Bad Request
			return;
		}

		const escrowed_device_key_pair = {
			device_public_key,
			escrowed_device_private_key,
		};

		res.status(200).send({
			escrowed_device_key_pair,
		} as apiResponses.EscrowDeviceKeyPair);
	}

	// Handle GET request
	if (req.method === 'GET') {
		const user_id = session.userId;
		const escrowed_device_private_key =
			await EscrowedDevicePrivateKeyFactory.readByUserId(user_id);
		if (!escrowed_device_private_key) {
			res.status(404).send('not found'); // Not Found
			return;
		}
		const device_public_key =
			await DevicePublicKeyFactory.readByEcdsaFingerprint(
				escrowed_device_private_key.device_public_key_ecdsa_fingerprint
			);

		const escrowed_device_key_pair = {
			device_public_key,
			escrowed_device_private_key,
		};

		res.status(200).send({
			escrowed_device_key_pair,
		} as apiResponses.EscrowDeviceKeyPair);
	}
};
