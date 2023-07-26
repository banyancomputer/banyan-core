import { NextApiRequest, NextApiResponse } from 'next';
import { getServerSession } from 'next-auth/next';
import { authOptions } from '@/pages/api/auth/[...nextauth]';
import { DevicePublicKeyFactory } from '@/lib/db';
import * as apiRequests from '@/lib/api/requests';
import * as apiResponses from '@/lib/api/responses';
import { Session } from 'next-auth';

export default async (req: NextApiRequest, res: NextApiResponse) => {
	// Get the user's session
	// TODO: Fix this ts-ignore s.t. we can type check session
	// @ts-ignore
	const session: Session = await getServerSession(req, res, authOptions);
	if (!session) {
		res.status(401).send({}); // Unauthorized
	}

	if (req.method === 'PUT') {
		let { device_public_key } = req.body as apiRequests.RegisterDevicePublicKey;
		device_public_key = { ...device_public_key, user_id: session.userId };
		try {
			device_public_key = await DevicePublicKeyFactory.create(
				device_public_key
			);
		} catch (e) {
			console.log('Error creating device public key: ', e);
			res.status(500).send('internal server error'); // Bad Request
			return;
		}
		res
			.status(200)
			.send({ device_public_key } as apiResponses.RegisterDevicePublicKey);
		return;
	}

	if (req.method === 'DELETE') {
		let { device_public_key_ecdsa_fingerprint } =
			req.body as apiRequests.DeleteDevicePublicKey;
		try {
			await DevicePublicKeyFactory.deleteByUserIdAndEcdsaFingerprint(
				session.userId,
				device_public_key_ecdsa_fingerprint
			);
		} catch (e) {
			console.log('Error deleting device public key: ', e);
			res.status(500).send('internal server error'); // Bad Request
			return;
		}
		res.status(200).send({}); // Bad Request
		return;
	}
};
