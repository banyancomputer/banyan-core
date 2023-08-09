import { NextApiRequest, NextApiResponse } from 'next';
import { getServerSession } from 'next-auth/next';
import { Session } from 'next-auth';
import { authOptions } from '../[...nextauth]';
import { AccountFactory, DeviceApiKeyFactory } from '@/lib/db';
import { isPrettyFingerprint } from '@/utils';

export default async (req: NextApiRequest, res: NextApiResponse) => {
	// Get the user's session
	// TODO: Fix this ts-ignore s.t. we can type check session
	// @ts-ignore
	const session: Session = await getServerSession(req, res, authOptions);
	if (!session) {
		res.status(401).send({}); // Unauthorized
	}

	// Get the user's account id
	const providerId = session.providerId;
	const accountId = await AccountFactory.idFromProviderId(providerId);
	if (!accountId) {
		res.status(404).send('account not found'); // Not Found

		return;
	}

	const fingerprint = req.query.fingerprint;
	if (fingerprint) {
		if (typeof fingerprint !== 'string' || !isPrettyFingerprint(fingerprint)) {
			res.status(400).send('bad request'); // Bad Request

			return;
		}
	}

	// Handle Get request
	if (req.method === 'GET') {
		// Get the fingerprint from the query string
		const { fingerprint } = req.query;

		// Get a specific device api key if a fingerprint is provided
		if (!fingerprint) {
			const deviceApiKeys = await DeviceApiKeyFactory.readAllByAccountId(
				accountId
			);
			res.status(200).send(deviceApiKeys);

			return;
		}
		const deviceApiKey = await DeviceApiKeyFactory.readByFingerprint(
			fingerprint as string
		);
		if (!deviceApiKey) {
			res.status(404).send('not found'); // Not Found

			return;
		}
		res.status(200).send(deviceApiKey);

		return;
	}

	if (req.method === 'DELETE') {
		if (!fingerprint) {
			res.status(400).send('bad request'); // Bad Request

			return;
		}
		try {
			await DeviceApiKeyFactory.deleteByAccountIdAndFingerprint(
				accountId,
				fingerprint
			);
		} catch (e) {
			console.log('Error deleting device api key: ', e);
			res.status(500).send('internal server error'); // Bad Request

			return;
		}
		res.status(200).send({}); // Bad Request

		return;
	}

	// Deny all other requests
	res.status(405).send('method not allowed'); // Method Not Allowed
};
