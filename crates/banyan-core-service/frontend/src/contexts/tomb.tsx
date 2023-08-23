import React, { ReactNode, createContext, useContext, useEffect, useState } from 'react';
import { TombWasm } from 'tomb-wasm-experimental';
import { Mutex } from 'async-mutex';
import { useSession } from 'next-auth/react';

import { useKeystore } from './keystore';
import {
    Bucket, BucketFile, BucketKey,
    BucketSnapshot, Metadata, MockBucket,
} from '@/lib/interfaces/bucket';

interface TombInterface {
    tomb: TombWasm | null;
    buckets: Bucket[];
    usedStorage: number;
    trash: Bucket;
    loadBucket: (id: string) => Promise<void>;
    unlockBucket: (id: string) => Promise<void>;
    takeColdSnapshot: (id: string) => Promise<void>;
    syncBucket: (id: string) => Promise<void>;
    getBuckets: () => Promise<Bucket[]>;
    createDirectory: (id: string, name: string) => Promise<void>;
    uploadFile: (id: string, path: string, file: any) => Promise<void>;
    renameFile: (id: string, path: string, newPath: string) => Promise<void>;
    getTrashBucket: () => Promise<Bucket>;
    getUsedStorage: () => Promise<number>;
    getMetadata: (id: string, path: string) => Promise<Metadata>;
    getBucketShapshots: (id: string) => Promise<BucketSnapshot[]>;
    getBucketKeys: (id: string) => Promise<BucketKey[]>;
    getFiles: (id: string, path: string) => Promise<BucketFile[]>;
    purgeSnapshot: (id: string) => void;
    deleteBucket: (id: string) => void;
    setBuckets: React.Dispatch<React.SetStateAction<Bucket[]>>;
    approveBucketAccess: (id: string) => Promise<void>;
    removeBucketAccess: (id: string) => Promise<void>;
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
    const mutex = async(calllack: (tomb: TombWasm) => Promise<any>) => {
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
    const loadBucket = async(id: string) => {
        await mutex(async tomb => {
            await tomb.load(id);
        });
    };

    const unlockBucket = async(id: string) => {
        await mutex(async tomb => {
            const encryptionKey = await getEncryptionKey();
            await tomb.unlock(id, encryptionKey.privateKey);
        });
    };

    const getFiles = async(id: string, path: string) => {
        await loadBucket(id);
        await unlockBucket(id);

        return await tomb!.ls(id, path);
    };

    const getBuckets = async() => await tomb!.getBuckets();

    const getBucketKeys = async(id: string) => await mutex(async tomb => await tomb!.getBucketKeys(id));

    const deleteBucket = async(id: string) => await tomb!.deleteBucket(id);

    const getBucketShapshots = async(id: string) => await tomb!.getBucketSnapshots(id);

    const approveBucketAccess = async(id: string) => {
        await tomb!.approveBucketAccess(id);
    };

    const removeBucketAccess = async(id: string) => {
        /** TODO:  connect removeBucketAccess method when in will be implemented.  */
        // return await tomb!.approveBucketAccess(id);
    };

    const getUsedStorage = async() => +(await tomb!.getTotalStorage()).toString();

    const takeColdSnapshot = async(id: string) => {
        await tomb!.snapshot(id);
    };

    const getMetadata = async(id: string, path: string) => await tomb!.getMetadata(id, path);

    const purgeSnapshot = async(id: string) => {
        await tomb!.purgeSnapshot(id);
    };

    const createDirectory = async(id: string, name: string) => {
        await tomb!.createDirectory(id, `/${name}`);
    };

    const syncBucket = async(id: string) => {
        await tomb!.syncBucket(id);
    };

    const uploadFile = async(id: string, path: string, file: any) => {
        await tomb!.upload(id, path, file);
    };

    const renameFile = async(id: string, path: string, newPath: string) => {
        await tomb!.rename(id, path, newPath);
    };

    const getTrashBucket: () => Promise<MockBucket> = async() => await tomb!.getTrashBucket();

    // Initialize the tomb client
    useEffect(() => {
        if (!keystoreInitialized || !session?.accountId) { return; }

        (async() => {
            try {
                const wrappingKey = await getEncryptionKey();
                const TombWasm = (await import('tomb-wasm-experimental')).TombWasm;
                const tomb = await new TombWasm(
                    wrappingKey.privateKey,
                    session.accountId,
                    window.location.origin
                );
                setTomb(tomb);
            } catch (err) {
                console.error(err);
            }
        })();
    }, [keystoreInitialized, session?.accountId]);

    useEffect(() => {
        if (tomb) {
            (async() => {
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
        (async() => {
            if (tomb) {
                for (const bucket of buckets) {
                    const id = bucket.id;
                    const files = await getFiles(id, '/');
                    const keys = await getBucketKeys(id);
                    setBuckets(buckets => buckets.map(bucket => bucket.id === id ? { ...bucket, files, keys } : bucket));
                }
            }
        })();
    }, [tomb, buckets.length]);

    return (
        <TombContext.Provider
            value={{
                tomb, buckets, trash, usedStorage,
                setBuckets, getBuckets, getBucketShapshots, loadBucket,
                unlockBucket, getFiles, getTrashBucket, takeColdSnapshot,
                getUsedStorage, createDirectory, uploadFile, renameFile,
                getMetadata, syncBucket, getBucketKeys, purgeSnapshot,
                removeBucketAccess, approveBucketAccess, deleteBucket,
            }}
        >
            {children}
        </TombContext.Provider>
    );
};

export const useTomb = () => useContext(TombContext);
