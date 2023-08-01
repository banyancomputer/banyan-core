import { NextApiRequest, NextApiResponse } from 'next';
import { getServerSession } from 'next-auth/next';
import { DeviceApiKeyFactory } from '@/lib/db';
import { authOptions } from '@/pages/api/auth/[...nextauth]';
import { Session } from 'next-auth';
import { AccountFactory } from '@/lib/db';
import { prettyFingerprintApiKeySpki, publicPemWrap } from '@/lib/utils'; 
import * as errors from '@/lib/db/models/errors';
import { b64UrlDecode } from '@/lib/utils/b64';

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

	const urlSpki = req.query.spki;
	if (!urlSpki || typeof urlSpki !== 'string') {
		res.status(400).send('bad request -- missing spki'); // Bad Request
		return;
	}
	const spki = b64UrlDecode(urlSpki as string);
	console.log('spki: ', spki);
	// Get the fingerprint from the spki
	const fingerprint = await prettyFingerprintApiKeySpki(spki);

        // Check if the device api key already exists
        const maybeDeviceApiKey = await DeviceApiKeyFactory.readByFingerprint(fingerprint as string);
        if (maybeDeviceApiKey) {
            res.status(409).send('conflict'); // Conflict
            return;
        }

		const pem = publicPemWrap(spki);

		const deviceApiKey = {
			accountId,
			fingerprint,
			pem,
		};

		try {
			await DeviceApiKeyFactory.create(deviceApiKey);
		} catch (e: any) {
			if (e.name === errors.BadModelFormat.name) {
				res.status(400).send('bad request -- bad format'); // Bad Request
				return;
			}
			console.log('Error creating device api key: ', e);
			res.status(500).send('internal server error'); // Bad Request
			return;
		}
		res.status(200).send(deviceApiKey); // Bad Request
		return;
	}
};
