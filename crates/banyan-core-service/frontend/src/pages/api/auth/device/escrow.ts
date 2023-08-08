import { NextApiRequest, NextApiResponse } from 'next';
import { getServerSession } from 'next-auth/next';
import { Session } from 'next-auth';
import { authOptions } from '../[...nextauth]';
import * as errors from '@/lib/db/models/errors';
import {
    AccountFactory,
    DeviceApiKeyFactory,
    EscrowedDeviceFactory,
} from '@/lib/db';
import { EscrowedDevice } from '@/lib/interfaces';

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
    // Find the account in the database
    const accountId = await AccountFactory.idFromProviderId(providerId);
    if (!accountId) {
        res.status(404).send('account not found'); // Not Found

        return;
    }

    // Handle POST request
    if (req.method === 'POST') {
        const escrowedDevice = { ...req.body, accountId } as EscrowedDevice;
        try {
            const resp = await EscrowedDeviceFactory.create(escrowedDevice);
            if (!resp) {
                res.status(404).send('not found -- no escrowed device'); // Not Found

                return;
            }
            res.status(200).send(resp);
        } catch (e: any) {
            // Check if the error is a bad model format
            if (e.name === errors.BadModelFormat.name) {
                res.status(400).send(`bad request -- bad format: ${e}`); // Bad Request

                return;
            }
            console.log('Error creating escrowed key material: ', e);
            res
                .status(500)
                .send('internal server error: could not escrow device key material'); // Bad Request
        }

        return;
    }

    // Handle GET request
    if (req.method === 'GET') {
        try {
            const resp = await EscrowedDeviceFactory.readByAccountId(accountId);
            if (!resp) {
                res.status(404).send('not found -- no escrowed device'); // Not Found

                return;
            }
            res.status(200).send(resp);
        } catch (e) {
            console.log('Error reading escrowed device: ', e);
            res
                .status(500)
                .send('internal server error: could not read escrowed device'); // Bad Request
        }
    }
};
