import { NextApiRequest, NextApiResponse } from 'next';
import { getServerSession } from 'next-auth/next';
import { DeviceApiKeyFactory } from '@/lib/db';
import { authOptions } from '@/pages/api/auth/[...nextauth]';
import { Session } from 'next-auth';
import { AccountFactory } from '@/lib/db';

export default async (req: NextApiRequest, res: NextApiResponse) => {
	// Get the user's session
	// TODO: Fix this ts-ignore s.t. we can type check session
	// @ts-ignore
	const session: Session = await getServerSession(req, res, authOptions);
	if (!session) {
		res.status(401).send({}); // Unauthorized
	}

	if (req.method === 'GET') {
		// Get the fingerprint and pem from the query string
		const { fingerprint, pem } = req.query;
		if (!fingerprint || !pem) {
			res.status(400).send('bad request'); // Bad Request
			return;
		}

		// TODO: Validate the pem
		// TODO: Validate the fingerprint

		// Check if the device api key already exists
		const maybeDeviceApiKey = await DeviceApiKeyFactory.readByFingerprint(
			fingerprint as string
		);
		if (maybeDeviceApiKey) {
			res.status(409).send('conflict'); // Conflict
			return;
		}

		// Create the device api key
		const provider_id = session.providerId;
		const account_id = await AccountFactory.idFromProviderId(provider_id);
		if (!account_id) {
			res.status(404).send('not found'); // Not Found
			return;
		}
		const deviceApiKey = {
			account_id,
			fingerprint: fingerprint as string,
			pem: pem as string,
		};
		try {
			await DeviceApiKeyFactory.create(deviceApiKey);
		} catch (e) {
			console.log('Error creating device api key: ', e);
			res.status(500).send('internal server error'); // Bad Request
			return;
		}
		res.status(200).send({}); // Bad Request
		return;
	}

	if (req.method === 'DELETE') {
		let { fingerprint } = req.query;

		if (!fingerprint) {
			res.status(400).send('bad request'); // Bad Request
			return;
		}

		// Get the user's account id
		const provider_id = session.providerId;
		const account_id = await AccountFactory.idFromProviderId(provider_id);
		if (!account_id) {
			res.status(404).send('not found'); // Not Found
			return;
		}

		try {
			await DeviceApiKeyFactory.deleteByAccountIdAndFingerprint(
				account_id,
				fingerprint as string
			);
		} catch (e) {
			console.log('Error deleting device api key: ', e);
			res.status(500).send('internal server error'); // Bad Request
			return;
		}

		res.status(200).send({}); // Bad Request
		return;
	}
};
