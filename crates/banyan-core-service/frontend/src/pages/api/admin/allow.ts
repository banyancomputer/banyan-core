import { NextApiRequest, NextApiResponse } from 'next';
import { Sequelize, UniqueConstraintError } from 'sequelize';
import { AllowedEmailFactory } from '@/lib/db';

// TODO: This can not be used in production. This route is being kept around for testing purposes.
// Eventually we will need a better solution for adding allowed alpha users / admin management in general.
export default async(req: NextApiRequest, res: NextApiResponse) => {
    // Only allow if running in dev mode
    if (process.env.NODE_ENV !== 'development') {
        res.status(403).send('forbidden'); // Forbidden

        return;
    }

    // Handle GET request
    if (req.method === 'GET') {
        // Return all allowed users
        const allowed = await AllowedEmailFactory.readAll();
        res.status(200).send(allowed);

        return;
    }

    // Handle POST request
    if (req.method === 'POST') {
        console.log('POST request: ', req.body);
        // Get the data from the request
        const data = req.body;
        let allowed;
        try {
            allowed = await AllowedEmailFactory.create(data);
        } catch (e) {
            if (e instanceof UniqueConstraintError) {
                res.status(409).send('conflict'); // Conflict

                return;
            }
            console.log('Error allowed login: ', e);
            res.status(500).send('internal server error'); // Bad Request

            return;
        }
        res.status(200).send(allowed);

        return;
    }

    // Handle DELETE request
    if (req.method === 'DELETE') {
        // Get the data from the request
        const data = req.body;
        if (!data.email) {
            res.status(400).send('bad request'); // Bad Request

            return;
        }
        let allowed;
        try {
            allowed = await AllowedEmailFactory.deleteByEmail(data.email);
        } catch (e) {
            console.log(`Error deleting allowed user: ${e}`);
            res.status(500).send('internal server error'); // Bad Request

            return;
        }
        // Send the allowed user
        res.status(200).send(allowed);

        return;
    }

    res.status(405).send('method not allowed'); // Method Not Allowed
};
