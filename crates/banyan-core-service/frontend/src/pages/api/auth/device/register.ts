import { NextApiRequest, NextApiResponse } from 'next';
import { getServerSession } from 'next-auth/next';
import { Session } from 'next-auth';
import { AccountFactory, DeviceApiKeyFactory } from '@/lib/db';
import { authOptions } from '@/pages/api/auth/[...nextauth]';
import { prettyFingerprintApiKeySpki, publicPemWrap } from '@/utils';
import * as errors from '@/lib/db/models/errors';
import { b64UrlDecode } from '@/utils/b64';

export default async(req: NextApiRequest, res: NextApiResponse) => {
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

    // Get the fingerprint from the spki
    const fingerprint = await prettyFingerprintApiKeySpki(spki);

    if (req.method === 'GET') {
        // Check if the device api key already exists
        const maybeDeviceApiKey = await DeviceApiKeyFactory.readByFingerprint(
            fingerprint as string
        );
        
        // If it does
        if (maybeDeviceApiKey) {
            // There is a conflict
            res.status(409).send('conflict');
            return;
        }

        // Re-wrap the spki
        const pem = publicPemWrap(spki);
        // Create the new struct
        const deviceApiKey = {
            accountId,
            fingerprint,
            pem,
        };

        // Try to create the device API key in the db
        try {
            await DeviceApiKeyFactory.create(deviceApiKey);
        } catch (e: any) {
            // If the request was formatted incorrectly in a known way
            if (e.name === errors.BadModelFormat.name) {
                // Bad Request
                res.status(400).send('bad request -- bad format');
                return;
            }
            // Otherwise, misc error
            console.log('Error creating device api key: ', e);
            // Bad Request
            res.status(500).send('internal server error');
            return;
        }

        

        // Now that the device API key has been plopped into the db
        // Let's also send a call to core telling it that the 

        // Okay
        res.status(200).send(deviceApiKey);
    }
};
