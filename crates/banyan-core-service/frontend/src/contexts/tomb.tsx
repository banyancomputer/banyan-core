import React, { ReactNode, createContext, useContext, useEffect, useState } from 'react';
import { TombWasm, WasmBucket, WasmMount } from 'tomb-wasm-experimental';
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
    mounts: Map<string, WasmMount>;
    usedStorage: number;
    usageLimit: number;
    trash: Bucket;
    isTrashLoading: boolean;
    areBucketsLoading: boolean;
    mountBucket: (id: string) => Promise<void>;
    takeColdSnapshot: (id: string) => Promise<void>;
    getBuckets: () => Promise<void>;
    createBucket: (name: string, storageClass: string, bucketType: string) => Promise<void>;
    createDirectory: (id: string, name: string) => Promise<void>;
    uploadFile: (id: string, path: string, file: any) => Promise<void>;
    renameFile: (id: string, path: string, newPath: string) => Promise<void>;
    getTrashBucket: () => Promise<void>;
    getUsedStorage: () => Promise<number>;
    getUsageLimit: () => Promise<number>;
    getBucketShapshots: (id: string) => Promise<BucketSnapshot[]>;
    getBucketKeys: (id: string) => Promise<BucketKey[]>;
    purgeSnapshot: (id: string) => void;
    deleteBucket: (id: string) => void;
    approveBucketAccess: (id: string) => Promise<void>;
    removeBucketAccess: (id: string) => Promise<void>;
};

const tombMutex = new Mutex();

const TombContext = createContext<TombInterface>({} as TombInterface);

export const TombProvider = ({ children }: { children: ReactNode }) => {
    // The active user's session
    const { data: session } = useSession();
    // The active user's keystore
    const { keystoreInitialized, getEncryptionKey, getApiKey } = useKeystore();
    const [tomb, setTomb] = useState<TombWasm | null>(null);
    const [buckets, setBuckets] = useState<Bucket[]>([]);
    const [mounts, setMounts] = useState<Map<string, WasmMount>>(new Map());
    const [trash, setTrash] = useState<Bucket>(new MockBucket());
    const [usedStorage, setUsedStorage] = useState<number>(0);
    const [usageLimit, setUsageLimit] = useState<number>(0);
    const [isTrashLoading, setIsTrashLoading] = useState<boolean>(true);
    const [areBucketsLoading, setAreBucketsLoading] = useState<boolean>(true);

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
    const mountBucket = async (id: string) => {
        await mutex(async tomb => {
            let key = await getEncryptionKey();
            return await tomb.mount(id, key);
        });
    };

    const getBuckets = async () => {
        setAreBucketsLoading(true);
        await mutex(async tomb => {
            const buckets = await tomb!.listBuckets().
                catch(err => {
                    console.error(err);
                    return [];
                });
            setBuckets(buckets);
        })
        setAreBucketsLoading(false);
    };

    const createBucket = async (name: string, storageClass: string, bucketType: string) => {
        await mutex(async tomb => {
            let key = await getEncryptionKey();
            await tomb!.createBucket(name, storageClass, bucketType, key.privateKey);
        })
    };

    const getBucketKeys = async (id: string) => await mutex(async tomb => await tomb!.listBucketKeys(id));

    const deleteBucket = async (id: string) => await tomb!.deleteBucket(id);

    const getBucketShapshots = async (id: string) => await tomb!.listBucketSnapshots(id);

    const approveBucketAccess = async (id: string) => {
        /** TODO:  connect approveBucketAccess method when in will be implemented.  */
        // await tomb!.approveBucketAccess(id);
    };

    const removeBucketAccess = async (id: string) => {
        /** TODO:  connect removeBucketAccess method when in will be implemented.  */
        // return await tomb!.approveBucketAccess(id);
    };

    const getUsedStorage = async () => +(await tomb!.getUsage()).toString();

    const getUsageLimit = async () => +(await tomb!.getUsageLimit()).toString();

    const purgeSnapshot = async (id: string) => {
        // await tomb!.purgeSnapshot(id);
    };

    const createDirectory = async (id: string, name: string) => {
        // await tomb!.createDirectory(id, `/${name}`);
    };

    const uploadFile = async (id: string, path: string, file: any) => {
        // await tomb!.upload(id, path, file);
    };

    const renameFile = async (id: string, path: string, newPath: string) => {
        // await tomb!.rename(id, path, newPath);
    };

    const takeColdSnapshot = async () => { }

    /** TODO: implement after adding to tomb-wasm */
    const getTrashBucket: () => Promise<void> = async () => {
        // setIsTrashLoading(true);
        // const trash = await tomb!();
        // const files = await getFiles(trash.id, '/');
        // setTrash({ ...trash, files });
        // setIsTrashLoading(false);
    }

    // Initialize the tomb client
    useEffect(() => {
        if (!keystoreInitialized || !session?.accountId) { return; }

        (async () => {
            try {
                const apiKey = await getApiKey();
                const TombWasm = (await import('tomb-wasm-experimental')).TombWasm;
                const tomb = new TombWasm(
                    apiKey,
                    session.accountId,
                    "http://localhost:3001"
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
                    await getBuckets();
                    const storage = await getUsedStorage();
                    setUsedStorage(storage);
                } catch (error: any) { };
            })();
        };
    }, [tomb]);

    return (
        <TombContext.Provider
            value={{
                tomb, buckets, trash, usedStorage, usageLimit, areBucketsLoading, isTrashLoading, mounts,
                getBuckets, getBucketShapshots, mountBucket, createBucket,
                getTrashBucket, takeColdSnapshot, getUsedStorage, createDirectory,
                uploadFile, renameFile, getBucketKeys, purgeSnapshot,
                removeBucketAccess, approveBucketAccess, deleteBucket, getUsageLimit
            }}
        >
            {children}
        </TombContext.Provider>
    );
};

export const useTomb = () => useContext(TombContext);