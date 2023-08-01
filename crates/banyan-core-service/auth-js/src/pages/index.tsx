import { NextPageWithLayout } from '@/pages/page';
import { useSession } from 'next-auth/react';
import { Session, getServerSession } from 'next-auth';
import { useEffect, useState } from 'react';
import { useKeystore } from '@/contexts/keystore';
import { Button, FormControl } from '@chakra-ui/react';
import Router from 'next/router';
import { authOptions } from './api/auth/[...nextauth]';
import BaseLayout from '@/layouts/base/BaseLayout';
import { signOut } from 'next-auth/react';

// NOTE: we need to dynamically import the TombBucket module in order to use its wasm
import dynamic from 'next/dynamic';
// import { ClientApi } from '@/lib/api/auth';
import { AccountFactory, DeviceApiKeyFactory, EscrowedDeviceFactory } from '@/lib/db';
import { DeviceApiKey, EscrowedDevice } from '@/lib/interfaces';
const TombBucket = dynamic(
	() => import('@/components/tomb/bucket/TombBucket'),
	{ ssr: false }
);

export async function getServerSideProps(context: any) {
	// If the user has a session, serve the page
	const session: Session | null = await getServerSession(
		// @ts-ignore
		context.req,
		context.res,
		authOptions
	);
	if (session) {
		const providerId = session.providerId;
		try { 
			const account_id = await AccountFactory.idFromProviderId(providerId);
			const deviceApiKeys = await DeviceApiKeyFactory.readAllByAccountId(
				account_id
			);
			const escrowedDevice = await EscrowedDeviceFactory.readByAccountId(
				account_id
			);
			return {
				// Just return empty props for now, eventually we'll pass more data
				props: {
					escrowedDevice: JSON.parse(JSON.stringify(escrowedDevice)),
					deviceApiKeys: JSON.parse(JSON.stringify(deviceApiKeys)),
				},
			};
		} catch (error) {
			console.error(error);
			return {
				props: {
					escrowedDevice: null,
					deviceApiKeys: null,
				},
			};
		}
	}
	// If no session, redirect to login
	return {
		redirect: {
			destination: '/login',
			permanent: false,
		},
	};
}

export interface IHomePage {
	escrowedDevice: EscrowedDevice | null;
	deviceApiKeys: DeviceApiKey[] | null;
}

const HomePage: NextPageWithLayout<IHomePage> = ({
	escrowedDevice,
	deviceApiKeys,
}) => {
	const { data: session } = useSession();
	const {
		initializeKeystore,
		keystoreInitialized,
		purgeKeystore,
	} = useKeystore();
	const [passkey, setPasskey] = useState<string>('');
	const [fingerprint, setFingerprint] = useState<string>('');
	const [error, setError] = useState<string | null>(null);

	const handleInitializeKeystore = () => {
		console.log('Acccount: Initializing keystore with passkey');
		initializeKeystore(passkey)
			.then(() => {
				console.log('Acccount: Keystore initialized');
				setError(null);
				Router.reload();
			})
			.catch((error: any) => {
				setError('Failed to initialize keystore: ' + error.message);
			});
	};

	const handlePurgeKeystore = () => {
		console.log('Acccount: Purging keystore');
		if (!session) {
			console.error('Acccount: User not logged in');
			setError('User not logged in');
			return;
		}
		purgeKeystore()
			.then(() => {
				console.log('Acccount: Keystore purged');
				setError(null);
				Router.reload();
			})
			.catch((_) => setError('Acccount: Failed to purge keystore'));
	};

	return (
		<>
			{/* NextAuth Session Information */}
			<div>
				<h1> Signed in as {session?.user?.email} </h1>
				<p> Account ID: {session?.providerId} </p>
				{/* <p> Escrowed Blob: {session?.escrowedDeviceBlob} </p> */}
				<Button
					colorScheme="blue"
					variant="solid"
					ml={4}
					w={40}
					onClick={() => signOut()}
				>
					Sign Out
				</Button>
			</div>
			<div className="flex flex-col gap-2 p-6">
				<h1 className="text-xl">Keystore context</h1>
				<div>
					{keystoreInitialized ? (
						<>
							<h2> Keystore Initialized </h2>
							<Button
								colorScheme="red"
								variant="solid"
								ml={4}
								w={40}
								onClick={handlePurgeKeystore}
							>
								Purge Keystore
							</Button>
						</>
					) : (
						<>
							<h2> Keystore Not Initialized </h2>
							{/* Key pair derivation / recovery form */}
							<div>
								{escrowedDevice ? (
									<p> Enter your passkey to recover your key pair </p>
								) : (
									<p>
										{' '}
										Derive a new key pair from a passkey -- don't forget it!{' '}
									</p>
								)}
								<FormControl>
									<label htmlFor="passkey" className="label">
										<span className="text-xxs !p-0 text-error text-left">
											{error || ''}
										</span>
									</label>
									<input
										id="passkey"
										type="password"
										placeholder="Passkey"
										className="input"
										onChange={(e: any) => setPasskey(e.target.value)}
									/>
									<Button
										colorScheme="blue"
										variant="solid"
										ml={4}
										w={40}
										onClick={handleInitializeKeystore}
									>
										Initialize Keystore
									</Button>
								</FormControl>
							</div>
						</>
					)}
				</div>
			</div>
			<div className="flex flex-col gap-2 p-6">
				<h1>Device API Keys</h1>
				{deviceApiKeys?.map((deviceApiKey: DeviceApiKey) => (
					<div key={deviceApiKey.id}>
						<p>Device API Key ID: {deviceApiKey.fingerprint}</p>
						<p>Device API Key: {deviceApiKey.pem}</p>
					</div>
				))}
			</div>

			<div className="flex flex-col gap-2 p-6">
				<h1> Tomb Wasm stuff </h1>
				<TombBucket />
			</div>
		</>
	);
};

export default HomePage;

HomePage.getLayout = (page) => {
	return <BaseLayout>{page}</BaseLayout>;
};
