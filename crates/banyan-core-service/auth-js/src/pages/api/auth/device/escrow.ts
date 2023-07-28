import { NextApiRequest, NextApiResponse } from 'next';
import { getServerSession } from 'next-auth/next';
import { authOptions } from '../[...nextauth]';
import { AccountFactory, DeviceApiKeyFactory } from '@/lib/db';
import { EscrowedDevice, verifyEscrowedDevice } from '@/lib/interfaces';
import * as apiRequests from '@/lib/api/requests';
import { Session } from 'next-auth';
import { FINGERPRINT_REGEX, PEM_REGEX } from '@/lib/utils';

export default async (req: NextApiRequest, res: NextApiResponse) => {
	// Get the user's session
	// TODO: Fix this ts-ignore s.t. we can type check session
	// @ts-ignore
	const session: Session = await getServerSession(req, res, authOptions);
	
	if (!session) {
		res.status(401).send({}); // Unauthorized
	}

	// Get the user's account id
	const provider_id = session.providerId;
	// Find the account in the database
	const account_id = await AccountFactory.idFromProviderId(provider_id);
	if (!account_id) {
		res.status(404).send('account not found'); // Not Found
		return;
	}
	
	// Handle POST request
	if (req.method === 'POST') {
		const { escrowed_device, api_key_pem, encryption_key_pem } = JSON.parse(
			req.body
		) as apiRequests.EscrowDevice;

		// If the ecdsa fingerprints are not the same, return an error
		if (
			escrowed_device === null ||
			api_key_pem === null ||
			encryption_key_pem === null
		) {
			console.log('Missing data: ', escrowed_device, api_key_pem, encryption_key_pem);
			res.status(400).send('bad request -- missing data'); // Bad Request
			return;
		}
		const { ecdsa_fingerprint } = escrowed_device as EscrowedDevice;

		// Regex the pems and verify the escrowed device
		if (
			!api_key_pem.match(PEM_REGEX) ||
			!encryption_key_pem.match(PEM_REGEX) ||
			!ecdsa_fingerprint.match(FINGERPRINT_REGEX) ||
			!verifyEscrowedDevice(escrowed_device as EscrowedDevice)
		) {
			console.log('Bad format: ', escrowed_device, api_key_pem, encryption_key_pem);
			console.log('Regex: ', ecdsa_fingerprint.match(FINGERPRINT_REGEX));
			res.status(400).send('bad request -- bad format'); // Bad Request
			return;
		}

		const deviceApiKey = {
			account_id,
			fingerprint: ecdsa_fingerprint,
			pem: api_key_pem,
		};

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
		try {
			const resp = await AccountFactory.readEscrowedDevice(account_id);
			if (!resp) {
				res.status(404).send('not found -- no escrowed device'); // Not Found
				return;
			}
			const { escrowed_device, encryption_key_pem, api_key_pem } = resp;
			// const { escrowed_device, api_key_pem, encryption_key_pem } = await AccountFactory.readEscrowedDevice(account_id);
			if (!escrowed_device || !api_key_pem || !encryption_key_pem) {
				res.status(404).send('not found -- missing key material!'); // Not Found
				return;
			}
			res
				.status(200)
				.send({ escrowed_device, api_key_pem, encryption_key_pem });
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
