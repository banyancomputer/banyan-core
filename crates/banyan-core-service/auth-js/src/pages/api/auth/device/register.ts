import { NextApiRequest, NextApiResponse } from 'next';
import { getServerSession } from 'next-auth/next';
import { DeviceApiKeyFactory } from '@/lib/db';
import { authOptions } from '@/pages/api/auth/[...nextauth]';
import { Session } from 'next-auth';
import { AccountFactory } from '@/lib/db';
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
    const account_id = await AccountFactory.idFromProviderId(provider_id);
    if (!account_id) {
        res.status(404).send('account not found'); // Not Found
        return;
    }

    const { fingerprint, pem } = req.query;

    if (fingerprint && pem) {
        if (
            typeof fingerprint !== 'string' ||
            !FINGERPRINT_REGEX.test(fingerprint)
        ) {
            res.status(400).send('bad request -- bad fingerprint'); // Bad Request
            return;
        }
        if (
            typeof pem !== 'string' ||
            !PEM_REGEX.test(pem)
        ) {
            console.log('Bad pem: ', pem);
            console.log('Regex: ', PEM_REGEX.test(pem as string));
            res.status(400).send('bad request -- bad pem'); // Bad Request
            return;
        }
        // TODO: Check if the fingerprint matches the pem
    } else {
        res.status(400).send('bad request -- missing fingerprint or pem'); // Bad Request
        return;
    }

	if (req.method === 'GET') {
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
			fingerprint,
			pem
		};
		try {
			await DeviceApiKeyFactory.create(deviceApiKey);
		} catch (e) {
			console.log('Error creating device api key: ', e);
			res.status(500).send('internal server error'); // Bad Request
			return;
		}
		res.status(200).send(deviceApiKey); // Bad Request
		return;
	}
};
