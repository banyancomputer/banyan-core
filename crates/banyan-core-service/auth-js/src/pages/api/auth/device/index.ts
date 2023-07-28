import { NextApiRequest, NextApiResponse } from 'next';
import { getServerSession } from 'next-auth/next';
import { authOptions } from '../[...nextauth]';
import { AccountFactory, DeviceApiKeyFactory } from '@/lib/db';
import { Session } from 'next-auth';

export default async (req: NextApiRequest, res: NextApiResponse) => {
	// Get the user's session
	// TODO: Fix this ts-ignore s.t. we can type check session
	// @ts-ignore
	const session: Session = await getServerSession(req, res, authOptions);
	if (!session) {
		res.status(401).send({}); // Unauthorized
	}

    // Handle Get request
    if (req.method === 'GET') {
        // Get the fingerprint from the query string
        const { fingerprint } = req.query;
       
        // Get a specific device api key if a fingerprint is provided
        if (fingerprint) {
            const deviceApiKey = await DeviceApiKeyFactory.readByFingerprint(fingerprint as string);
            if (!deviceApiKey) {
                res.status(404).send('not found'); // Not Found
                return;
            }
            res.status(200).send(deviceApiKey);
            return;
        }

        // Get the user's account id
        const provider_id = session.providerId;
        const account_id = await AccountFactory.idFromProviderId(provider_id);
        if (!account_id) {
            res.status(404).send('not found'); // Not Found
            return;
        }

        // Get all device api keys for the user
        const deviceApiKeys = await DeviceApiKeyFactory.readAllByAccountId(account_id);
        res.status(200).send(deviceApiKeys);
        return;        
    }

    // Deny all other requests
    res.status(405).send('method not allowed'); // Method Not Allowed
    return;	
};
