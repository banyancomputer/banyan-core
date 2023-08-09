import React, { createContext, useContext, useEffect, useState } from 'react';
import { TombWasm } from 'tomb-wasm-experimental';
import { useSession } from 'next-auth/react';
import { webcrypto } from 'one-webcrypto';
import { useKeystore } from './keystore';
const TombContext = createContext<{
	tombInitialized: boolean;
	tomb: TombWasm | null;

	// Bucket Api
	loadBucket: (bucket_id: string) => Promise<void>;
	// TODO: better typing
	unlockBucket: (bucket_id: string) => Promise<void>;
	lsBucket: (bucket_id: string, path: string) => Promise<any>;
}>({
	tombInitialized: false,
	tomb: null,
	loadBucket: async (bucket_id: string) => {},
	unlockBucket: async (bucket_id: string) => {},
	lsBucket: async (bucket_id: string, path: string) => {},
});

export const TombProvider = ({ children }: any) => {
	// The active user's session
	const { data: session } = useSession();
	// The active user's keystore
	const { keystoreInitialized, getApiKey, getEncryptionKey } = useKeystore();
	// Our tomb-wasm module -- needs to be loaded asynchronously
	const [tombModule, setTombModule] = useState<any | null>(null);
	// Whether the tomb client has been initialized
	const [tombInitialized, setTombInitialized] = useState<boolean>(false);
	// Our tomb client
	const [tomb, setTomb] = useState<TombWasm | null>(null);

	// Load the tomb-wasm module
	useEffect(() => {
		import('tomb-wasm-experimental').then((module) => {
			setTombModule(module);
		});
	}, []);

	// Initialize the tomb client
	useEffect(() => {
		const initTomb = async () => {
			try {
				const wrappingKey = await getEncryptionKey();
				const apiKey = await getApiKey();
				const tomb_wasm: any = tombModule?.TombWasm;
				const tomb = await new tomb_wasm(
					wrappingKey?.privateKey,
					apiKey?.privateKey,
					session?.accountId,
					'localhost:8080'
				);
				setTomb(tomb);
				setTombInitialized(true);
			} catch (err) {
				console.error(err);
			}
			console.log('Tomb initialized');
		};
		if (tombModule && keystoreInitialized && session?.accountId) {
			initTomb();
		} else {
			console.log('Tomb not initialized');
			console.log('tombModule: ', tombModule);
			console.log('keystoreInitialized: ', keystoreInitialized);
			console.log('session?.accountId: ', session?.accountId);
		}
	}, [tombModule, keystoreInitialized, session?.accountId]);

	const loadBucket = async (bucket_id: string) => {
		if (tomb) {
			await tomb.loadBucket(bucket_id);
		}
	};

	const unlockBucket = async (bucket_id: string) => {
		if (tomb) {
			const encryptionKey = await getEncryptionKey();
			await tomb.unlockBucket(bucket_id, encryptionKey.privateKey);
		}
	};

	const lsBucket = async (bucket_id: string, path: string) => {
		if (tomb) {
			const contents = await tomb.lsBucket(bucket_id, path);

			return contents;
		}
	};

	// TODO: Add more Bindings to tomb-wasm

	return (
		<TombContext.Provider
			value={{ tombInitialized, tomb, loadBucket, unlockBucket, lsBucket }}
		>
			{children}
		</TombContext.Provider>
	);
};

export const useTomb = () => useContext(TombContext);
