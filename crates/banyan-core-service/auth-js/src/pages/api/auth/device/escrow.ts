import { NextApiRequest, NextApiResponse } from 'next';
import { getServerSession } from 'next-auth/next';
import { authOptions } from '../[...nextauth]';
import { AccountFactory, DeviceApiKeyFactory } from '@/lib/db';
import { EscrowedDevice, verifyEscrowedDevice } from '@/lib/interfaces';
import * as apiRequests from '@/lib/api/requests';
import { Session } from 'next-auth';

export default async (req: NextApiRequest, res: NextApiResponse) => {
	// Get the user's session
	// TODO: Fix this ts-ignore s.t. we can type check session
	// @ts-ignore
	const session: Session = await getServerSession(req, res, authOptions);
	if (!session) {
		res.status(401).send({}); // Unauthorized
	}

	// Handle POST request
	if (req.method === 'POST') {
		const { escrowed_device, api_key_pem, encryption_key_pem } = JSON.parse(
			req.body
		) as apiRequests.EscrowDevice;

		// If the ecdsa fingerprints are not the same, return an error
		if (
			!verifyEscrowedDevice(escrowed_device) ||
			api_key_pem === null ||
			encryption_key_pem === null
		) {
			res.status(400).send('bad request'); // Bad Request
			return;
		}

		const provider_id = session.providerId;
		console.log('Provider ID: ', provider_id);
		
		// Find the account in the database
		const account_id = await AccountFactory.idFromProviderId(provider_id);
		console.log('Account ID: ', account_id);
		if (!account_id) {
			res.status(404).send('not found'); // Not Found
			return;
		}

		const { ecdsa_fingerprint } = escrowed_device as EscrowedDevice;

		const deviceApiKey = {
			account_id,
			fingerprint: ecdsa_fingerprint,
			pem: api_key_pem,
		};

		// console.log(
		// 	'Updating key material: ',
		// 	account_id,
		// 	escrowed_device,
		// 	ecdsa_spki_pem,
		// 	ecdh_spki_pem
		// );
		console.log('Updating key material: ', account_id, escrowed_device);
		console.log('Creating device api key: ', deviceApiKey);

		try {
			await AccountFactory.updateEscrowedDevice(
				account_id,
				escrowed_device,
				api_key_pem,
				encryption_key_pem
			);
		} catch (e) {
			console.log('Error creating escrowed key material: ', e);
			res
				.status(500)
				.send('internal server error: could not escrow device key material'); // Bad Request
			return;
		}

		console.log('Creating device api key: ', deviceApiKey);

		try {
			await DeviceApiKeyFactory.create(deviceApiKey);
		} catch (e) {
			console.log('Error creating device api key: ', e);
			res
				.status(500)
				.send('internal server error: could not create device api key'); // Bad Request
			return;
		}

		res.status(200).send({}); // OK
		return;
	}

	// Handle GET request
	if (req.method === 'GET') {

		const provider_id = session.providerId;
		console.log('Provider ID: ', provider_id);
		
		// Find the account in the database
		const account_id = await AccountFactory.idFromProviderId(provider_id);
		console.log('Account ID: ', account_id);
		if (!account_id) {
			res.status(404).send('not found'); // Not Found
			return;
		}

		try {
			const resp = await AccountFactory.readEscrowedDevice(account_id);
			if (!resp) {
				res.status(404).send('not found'); // Not Found
				return;
			}
			const { escrowed_device, encryption_key_pem, api_key_pem } = resp;
			// const { escrowed_device, api_key_pem, encryption_key_pem } = await AccountFactory.readEscrowedDevice(account_id);
			if (!escrowed_device || !api_key_pem || !encryption_key_pem) {
				res.status(404).send('not found'); // Not Found
				return;
			}
			res.status(200).send({ escrowed_device, api_key_pem, encryption_key_pem });
		} catch (e) {
			console.log('Error reading escrowed device: ', e);
			res
				.status(500)
				.send('internal server error: could not read escrowed device'); // Bad Request
			return;
		}
		return;
	}
};
