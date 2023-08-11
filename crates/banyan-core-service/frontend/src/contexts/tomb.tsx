import React, { createContext, useContext, useEffect, useState } from 'react';
import { TombWasm } from 'tomb-wasm';
import { useSession } from 'next-auth/react';
import { useKeystore } from './keystore';
import { Mutex } from 'async-mutex';

// TODO: Better typing for tomb-wasm

const TombContext = createContext<{
	tombInitialized: boolean;

    // Banyan Api methods


	// Bucket Api
	loadBucket: (bucket_id: string) => Promise<void>;
	unlockBucket: (bucket_id: string) => Promise<void>;
	lsBucket: (bucket_id: string, path: string) => Promise<any>;
}>({
	tombInitialized: false,
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
	// A mutex to prevent concurrent tomb operations -- wasm is not thread-safe
	const tombMutex = new Mutex();

    /* Effect Hooks */

	// Load the tomb-wasm module
	useEffect(() => {
		import('tomb-wasm').then((module) => {
			setTombModule(module);
		});
	}, []);

	// Initialize the tomb client when the keystore is ready
	useEffect(() => {
		const initTomb = async () => {
			try {
				const apiKey = await getApiKey();
				const tomb_wasm: any = tombModule?.TombWasm;
				const tomb = await new tomb_wasm(
					apiKey?.privateKey,
					session?.accountId,
                    // TODO: Make this configurable
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
		await tombMethod(async (t) => {
			await t.load(bucket_id);
		});
	};

	const unlockBucket = async (bucket_id: string) => {
		await tombMethod(async (t) => {
			const encryptionKey = await getEncryptionKey();
			await t.unlock(bucket_id, encryptionKey.privateKey);
		});
	};

	const lsBucket = async (bucket_id: string, path: string) => {
		return await tombMethod(async (t) => {
			const contents = await t.ls(bucket_id, path);
			return contents;
		});
	};


    /* Helper Methods */

    const tombMethod = async (call: (tomb: TombWasm) => Promise<any>) => {
		if (tomb) {
			const release = await tombMutex.acquire();
			try {
				const result = await call(tomb);
				return result;
			} catch (err) {
				console.error(err);
			} finally {
				release();
			}
		}
	};


	return (
		<TombContext.Provider
			value={{ tombInitialized, loadBucket, unlockBucket, lsBucket }}
		>
			{children}
		</TombContext.Provider>
	);
};

export const useTomb = () => useContext(TombContext);
