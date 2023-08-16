import React, { ReactNode, createContext, useContext, useEffect, useState } from 'react';
import { TombWasm } from 'tomb-wasm-experimental';
import { useSession } from 'next-auth/react';
import { useKeystore } from './keystore';
import { Bucket, BucketFile, MockBucket } from '@/lib/interfaces/bucket';
import { Mutex } from 'async-mutex';

interface TombInterface {
    tomb: TombWasm | null;
    buckets: Bucket[];
    usedStorage: number;
    trash: Bucket;
    loadBucket: (bucket_id: string) => Promise<void>;
    unlockBucket: (bucket_id: string) => Promise<void>;
    getBuckets: () => Promise<Bucket[]>;
    getTrashBucket: () => Promise<Bucket>;
    getUsedStorage: () => Promise<BigInt>;
    getFiles: (bucket_id: string, path: string) => Promise<BucketFile[]>;
    setBuckets: React.Dispatch<React.SetStateAction<Bucket[]>>;
};

const tombMutex = new Mutex();

const TombContext = createContext<TombInterface>({} as TombInterface);

export const TombProvider = ({ children }: { children: ReactNode }) => {
    // The active user's session
    const { data: session } = useSession();
    // The active user's keystore
    const { keystoreInitialized, getEncryptionKey } = useKeystore();
    const [tomb, setTomb] = useState<TombWasm | null>(null);
    const [buckets, setBuckets] = useState<Bucket[]>([]);
    const [trash, setTrash] = useState<Bucket>(new MockBucket());
    const [usedStorage, setUsedStorage] = useState<number>(0);

    /** Prevents rust recursion error. */
    const mutex = async (calllack: (tomb: TombWasm) => Promise<any>) => {
        if (tomb) {
            const release = await tombMutex.acquire();
            try {
                return await calllack(tomb);
            } catch (err) {
                console.error('err', err);
            } finally {
                release();
            }
        }
    };

    // Load the tomb-wasm module
    const loadBucket = async (bucket_id: string) => {
        await mutex(async tomb => {
            await tomb.load(bucket_id);
        });
    };

    const unlockBucket = async (bucket_id: string) => {
        await mutex(async tomb => {
            const encryptionKey = await getEncryptionKey();
            await tomb.unlock(bucket_id, encryptionKey.privateKey);
        });
    };

    const getFiles = async (bucket_id: string, path: string) => {
        await loadBucket(bucket_id);
        await unlockBucket(bucket_id);

        return await tomb!.ls(bucket_id, path);
    };
    const getBuckets = async () => {
        return await tomb!.getBuckets();
    };
    const getUsedStorage = async () => {
        return +(await tomb!.getTotalStorage()).toString();
    };

    const getTrashBucket: () => Promise<MockBucket> = async () => {
        return await tomb!.getTrashBucket();

    };

    // Initialize the tomb client
    useEffect(() => {
        if (!keystoreInitialized || !session?.accountId) { return; }

        (async () => {
            try {
                const wrappingKey = await getEncryptionKey();
                const TombWasm = (await import('tomb-wasm-experimental')).TombWasm;
                const tomb = await new TombWasm(
                    wrappingKey.privateKey,
                    session.accountId,
                    'localhost:3000'
                );
                setTomb(tomb);
            } catch (err) {
                console.error(err);
            }
        })();
    }, [keystoreInitialized, session?.accountId]);

    useEffect(() => {
        if (tomb) {
            (async () => {
                try {
                    const buckets = await getBuckets();
                    setBuckets(buckets.map(bucket => ({ ...bucket, files: [] })));
                    const storage = await getUsedStorage();
                    setUsedStorage(storage);
                    const trash = await getTrashBucket();
                    const files = await getFiles(trash.id, '/');
                    setTrash({ ...trash, files });
                } catch (error: any) { };
            })();
        };
    }, [tomb]);

    useEffect(() => {
        (async () => {
            if (tomb) {
                for (const bucket of buckets) {
                    const id = bucket.id;
                    const files = await getFiles(id, '/');
                    setBuckets(buckets => buckets.map(bucket => bucket.id === id ? { ...bucket, files } : bucket));
                }
            }
        })();
    }, [tomb, buckets.length]);

    return (
        <TombContext.Provider
            value={{ tomb, buckets, trash, usedStorage, setBuckets, getBuckets, loadBucket, unlockBucket, getFiles, getTrashBucket, getUsedStorage }}
        >
            {children}
        </TombContext.Provider>
    );
};

export const useTomb = () => useContext(TombContext);
