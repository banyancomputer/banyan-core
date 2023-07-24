import { NextApiRequest, NextApiResponse } from 'next';
import { getServerSession } from 'next-auth/next';
import { authOptions } from '../auth/[...nextauth]';
import { EscrowedKeyFactory } from '@/lib/db';
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
		const data = req.body;
		let escrowedKey;
		try {
			escrowedKey = await EscrowedKeyFactory.create(data);
		} catch (e) {
			console.log('Error creating escrowed key: ', e);
			res.status(500).send('internal server error'); // Bad Request
			return;
		}
		res.status(200).send(escrowedKey);
	}

	// Handle GET request
	if (req.method === 'GET') {
		const id = session.id;
		const escrowedKey = await EscrowedKeyFactory.readByOwner(id);
		res.status(200).send(escrowedKey);
	}
};
